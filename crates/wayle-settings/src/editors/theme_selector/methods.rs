//! Theme application and list rebuild handlers for `ThemeSelectorControl`.

use relm4::{gtk::prelude::*, prelude::*};

use super::{
    ThemeSelectorControl,
    helpers::{apply_palette, populate_list},
    scaled_max_height,
};

impl ThemeSelectorControl {
    pub(super) fn on_apply(&mut self, name: String) {
        let themes = self.available.get();

        let Some(theme) = themes.iter().find(|entry| entry.name == name) else {
            return;
        };

        apply_palette(&self.palette, &theme.palette);
        self.popover.popdown();
    }

    pub(super) fn on_rebuild_list(&mut self, sender: &ComponentSender<Self>) {
        populate_list(&self.list_box, &self.available.get(), sender);
    }

    pub(super) fn on_scale_changed(&mut self) {
        self.scrolled
            .set_max_content_height(scaled_max_height(self.scale.get().value()));
    }
}
