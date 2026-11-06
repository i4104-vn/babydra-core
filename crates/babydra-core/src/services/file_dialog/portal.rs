//! XDG Desktop Portal FileChooser D-Bus implementation.
//! Provides `org.freedesktop.impl.portal.FileChooser` for system-wide dialog delegation.

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use tokio::sync::oneshot;
use zbus::interface;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use super::types::{FileDialogMode, FileDialogOptions, FileDialogResult, FileFilter};

/// Request sent from the D-Bus portal backend to the UI dialog runner.
pub struct PortalDialogRequest {
    pub options: FileDialogOptions,
    pub response_tx: oneshot::Sender<FileDialogResult>,
}

/// D-Bus service implementing `org.freedesktop.impl.portal.FileChooser`.
pub struct FileChooserPortalService {
    request_tx: tokio::sync::mpsc::UnboundedSender<PortalDialogRequest>,
}

impl FileChooserPortalService {
    pub fn new(request_tx: tokio::sync::mpsc::UnboundedSender<PortalDialogRequest>) -> Self {
        Self { request_tx }
    }

    async fn execute_dialog(&self, options: FileDialogOptions) -> (u32, HashMap<String, OwnedValue>) {
        let (tx, rx) = oneshot::channel();
        let req = PortalDialogRequest {
            options,
            response_tx: tx,
        };

        if self.request_tx.send(req).is_err() {
            // Service not accepting requests -> Error code 2
            return (2, HashMap::new());
        }

        match rx.await {
            Ok(result) => {
                if result.cancelled || result.paths.is_empty() {
                    // Response code 1: User cancelled
                    (1, HashMap::new())
                } else {
                    let mut ret = HashMap::new();
                    let uris: Vec<String> = result
                        .paths
                        .into_iter()
                        .map(|p| {
                            let s = p.to_string_lossy().to_string();
                            if s.starts_with("file://") {
                                s
                            } else {
                                format!("file://{}", s)
                            }
                        })
                        .collect();

                    if let Ok(val) = OwnedValue::try_from(Value::from(uris)) {
                        ret.insert("uris".to_string(), val);
                    }
                    // Response code 0: Success
                    (0, ret)
                }
            }
            Err(_) => (2, HashMap::new()),
        }
    }
}

fn get_bool_opt(options: &HashMap<String, OwnedValue>, key: &str) -> bool {
    options
        .get(key)
        .and_then(|v| match v.deref() {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
        .unwrap_or(false)
}

fn get_string_opt(options: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    options.get(key).and_then(|v| match v.deref() {
        Value::Str(s) => Some(s.as_str().to_string()),
        _ => None,
    })
}

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserPortalService {
    /// OpenFile method called by xdg-desktop-portal when an app wants to select a file/directory.
    async fn open_file(
        &self,
        _handle: ObjectPath<'_>,
        _app_id: String,
        _parent_window: String,
        title: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        let multiple = get_bool_opt(&options, "multiple");
        let directory = get_bool_opt(&options, "directory");

        let mode = if directory {
            FileDialogMode::OpenFolder
        } else if multiple {
            FileDialogMode::OpenMultipleFiles
        } else {
            FileDialogMode::OpenFile
        };

        let initial_folder = get_string_opt(&options, "current_folder").map(|s| {
            PathBuf::from(s.trim_start_matches("file://"))
        });

        let accept_label = get_string_opt(&options, "accept_label");

        let dialog_title = if title.is_empty() {
            if directory {
                "Open Folder".to_string()
            } else {
                "Open File".to_string()
            }
        } else {
            title
        };

        let dialog_opts = FileDialogOptions {
            title: dialog_title,
            mode,
            initial_folder,
            initial_file: None,
            accept_label,
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        };

        Ok(self.execute_dialog(dialog_opts).await)
    }

    /// SaveFile method called by xdg-desktop-portal when saving a single file.
    async fn save_file(
        &self,
        _handle: ObjectPath<'_>,
        _app_id: String,
        _parent_window: String,
        title: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        let initial_file = get_string_opt(&options, "current_name");

        let initial_folder = get_string_opt(&options, "current_folder").map(|s| {
            PathBuf::from(s.trim_start_matches("file://"))
        });

        let accept_label = get_string_opt(&options, "accept_label");

        let dialog_title = if title.is_empty() {
            "Save File".to_string()
        } else {
            title
        };

        let dialog_opts = FileDialogOptions {
            title: dialog_title,
            mode: FileDialogMode::SaveFile,
            initial_folder,
            initial_file,
            accept_label,
            filters: vec![FileFilter::all_files()],
            default_filter_index: Some(0),
            show_hidden: false,
        };

        Ok(self.execute_dialog(dialog_opts).await)
    }

    /// SaveFiles method called by xdg-desktop-portal when saving multiple files.
    async fn save_files(
        &self,
        _handle: ObjectPath<'_>,
        _app_id: String,
        _parent_window: String,
        title: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        // Fallback to SaveFile behavior
        self.save_file(_handle, _app_id, _parent_window, title, options).await
    }
}

/// Starts the D-Bus Portal server on the session bus.
pub async fn start_portal_service(
    request_tx: tokio::sync::mpsc::UnboundedSender<PortalDialogRequest>,
) -> Result<zbus::Connection, zbus::Error> {
    let conn = zbus::connection::Builder::session()?
        .name("org.freedesktop.impl.portal.desktop.babydra")?
        .serve_at(
            "/org/freedesktop/portal/desktop",
            FileChooserPortalService::new(request_tx),
        )?
        .build()
        .await?;

    Ok(conn)
}
