//! Backend logic for screenshot capturing, regional cropping, annotations (pen, shapes, blur),
//! and clipboard/file saving capabilities.

pub mod capture;

pub use capture::{capture_fullscreen, capture_screen, get_screenshot_path};
