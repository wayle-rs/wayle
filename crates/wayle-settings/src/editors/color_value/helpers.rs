//! Widget builders for `ColorValueControl`: dropdown with color swatches
//! and the custom-color `ColorDialogButton`.

use relm4::{
    gtk,
    gtk::{Expression, glib::SignalHandlerId, prelude::*},
    prelude::*,
};
use wayle_config::schemas::styling::ColorValue;

use super::{
    ColorValueControl, ColorValueMsg, conversion::hex_to_rgba, dropdown::setup_dropdown_factory,
    tokens::ColorItem,
};

pub(super) fn build_dropdown(
    items: &[ColorItem],
    current_index: u32,
    sender: &ComponentSender<ColorValueControl>,
) -> (gtk::DropDown, SignalHandlerId) {
    let labels: Vec<String> = items
        .iter()
        .map(|color_item| color_item.label.to_string())
        .collect();

    let string_list = gtk::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());

    let dropdown = gtk::DropDown::new(Some(string_list), Expression::NONE);
    dropdown.set_selected(current_index);
    dropdown.set_cursor_from_name(Some("pointer"));
    dropdown.add_css_class("color-value-dropdown");

    setup_dropdown_factory(&dropdown, items);

    if let Some(popover) = dropdown
        .last_child()
        .and_then(|child| child.downcast::<gtk::Popover>().ok())
    {
        popover.set_halign(gtk::Align::Center);
    }

    let input_sender = sender.input_sender().clone();
    let handler = dropdown.connect_selected_notify(move |dropdown| {
        let _ = input_sender.send(ColorValueMsg::DropdownSelected(dropdown.selected()));
    });

    (dropdown, handler)
}

pub(super) fn build_color_button(
    current: &ColorValue,
    sender: &ComponentSender<ColorValueControl>,
) -> (gtk::ColorDialogButton, SignalHandlerId) {
    let dialog = gtk::ColorDialog::new();
    let button = gtk::ColorDialogButton::new(Some(dialog));
    button.set_cursor_from_name(Some("pointer"));

    button.add_css_class("color-value-swatch");
    button.set_vexpand(false);
    button.set_valign(gtk::Align::Center);

    let is_custom = matches!(current, ColorValue::Custom(_));
    button.set_visible(is_custom);

    if let ColorValue::Custom(hex) = current {
        button.set_rgba(&hex_to_rgba(hex.as_str()));
    }

    let input_sender = sender.input_sender().clone();
    let handler = button.connect_rgba_notify(move |_button| {
        let _ = input_sender.send(ColorValueMsg::ColorButtonChanged);
    });

    (button, handler)
}
