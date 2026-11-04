pub use crate::models::config::SidebarItem;
use std::path::PathBuf;

pub fn get_sidebar_layout_path() -> PathBuf {
    crate::config::get_config_dir().join("explore.json")
}

pub fn default_sidebar_items() -> Vec<SidebarItem> {
    let mut default_items = Vec::new();
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

    // Add default folders
    let folders = [
        ("downloads", "explore.downloads", "folder-download", dirs::download_dir(), "Downloads"),
        ("documents", "explore.documents", "folder-documents", dirs::document_dir(), "Documents"),
        ("pictures", "explore.pictures", "folder-pictures", dirs::picture_dir(), "Pictures"),
        ("music", "explore.music", "folder-music", dirs::audio_dir(), "Music"),
        ("desktop", "explore.desktop", "folder-desktop", dirs::desktop_dir(), "Desktop"),
        ("videos", "explore.videos", "folder-videos", dirs::video_dir(), "Videos"),
    ];
    for (id, name, icon, configured_path, fb) in folders {
        let p = configured_path.unwrap_or_else(|| home.join(fb));
        default_items.push(SidebarItem {
            id: id.to_string(),
            name: name.to_string(),
            icon: icon.to_string(),
            path: p,
            is_bookmark: false,
        });
    }

    let trash_path = dirs::data_local_dir()
        .unwrap_or_else(|| home.join(".local/share"))
        .join("Trash/files");
    default_items.push(SidebarItem {
        id: "trash".to_string(),
        name: "explore.trash".to_string(),
        icon: "user-trash".to_string(),
        path: trash_path,
        is_bookmark: false,
    });

    // This PC
    default_items.push(SidebarItem {
        id: "this_pc".to_string(),
        name: "explore.local_disk".to_string(),
        icon: "drive-harddisk".to_string(),
        path: PathBuf::from("/"),
        is_bookmark: false,
    });

    default_items
}

pub fn load_sidebar_layout() -> Vec<SidebarItem> {
    let cfg = crate::config::load_explore_cfg();
    let mut items = if cfg.sidebar_items.is_empty() {
        default_sidebar_items()
    } else {
        cfg.sidebar_items
    };
    items.retain(|i| i.id != "home");
    items
}

pub fn save_sidebar_layout(items: &[SidebarItem]) {
    let mut cfg = crate::config::load_explore_cfg();
    cfg.sidebar_items = items.to_vec();
    crate::config::save_explore_cfg(&cfg);
}
