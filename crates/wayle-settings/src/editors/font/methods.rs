//! Private behavior for `FontPicker`: search, selection, and reset.

use relm4::{gtk, gtk::prelude::*};

use super::picker::FontPicker;

impl FontPicker {
    pub(super) fn apply_search(&mut self, query: &str) {
        let value = if query.is_empty() { None } else { Some(query) };
        self.filter.set_search(value);
    }

    pub(super) fn reset_search(&mut self) {
        self.filter.set_search(None);
    }

    pub(super) fn select_and_close(&mut self, position: u32, popover: &gtk::Popover) {
        let Some(family) = self.family_at(position) else {
            return;
        };

        self.property.set(family);
        popover.popdown();
    }

    fn family_at(&self, position: u32) -> Option<String> {
        let string_object = self
            .filter_model
            .item(position)
            .and_downcast::<gtk::StringObject>()?;

        Some(string_object.string().to_string())
    }
}
