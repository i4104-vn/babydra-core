//! Background listener for system screen brightness changes.

use super::state::get_brightness;
use std::sync::mpsc;
use std::time::Duration;

/// Spawns a background thread checking brightness changes and returns updates
/// through a standard channel for UI adapters to consume.
pub fn spawn_brightness_listener() -> mpsc::Receiver<f64> {
    let (sender, receiver) = mpsc::channel();
    std::thread::Builder::new()
        .name("babydra-brightness-listener".into())
        .spawn(move || {
            let mut last_brightness = get_brightness();

            loop {
                std::thread::sleep(Duration::from_millis(80));

                let current = get_brightness();
                if (current - last_brightness).abs() >= 0.5 {
                    last_brightness = current;
                    if sender.send(current).is_err() { break; }
                }
            }
        })
        .ok();
    receiver
}
