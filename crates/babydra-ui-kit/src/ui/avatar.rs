//! UI-side avatar normalization before handing bytes to the headless core.

use gdk_pixbuf::Pixbuf;
use std::path::Path;

/// Crops a chosen image to a centered 256px square PNG, then stores it via core.
/// If the image cannot be decoded, the original file is passed through unchanged.
pub fn set_avatar(path: &Path) -> babydra_core::CoreResult<()> {
    let normalized = Pixbuf::from_file(path).ok().and_then(|pixbuf| {
        let width = pixbuf.width();
        let height = pixbuf.height();
        let side = width.min(height);
        if side <= 0 {
            return None;
        }
        let crop = pixbuf.new_subpixbuf((width - side) / 2, (height - side) / 2, side, side);
        crop.scale_simple(256, 256, gdk_pixbuf::InterpType::Bilinear)
            .and_then(|scaled| scaled.save_to_bufferv("png", &[]).ok())
    });

    let Some(bytes) = normalized else {
        return babydra_core::set_avatar(path);
    };

    let temp_path = std::env::temp_dir().join(format!(
        "babydra-avatar-{}-{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::write(&temp_path, bytes)?;
    let result = babydra_core::set_avatar(&temp_path);
    let _ = std::fs::remove_file(temp_path);
    result
}
