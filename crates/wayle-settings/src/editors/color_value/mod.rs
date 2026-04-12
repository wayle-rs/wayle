//! Dropdown for ColorValue config properties. Shows grouped token options
//! with color dots, plus Auto, Transparent, and Custom (with ColorDialogButton).

mod helpers;

use gtk4::{Expression, prelude::*};
use helpers::{
    CUSTOM_ID, ColorItem, HEADER_ID, build_items, find_index, hex_to_rgba, rgba_to_hex,
    setup_dropdown_factory,
};
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::styling::{ColorValue, HexColor},
};

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct ColorValueControl {
    property: ConfigProperty<ColorValue>,
    dropdown: gtk4::DropDown,
    dropdown_handler: gtk4::glib::SignalHandlerId,
    color_button: gtk4::ColorDialogButton,
    color_button_handler: gtk4::glib::SignalHandlerId,
    items: Vec<ColorItem>,
}

#[derive(Debug)]
pub(crate) enum ColorValueMsg {
    DropdownSelected(u32),
    ColorButtonChanged,
    Refresh,
}

impl ColorValueControl {
    fn handle_dropdown_selected(&mut self, index: u32) -> bool {
        let Some(item) = self.items.get(index as usize) else {
            return false;
        };

        if item.id == HEADER_ID {
            return false;
        }

        if item.id == CUSTOM_ID {
            self.select_custom();
        } else {
            self.color_button.set_visible(false);
            self.property.set(item.value.clone());
        }

        true
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

    fn handle_color_button_changed(&mut self) -> bool {
        let rgba = self.color_button.rgba();
        let hex_str = rgba_to_hex(&rgba);

        let Ok(hex) = HexColor::new(&hex_str) else {
            return false;
        };

        self.property.set(ColorValue::Custom(hex));
        true
    }

    fn refresh_from_property(&mut self) {
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

impl SimpleComponent for ColorValueControl {
    type Init = ConfigProperty<ColorValue>;
    type Input = ColorValueMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .hexpand(false)
            .valign(gtk4::Align::Center)
            .build()
    }

    fn init(
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("color-value-control");

        let items = build_items();
        let current = property.get();
        let current_index = find_index(&items, &current);

        let (dropdown, dropdown_handler) = build_dropdown(&items, current_index, &sender);

        let (color_button, color_button_handler) = build_color_button(&current, &sender);

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            let _ = input_sender.send(ColorValueMsg::Refresh);
        });

        root.append(&color_button);
        root.append(&dropdown);

        let model = Self {
            property,
            dropdown,
            dropdown_handler,
            color_button,
            color_button_handler,
            items,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ColorValueMsg::DropdownSelected(index) => {
                if self.handle_dropdown_selected(index) {
                    let _ = sender.output(ControlOutput::ValueChanged);
                }
            }

            ColorValueMsg::ColorButtonChanged => {
                if self.handle_color_button_changed() {
                    let _ = sender.output(ControlOutput::ValueChanged);
                }
            }

            ColorValueMsg::Refresh => self.refresh_from_property(),
        }
    }
}

fn build_dropdown(
    items: &[ColorItem],
    current_index: u32,
    sender: &ComponentSender<ColorValueControl>,
) -> (gtk4::DropDown, gtk4::glib::SignalHandlerId) {
    let labels: Vec<String> = items
        .iter()
        .map(|color_item| color_item.label.to_string())
        .collect();

    let string_list = gtk4::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());

    let dropdown = gtk4::DropDown::new(Some(string_list), Expression::NONE);
    dropdown.set_selected(current_index);
    dropdown.set_cursor_from_name(Some("pointer"));
    dropdown.add_css_class("color-value-dropdown");

    setup_dropdown_factory(&dropdown, items);

    if let Some(popover) = dropdown
        .last_child()
        .and_then(|child| child.downcast::<gtk4::Popover>().ok())
    {
        popover.set_halign(gtk4::Align::Center);
    }

    let input_sender = sender.input_sender().clone();
    let handler = dropdown.connect_selected_notify(move |dropdown| {
        let _ = input_sender.send(ColorValueMsg::DropdownSelected(dropdown.selected()));
    });

    (dropdown, handler)
}

fn build_color_button(
    current: &ColorValue,
    sender: &ComponentSender<ColorValueControl>,
) -> (gtk4::ColorDialogButton, gtk4::glib::SignalHandlerId) {
    let dialog = gtk4::ColorDialog::new();
    let button = gtk4::ColorDialogButton::new(Some(dialog));
    button.set_cursor_from_name(Some("pointer"));

    button.add_css_class("color-value-swatch");
    button.set_vexpand(false);
    button.set_valign(gtk4::Align::Center);

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
