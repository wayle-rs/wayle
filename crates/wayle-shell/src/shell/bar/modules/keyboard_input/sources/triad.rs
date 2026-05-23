//! Triad implementation of [`KeyboardLayoutSource`].

use std::sync::Arc;

use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use wayle_triad::{TriadEvent, TriadService};

use super::{CurrentLayout, KeyboardLayoutSource};

pub(crate) struct TriadKeyboardLayoutSource {
    service: Arc<TriadService>,
}

impl TriadKeyboardLayoutSource {
    pub(crate) fn new(service: Arc<TriadService>) -> Self {
        Self { service }
    }
}

impl KeyboardLayoutSource for TriadKeyboardLayoutSource {
    fn snapshot(&self) -> Option<CurrentLayout> {
        current_layout_from(&self.service)
    }

    fn changes(&self) -> BoxStream<'static, Option<CurrentLayout>> {
        let service = Arc::clone(&self.service);
        let initial = current_layout_from(&service);
        let updates = service.events().filter_map(move |event| {
            let layout = translate_event(&service, event);
            async move { layout }
        });

        Box::pin(stream::once(async move { initial }).chain(updates))
    }
}

fn translate_event(service: &TriadService, event: TriadEvent) -> Option<Option<CurrentLayout>> {
    match event {
        TriadEvent::StateChanged => Some(current_layout_from(service)),
        TriadEvent::LayoutStateChanged | TriadEvent::WindowChanged { .. } => None,
    }
}

fn current_layout_from(service: &TriadService) -> Option<CurrentLayout> {
    let layouts = service.keyboard_layouts.get()?;
    let label = layouts.names.get(layouts.current_idx as usize)?.clone();
    Some(CurrentLayout { label })
}
