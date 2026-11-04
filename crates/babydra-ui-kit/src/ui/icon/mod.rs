use glib::object::ObjectExt;

pub mod resolver;

pub use resolver::{
    get_fallback_icon, get_file_icon, get_icon_from_svg, get_icon_name_for_file, get_logo_png,
    set_fallback_icon, set_file_icon,
};

/// Whether dark mode is currently active.
pub fn is_dark_mode() -> bool {
    crate::ui::theme::is_dark_mode()
}

/// Returns the current `icon colored`.
pub fn get_icon_colored(name: &str, size: i32, color_hex: &str) -> gtk4::Image {
    let final_color = if !is_dark_mode() {
        let normalized = color_hex.replace(' ', "");
        match normalized.as_str() {
            "rgba(255,255,255,0.8)" | "rgba(255,255,255,0.80)" => {
                "rgba(28, 28, 30, 0.85)".to_string()
            }
            "rgba(255,255,255,0.7)" | "rgba(255,255,255,0.70)" => {
                "rgba(28, 28, 30, 0.75)".to_string()
            }
            "rgba(255,255,255,0.6)" | "rgba(255,255,255,0.60)" => {
                "rgba(28, 28, 30, 0.65)".to_string()
            }
            "rgba(255,255,255,0.5)" | "rgba(255,255,255,0.50)" => {
                "rgba(28, 28, 30, 0.55)".to_string()
            }
            "rgba(255,255,255,0.4)" | "rgba(255,255,255,0.40)" => {
                "rgba(28, 28, 30, 0.45)".to_string()
            }
            _ => {
                if let Some(alpha_str) = normalized
                    .strip_prefix("rgba(255,255,255,")
                    .and_then(|s| s.strip_suffix(')'))
                {
                    if let Ok(alpha) = alpha_str.parse::<f32>() {
                        let mapped_alpha = (alpha + 0.05).min(1.0);
                        format!("rgba(28, 28, 30, {:.2})", mapped_alpha)
                    } else {
                        color_hex.to_string()
                    }
                } else {
                    color_hex.to_string()
                }
            }
        }
    } else {
        color_hex.to_string()
    };

    if let Some((dark_svg, light_svg)) = get_icon_svg_pair(name) {
        let is_dark = is_dark_mode();
        let base_svg = if is_dark { dark_svg } else { light_svg };
        let colored_svg = base_svg
            .replace("#ffffff", &final_color)
            .replace("#FFFFFF", &final_color)
            .replace("#1c1c1e", &final_color)
            .replace("#1C1C1E", &final_color);
        return get_icon_from_svg(&colored_svg, size);
    }
    get_fallback_icon(name, "image-missing")
}

type IconWatchItem = (glib::WeakRef<gtk4::Image>, String, i32);

thread_local! {
    static ICON_WATCHERS: std::cell::RefCell<Vec<IconWatchItem>> = const { std::cell::RefCell::new(Vec::new()) };
    static WATCHER_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn register_icon_watcher(img: &gtk4::Image, name: &str, size: i32) {
    if let Some(settings) = gtk4::Settings::default() {
        WATCHER_REGISTERED.with(|registered| {
            if !registered.get() {
                settings.connect_gtk_application_prefer_dark_theme_notify(|_| {
                    ICON_WATCHERS.with(|watchers| {
                        let mut list = watchers.borrow_mut();
                        list.retain(|(weak_img, name, size)| {
                            if let Some(img) = weak_img.upgrade() {
                                load_icon_image_data(&img, name, *size);
                                true
                            } else {
                                false
                            }
                        });
                    });
                });
                registered.set(true);
            }
        });

        ICON_WATCHERS.with(|watchers| {
            watchers
                .borrow_mut()
                .push((img.downgrade(), name.to_string(), size));
        });
    }
}

fn get_icon_svg_pair(name: &str) -> Option<(&'static str, &'static str)> {
    babydra_icons::resolve_icon_pair(name)
}

/// Returns the SVG source for an icon in the requested theme mode.
pub fn get_icon_svg(name: &str, dark_mode: bool) -> Option<&'static str> {
    let (dark, light) = get_icon_svg_pair(name)?;
    Some(if dark_mode { dark } else { light })
}

fn load_icon_image_data(img: &gtk4::Image, name: &str, size: i32) {
    if name == "logo" {
        let logo_img = get_logo_png(size);
        img.set_paintable(logo_img.paintable().as_ref());
        img.set_pixel_size(size);
        return;
    }
    let is_dark = is_dark_mode();

    if let Some((dark_svg, light_svg)) = get_icon_svg_pair(name) {
        let svg_content = if is_dark { dark_svg } else { light_svg };
        let icon_img = get_icon_from_svg(svg_content, size);
        img.set_paintable(icon_img.paintable().as_ref());
        img.set_pixel_size(size);
    } else {
        let icon_img = get_fallback_icon(name, "image-missing");
        img.set_paintable(icon_img.paintable().as_ref());
        img.set_pixel_size(size);
    }
}

/// Sets the image content from local SVG or system icon theme.
pub fn set_image_from_icon(img: &gtk4::Image, name: &str, size: i32) {
    load_icon_image_data(img, name, size);
    ICON_WATCHERS.with(|watchers| {
        let mut list = watchers.borrow_mut();
        for (weak_img, watch_name, watch_size) in list.iter_mut() {
            if let Some(target) = weak_img.upgrade() {
                if target == *img {
                    *watch_name = name.to_string();
                    *watch_size = size;
                    return;
                }
            }
        }
    });
}

/// Helper function to retrieve an SVG icon widget by name. Defaults to white in dark mode and dark gray in light mode.
/// Automatically updates icon paintable when theme switches between Dark and Light mode.
pub fn get_icon(name: &str, size: i32) -> gtk4::Image {
    let img = gtk4::Image::new();
    load_icon_image_data(&img, name, size);
    register_icon_watcher(&img, name, size);
    img
}

/// Retrieves an icon by name, prioritizing the active system icon theme (GtkIconTheme).
/// If the theme does not have the icon, falls back to the embedded SVG icon.
pub fn get_theme_or_svg_icon(name: &str, size: i32) -> gtk4::Image {
    let path = std::path::Path::new(name);
    if path.is_absolute() && path.exists() {
        let img = gtk4::Image::from_file(path);
        img.set_pixel_size(size);
        return img;
    }

    let display = gdk4::Display::default();
    if let Some(ref disp) = display {
        let theme = gtk4::IconTheme::for_display(disp);

        if theme.has_icon(name) {
            let img = gtk4::Image::from_icon_name(name);
            img.set_pixel_size(size);
            return img;
        }

        let alias = if name == "babydra-explore" && theme.has_icon("system-file-manager") {
            Some("system-file-manager")
        } else if name == "trash" && theme.has_icon("user-trash") {
            Some("user-trash")
        } else if name == "user-trash" && theme.has_icon("trash") {
            Some("trash")
        } else if name == "drive-removable-media" && theme.has_icon("media-removable") {
            Some("media-removable")
        } else {
            name.strip_suffix("-symbolic")
                .filter(|base| theme.has_icon(base))
        };

        if let Some(resolved) = alias {
            let img = gtk4::Image::from_icon_name(resolved);
            img.set_pixel_size(size);
            return img;
        }
    }

    get_icon(name, size)
}

