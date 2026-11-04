//! Adapters for polling headless core subscriptions on the GTK main loop.

use babydra_core::models::desktop::workspace::{WorkspaceReceiver, WorkspaceSnapshot};
use babydra_core::models::network::{NetworkReceiver, NetworkSnapshot};
use babydra_core::models::system::monitor::{MonitorReceiver, MonitorSnapshot};
use glib::{ControlFlow, MainContext};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

pub trait GlibReceiverAttach<T> {
    fn attach<F: FnMut(T) -> ControlFlow + 'static>(
        self,
        context: Option<&MainContext>,
        callback: F,
    );
}

fn attach_receiver<T: 'static, F: FnMut(T) -> ControlFlow + 'static>(
    receiver: Receiver<T>,
    interval: Duration,
    mut callback: F,
) {
    glib::timeout_add_local(interval, move || loop {
        match receiver.try_recv() {
            Ok(value) => {
                if callback(value) == ControlFlow::Break {
                    return ControlFlow::Break;
                }
            }
            Err(TryRecvError::Empty) => return ControlFlow::Continue,
            Err(TryRecvError::Disconnected) => return ControlFlow::Break,
        }
    });
}

impl GlibReceiverAttach<WorkspaceSnapshot> for WorkspaceReceiver {
    fn attach<F: FnMut(WorkspaceSnapshot) -> ControlFlow + 'static>(
        self,
        _context: Option<&MainContext>,
        callback: F,
    ) {
        attach_receiver(self.into_inner(), Duration::from_millis(100), callback);
    }
}

impl GlibReceiverAttach<NetworkSnapshot> for NetworkReceiver {
    fn attach<F: FnMut(NetworkSnapshot) -> ControlFlow + 'static>(
        self,
        _context: Option<&MainContext>,
        callback: F,
    ) {
        attach_receiver(self.into_inner(), Duration::from_millis(200), callback);
    }
}

impl GlibReceiverAttach<MonitorSnapshot> for MonitorReceiver {
    fn attach<F: FnMut(MonitorSnapshot) -> ControlFlow + 'static>(
        self,
        _context: Option<&MainContext>,
        callback: F,
    ) {
        attach_receiver(self.into_inner(), Duration::from_millis(500), callback);
    }
}
