//! Save / refresh / reapply-scheme handlers for `TomlEditorControl`.

use relm4::{
    gtk as _,
    gtk::{glib, prelude::*},
};
use serde::{Deserialize, Serialize};
use sourceview5::prelude::*;

use super::{
    TomlEditorControl,
    helpers::{deserialize_with_key, serialize_with_key},
};
use crate::app::sourceview_scheme::SCHEME_ID;

impl<T> TomlEditorControl<T>
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    pub(super) fn on_save(&mut self) {
        let start = self.buffer.start_iter();
        let end = self.buffer.end_iter();
        let text = self.buffer.text(&start, &end, false);

        let Some(value) = deserialize_with_key::<T>(&text, self.key) else {
            self.scrolled.add_css_class("error");
            return;
        };

        self.scrolled.remove_css_class("error");
        self.property.set(value);
        self.dirty_badge.set_visible(false);
    }

    pub(super) fn on_refresh(&mut self) {
        self.buffer.block_signal(&self.changed_id);
        let toml_text = serialize_with_key(&self.property.get(), self.key);
        self.buffer.set_text(&toml_text);
        self.buffer.unblock_signal(&self.changed_id);
        self.dirty_badge.set_visible(false);
    }

    pub(super) fn on_reapply_scheme(&mut self) {
        let buffer = self.buffer.clone();

        glib::idle_add_local_once(move || {
            let manager = sourceview5::StyleSchemeManager::default();

            if let Some(scheme) = manager.scheme(SCHEME_ID) {
                buffer.set_style_scheme(Some(&scheme));
            }
        });
    }
}
