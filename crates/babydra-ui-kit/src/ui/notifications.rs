//! Bridges the core notification event stream to UI state on the GTK thread.

use babydra_core::models::NotificationMsg;
use glib::ControlFlow;
use std::time::Duration;

pub fn spawn_notif_dbus() {
    let mut receiver = babydra_core::services::notification::spawn_notif_dbus();
    glib::timeout_add_local(Duration::from_millis(100), move || loop {
        match receiver.try_recv() {
            Ok(NotificationMsg::New {
                summary,
                body,
                icon,
                app_name,
                timeout,
                command,
            }) => {
                babydra_core::services::notification::service::show_notif_popup_with_cmd(
                    &summary, &body, &icon, &app_name, timeout, command,
                );
            }
            Ok(NotificationMsg::Close) => {
                babydra_core::services::notification::service::close_notif_popup();
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                return ControlFlow::Continue;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => {
                return ControlFlow::Break;
            }
        }
    });
}
