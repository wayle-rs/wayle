//! Page lifecycle: builds the active `SettingsPage` on demand and tears down
//! the previous one after the crossfade finishes so the transition still has
//! a "from" child to animate out of.

use std::time::Duration;

use relm4::{
    gtk::{self, glib},
    prelude::*,
};
use tracing::warn;

use super::SettingsApp;
use crate::pages::page::SettingsPage;

const TRANSITION_CLEANUP_BUFFER_MS: u32 = 50;

impl SettingsApp {
    pub(super) fn show_page(&mut self, id: &'static str) {
        if let Some((current, _)) = &self.current_page
            && *current == id
        {
            return;
        }

        let Some(&factory) = self.factories.get(id) else {
            warn!(page_id = id, "no factory registered for page");
            return;
        };

        let entry = factory(self.config_service.config());
        let new_page = SettingsPage::builder().launch(entry.spec).detach();
        self.stack.add_child(new_page.widget());
        self.stack.set_visible_child(new_page.widget());

        let previous = self.current_page.replace((entry.id, new_page));
        schedule_removal(&self.stack, previous);
    }
}

fn schedule_removal(
    stack: &gtk::Stack,
    previous: Option<(&'static str, Controller<SettingsPage>)>,
) {
    let Some((_, old)) = previous else {
        return;
    };

    let stack = stack.clone();
    let old_widget = old.widget().clone();
    let delay_ms = stack.transition_duration() + TRANSITION_CLEANUP_BUFFER_MS;

    glib::timeout_add_local_once(Duration::from_millis(delay_ms as u64), move || {
        stack.remove(&old_widget);
        drop(old);
    });
}
