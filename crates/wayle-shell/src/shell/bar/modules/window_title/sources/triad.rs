//! Triad implementation of [`FocusedWindowSource`].

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use wayle_triad::{TriadEvent, TriadService};

use super::{FocusedWindow, FocusedWindowSource};

pub(crate) struct TriadFocusedWindowSource {
    service: Arc<TriadService>,
}

impl TriadFocusedWindowSource {
    pub(crate) fn new(service: Arc<TriadService>) -> Self {
        Self { service }
    }
}

impl FocusedWindowSource for TriadFocusedWindowSource {
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

fn translate_event(service: &TriadService, event: TriadEvent) -> Option<Option<FocusedWindow>> {
    match event {
        TriadEvent::StateChanged
        | TriadEvent::LayoutStateChanged
        | TriadEvent::WindowChanged { .. } => Some(focused_window_by_id(
            service,
            service.focused_window_id.get(),
        )),
    }
}

fn focused_window_by_id(service: &TriadService, id: Option<u64>) -> Option<FocusedWindow> {
    let id = id?;
    let window = service.window(id)?;
    Some(FocusedWindow {
        title: window.title.get().unwrap_or_default(),
        app_id: window.app_id.get().unwrap_or_default(),
    })
}
