//! File Dialog service providing unified file & directory picking models,
//! places discovery, filter checking, and XDG Desktop Portal D-Bus implementation.

pub mod filter;
pub mod places;
pub mod portal;
pub mod types;

pub use filter::{matches_filter, matches_pattern};
pub use places::{find_matching_place, get_default_places};
pub use portal::{start_portal_service, FileChooserPortalService, PortalDialogRequest};
pub use types::{
    FileDialogMode, FileDialogOptions, FileDialogResult, FileFilter, QuickAccessPlace,
};

