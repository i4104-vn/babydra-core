//! Bridges headless system event streams onto the GTK main loop.

use babydra_core::services::system::volume::VolumeState;
use glib::ControlFlow;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

fn poll_receiver<T: 'static, F: FnMut(T) + 'static>(
    receiver: Receiver<T>,
    interval: Duration,
    mut callback: F,
) {
    glib::timeout_add_local(interval, move || loop {
        match receiver.try_recv() {
            Ok(value) => callback(value),
            Err(TryRecvError::Empty) => return ControlFlow::Continue,
            Err(TryRecvError::Disconnected) => return ControlFlow::Break,
        }
    });
}

pub fn spawn_brightness_listener<F: Fn(f64) + 'static>(callback: F) {
    poll_receiver(
        babydra_core::services::system::backlight::spawn_brightness_listener(),
        Duration::from_millis(80),
        callback,
    );
}

pub fn spawn_volume_listener<F: Fn(VolumeState) + 'static>(callback: F) {
    poll_receiver(
        babydra_core::services::system::volume::spawn_volume_listener(),
        Duration::from_millis(150),
        callback,
    );
}
