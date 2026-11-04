//! Bridges locale changes from core's headless watcher to the GTK main loop.

use glib::ControlFlow;
use std::time::Duration;

pub fn watch_locale_change<F: Fn(&str) + 'static>(callback: F) {
    let receiver = babydra_core::i18n::watch_locale_changes();
    glib::timeout_add_local(Duration::from_millis(300), move || {
        loop {
            match receiver.try_recv() {
                Ok(locale) => callback(&locale),
                Err(std::sync::mpsc::TryRecvError::Empty) => return ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return ControlFlow::Break,
            }
        }
    });
}
