//! Niri implementation of [`FocusedWindowSource`].
//!
//! Niri's `events()` returns a `'static` stream, so we adapt it directly.

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use wayle_niri::{Event, NiriService};

use super::{FocusedWindow, FocusedWindowSource};

pub(crate) struct NiriFocusedWindowSource {
    service: Arc<NiriService>,
}

impl NiriFocusedWindowSource {
    pub(crate) fn new(service: Arc<NiriService>) -> Self {
        Self { service }
    }
}

impl FocusedWindowSource for NiriFocusedWindowSource {
    fn snapshot(&self) -> Option<FocusedWindow> {
        focused_window_by_id(&self.service, self.service.focused_window_id.get())
    }

    fn changes(&self) -> BoxStream<'static, Option<FocusedWindow>> {
        let service = Arc::clone(&self.service);
        let mapped = service.events().filter_map(move |event| {
            let focused = translate_event(&service, event);
            async move { focused }
        });
        Box::pin(mapped)
    }
}

fn translate_event(service: &NiriService, event: Event) -> Option<Option<FocusedWindow>> {
    match event {
        Event::WindowFocusChanged { id } => Some(focused_window_by_id(service, id)),
        Event::WindowOpenedOrChanged { window } if window.is_focused => Some(Some(FocusedWindow {
            title: window.title.unwrap_or_default(),
            app_id: window.app_id.unwrap_or_default(),
        })),
        _ => None,
    }
}

fn focused_window_by_id(service: &NiriService, id: Option<u64>) -> Option<FocusedWindow> {
    let id = id?;
    let window = service.window(id)?;
    Some(FocusedWindow {
        title: window.title.get().unwrap_or_default(),
        app_id: window.app_id.get().unwrap_or_default(),
    })
}
