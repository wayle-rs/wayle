//! Hyprland implementation of [`KeyboardLayoutSource`].
//!
//! Hyprland's `events()` returns a stream tied to `&self` under Rust 2024
//! capture rules, so we forward events through a spawned task + unbounded
//! channel to produce a `'static` stream. The task owns its own [`Arc`] of
//! the service and ends when the consumer drops the stream.

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use tokio::{runtime::Handle, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::warn;
use wayle_hyprland::{DeviceInfo, HyprlandEvent, HyprlandService};

use super::{CurrentLayout, KeyboardLayoutSource};

pub(crate) struct HyprlandKeyboardLayoutSource {
    service: Arc<HyprlandService>,
}

impl HyprlandKeyboardLayoutSource {
    pub(crate) fn new(service: Arc<HyprlandService>) -> Self {
        Self { service }
    }
}

impl KeyboardLayoutSource for HyprlandKeyboardLayoutSource {
    fn snapshot(&self) -> Option<CurrentLayout> {
        let runtime = Handle::current();
        match runtime.block_on(self.service.devices()) {
            Ok(devices) => main_keyboard_layout(&devices).map(|label| CurrentLayout {
                label: label.to_string(),
            }),
            Err(err) => {
                warn!(error = %err, "cannot read hyprland keyboard devices");
                None
            }
        }
    }

    fn changes(&self) -> BoxStream<'static, Option<CurrentLayout>> {
        let service = Arc::clone(&self.service);
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut events = service.events();
            while let Some(event) = events.next().await {
                let HyprlandEvent::ActiveLayout { layout, .. } = event else {
                    continue;
                };
                if tx.send(Some(CurrentLayout { label: layout })).is_err() {
                    return;
                }
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}

fn main_keyboard_layout(devices: &DeviceInfo) -> Option<&str> {
    devices
        .keyboards
        .iter()
        .find(|keyboard| keyboard.main)
        .map(|keyboard| keyboard.active_keymap.as_str())
}
