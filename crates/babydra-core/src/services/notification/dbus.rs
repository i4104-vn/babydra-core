//! Notification daemon lifecycle and a headless event stream.

use crate::models::NotificationMsg;
use crate::services::notification::service::spawn_dbus_listener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    OnceLock,
};
use tokio::sync::broadcast;

static STARTED: AtomicBool = AtomicBool::new(false);
static EVENTS: OnceLock<broadcast::Sender<NotificationMsg>> = OnceLock::new();

/// Starts the notification daemon once and returns a receiver for notification
/// events. UI crates are responsible for consuming these events on their own
/// main loop.
pub fn spawn_notif_dbus() -> broadcast::Receiver<NotificationMsg> {
    let sender = EVENTS.get_or_init(|| {
        let (sender, _) = broadcast::channel(128);
        sender
    });
    let receiver = sender.subscribe();

    if !STARTED.swap(true, Ordering::SeqCst) {
        let (daemon_tx, mut daemon_rx) = tokio::sync::mpsc::unbounded_channel::<NotificationMsg>();
        spawn_dbus_listener(daemon_tx);
        let sender = sender.clone();
        std::thread::Builder::new()
            .name("babydra-notification-events".into())
            .spawn(move || {
                let runtime = tokio::runtime::Runtime::new().expect("create notification runtime");
                runtime.block_on(async move {
                    while let Some(message) = daemon_rx.recv().await {
                        let _ = sender.send(message);
                    }
                });
            })
            .ok();
    }

    receiver
}
