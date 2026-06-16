//! MangoWM implementation of [`FocusedWindowSource`].
//!
//! Mango exposes the focused client as a single reactive property, so the
//! source watches it directly.

use std::sync::Arc;

use futures::{StreamExt, stream::BoxStream};
use wayle_mango::{FocusedClient, MangoService};

use super::{FocusedWindow, FocusedWindowSource};

pub(crate) struct MangoFocusedWindowSource {
    service: Arc<MangoService>,
}

impl MangoFocusedWindowSource {
    pub(crate) fn new(service: Arc<MangoService>) -> Self {
        Self { service }
    }
}

impl FocusedWindowSource for MangoFocusedWindowSource {
    fn snapshot(&self) -> Option<FocusedWindow> {
        self.service.focused_client.get().map(focused_window_from)
    }

    fn changes(&self) -> BoxStream<'static, Option<FocusedWindow>> {
        let updates = self
            .service
            .focused_client
            .watch()
            .skip(1)
            .map(|client| client.map(focused_window_from));
        Box::pin(updates)
    }
}

fn focused_window_from(client: FocusedClient) -> FocusedWindow {
    FocusedWindow {
        title: client.title.unwrap_or_default(),
        app_id: client.app_id.unwrap_or_default(),
    }
}
