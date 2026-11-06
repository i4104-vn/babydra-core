//! Quick access / places resolver for File Chooser sidebar.

use std::path::{Path, PathBuf};
use crate::i18n::trans;
use super::types::QuickAccessPlace;

/// Resolves standard bookmarks and user directories for the File Chooser sidebar.
pub fn get_default_places() -> Vec<QuickAccessPlace> {
    let mut places = Vec::new();

    // 1. Home directory
    if let Some(home) = dirs::home_dir() {
        places.push(QuickAccessPlace {
            name: trans("file_dialog.place_home"),
            path: home.clone(),
            icon_name: "user-home".to_string(),
        });

        // 2. Desktop
        if let Some(desktop) = dirs::desktop_dir() {
            if desktop.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_desktop"),
                    path: desktop,
                    icon_name: "user-desktop".to_string(),
                });
            }
        }

        // 3. Documents
        if let Some(docs) = dirs::document_dir() {
            if docs.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_documents"),
                    path: docs,
                    icon_name: "folder-documents".to_string(),
                });
            }
        }

        // 4. Downloads
        if let Some(downloads) = dirs::download_dir() {
            if downloads.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_downloads"),
                    path: downloads,
                    icon_name: "folder-download".to_string(),
                });
            }
        }

        // 5. Music
        if let Some(music) = dirs::audio_dir() {
            if music.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_music"),
                    path: music,
                    icon_name: "folder-music".to_string(),
                });
            }
        }

        // 6. Pictures
        if let Some(pics) = dirs::picture_dir() {
            if pics.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_pictures"),
                    path: pics,
                    icon_name: "folder-pictures".to_string(),
                });
            }
        }

        // 7. Videos
        if let Some(vids) = dirs::video_dir() {
            if vids.exists() {
                places.push(QuickAccessPlace {
                    name: trans("file_dialog.place_videos"),
                    path: vids,
                    icon_name: "folder-videos".to_string(),
                });
            }
        }

        // 8. Trash folder
        let trash_path = home.join(".local/share/Trash/files");
        if trash_path.exists() {
            places.push(QuickAccessPlace {
                name: trans("file_dialog.place_trash"),
                path: trash_path,
                icon_name: "user-trash".to_string(),
            });
        }
    }

    // 9. Root Filesystem
    let root = PathBuf::from("/");
    places.push(QuickAccessPlace {
        name: trans("file_dialog.place_root"),
        path: root,
        icon_name: "drive-harddisk".to_string(),
    });

    places
}

/// Returns whether a given directory path matches one of the standard places.
pub fn find_matching_place<'a>(places: &'a [QuickAccessPlace], path: &Path) -> Option<&'a QuickAccessPlace> {
    places.iter().find(|p| p.path == path)
}

