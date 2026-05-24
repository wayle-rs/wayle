//! [`KeyboardInput`] private impl methods: label rendering.

use relm4::{ComponentController, gtk};
use wayle_widgets::{prelude::BarButtonInput, utils::force_window_resize};

use super::{component::KeyboardInput, helpers};

impl KeyboardInput {
    pub(super) fn update_label(&self, root: &gtk::Box) {
        let config = self.config.config();
        let format = config.modules.keyboard_input.format.get();
        let layout_alias_map = config.modules.keyboard_input.layout_alias_map.get();

        let label = helpers::format_label(&self.current_layout, &format, &layout_alias_map);
        self.bar_button.emit(BarButtonInput::SetLabel(label));
        force_window_resize(root);
    }
}
