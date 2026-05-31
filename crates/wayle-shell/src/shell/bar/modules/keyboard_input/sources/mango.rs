//! MangoWM implementation of [`KeyboardLayoutSource`].
//!
//! Mango exposes the active layout as a single reactive property, so the
//! source watches it directly.

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use wayle_mango::MangoService;

use super::{CurrentLayout, KeyboardLayoutSource};

pub(crate) struct MangoKeyboardLayoutSource {
    service: Arc<MangoService>,
}

impl MangoKeyboardLayoutSource {
    pub(crate) fn new(service: Arc<MangoService>) -> Self {
        Self { service }
    }
}

impl KeyboardLayoutSource for MangoKeyboardLayoutSource {
    fn snapshot(&self) -> Option<CurrentLayout> {
        self.service.keyboard_layout.get().map(current_layout_from)
    }

    fn changes(&self) -> BoxStream<'static, Option<CurrentLayout>> {
        let updates = self
            .service
            .keyboard_layout
            .watch()
            .map(|layout| layout.map(current_layout_from));
        Box::pin(updates)
    }
}

fn current_layout_from(label: String) -> CurrentLayout {
    CurrentLayout { label }
}
