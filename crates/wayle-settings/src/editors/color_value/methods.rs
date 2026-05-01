//! Dropdown/color-button behavior for `ColorValueControl`.

use relm4::gtk::prelude::*;
use wayle_config::schemas::styling::{ColorValue, HexColor};

use super::{
    ColorValueControl,
    conversion::{hex_to_rgba, rgba_to_hex},
    tokens::{CUSTOM_ID, HEADER_ID, find_index},
};

impl ColorValueControl {
    pub(super) fn handle_dropdown_selected(&mut self, index: u32) {
        let Some(item) = self.items.get(index as usize) else {
            return;
        };

        if item.id == HEADER_ID {
            return;
        }

        if item.id == CUSTOM_ID {
            self.select_custom();
            return;
        }

        self.color_button.set_visible(false);
        self.property.set(item.value.clone());
    }

    fn select_custom(&mut self) {
        let current_hex = match self.property.get() {
            ColorValue::Custom(hex) => hex,
            _ => HexColor::new("#ffffff").unwrap_or_default(),
        };

        self.property.set(ColorValue::Custom(current_hex.clone()));
        self.color_button.set_visible(true);

        let rgba = hex_to_rgba(current_hex.as_str());

        self.color_button.block_signal(&self.color_button_handler);
        self.color_button.set_rgba(&rgba);
        self.color_button.unblock_signal(&self.color_button_handler);
    }

    pub(super) fn handle_color_button_changed(&mut self) {
        let rgba = self.color_button.rgba();
        let hex_str = rgba_to_hex(&rgba);

        let Ok(hex) = HexColor::new(&hex_str) else {
            return;
        };

        self.property.set(ColorValue::Custom(hex));
    }

    pub(super) fn refresh_from_property(&mut self) {
        let current = self.property.get();
        let index = find_index(&self.items, &current);

        self.dropdown.block_signal(&self.dropdown_handler);
        self.dropdown.set_selected(index);
        self.dropdown.unblock_signal(&self.dropdown_handler);

        let is_custom = matches!(current, ColorValue::Custom(_));
        self.color_button.set_visible(is_custom);

        if let ColorValue::Custom(ref hex) = current {
            let rgba = hex_to_rgba(hex.as_str());

            self.color_button.block_signal(&self.color_button_handler);
            self.color_button.set_rgba(&rgba);
            self.color_button.unblock_signal(&self.color_button_handler);
        }
    }
}
