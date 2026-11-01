//! Hyprland implementation of [`FocusedWindowSource`].
//!
//! Hyprland's `events()` returns a stream tied to `&self` under Rust 2024
//! capture rules, so we forward events through a spawned task + unbounded
//! channel to produce a `'static` stream. The task owns its own [`Arc`] of
//! the service and ends when the consumer drops the stream.

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use tokio::{runtime::Handle, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use wayle_hyprland::{HyprlandEvent, HyprlandService};

use super::{FocusedWindow, FocusedWindowSource};

pub(crate) struct HyprlandFocusedWindowSource {
    service: Arc<HyprlandService>,
}

impl HyprlandFocusedWindowSource {
    pub(crate) fn new(service: Arc<HyprlandService>) -> Self {
        Self { service }
    }
}

impl FocusedWindowSource for HyprlandFocusedWindowSource {
    fn snapshot(&self) -> Option<FocusedWindow> {
        let runtime = Handle::current();
        let client = runtime.block_on(self.service.active_window())?;
        Some(FocusedWindow {
            title: client.title.get(),
            app_id: client.class.get(),
        })
    }

    fn changes(&self) -> BoxStream<'static, Option<FocusedWindow>> {
        let service = Arc::clone(&self.service);
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut events = service.events();
            while let Some(event) = events.next().await {
                let Some(focused) = translate_event(&service, event).await else {
                    continue;
                };
                if tx.send(focused).is_err() {
                    return;
                }
            }
        });

        Box::pin(UnboundedReceiverStream::new(rx))
    }
}

async fn translate_event(
    service: &HyprlandService,
    event: HyprlandEvent,
) -> Option<Option<FocusedWindow>> {
    match event {
        HyprlandEvent::ActiveWindow { class, title } => {
            if class.is_empty() && title.is_empty() {
                Some(None)
            } else {
                Some(Some(FocusedWindow {
                    title,
                    app_id: class,
                }))
            }
        }
        HyprlandEvent::WindowTitleV2 { address, title } => {
            let active = service.active_window().await?;
            if active.address.get() != address {
                return None;
            }
            Some(Some(FocusedWindow {
                title,
                app_id: active.class.get(),
            }))
        }
        _ => None,
    }
}
