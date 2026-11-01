//! [`WindowTitle`] private impl methods: label and icon rendering.

use relm4::{ComponentController, gtk};
use wayle_widgets::{prelude::BarButtonInput, utils::force_window_resize};

use super::{
    component::WindowTitle,
    helpers::{self, IconContext},
};

impl WindowTitle {
    pub(super) fn update_display(&self, format: &str, root: &gtk::Box) {
        let window_title = &self.config.config().modules.window_title;

        let label = helpers::format_label(format, &self.current_title, &self.current_app_id);
        let icon = helpers::resolve_icon(&IconContext {
            title: &self.current_title,
            app_id: &self.current_app_id,
            user_mappings: &window_title.icon_mappings.get(),
            fallback: &window_title.icon_name.get(),
        });

        self.bar_button.emit(BarButtonInput::SetLabel(label));
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
        force_window_resize(root);
    }

    pub(super) fn update_label(&self, format: &str, root: &gtk::Box) {
        let label = helpers::format_label(format, &self.current_title, &self.current_app_id);
        self.bar_button.emit(BarButtonInput::SetLabel(label));
        force_window_resize(root);
    }
}
