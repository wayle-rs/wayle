//! Dropdown for ColorValue config properties. Shows grouped token options
//! with color dots, plus Auto, Transparent, and Custom (with ColorDialogButton).

mod conversion;
mod dropdown;
mod tokens;

mod row;
use conversion::{hex_to_rgba, rgba_to_hex};
use dropdown::setup_dropdown_factory;
use relm4::{
    gtk::{Expression, glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::color_value;
use tokens::{CUSTOM_ID, ColorItem, HEADER_ID, build_items, find_index};
use wayle_config::{
    ConfigProperty,
    schemas::styling::{ColorValue, HexColor},
};

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct ColorValueControl {
    property: ConfigProperty<ColorValue>,
    dropdown: gtk::DropDown,
    dropdown_handler: SignalHandlerId,
    color_button: gtk::ColorDialogButton,
    color_button_handler: SignalHandlerId,
    items: Vec<ColorItem>,
    _watcher: WatcherHandle,
}

#[derive(Debug)]
pub(crate) enum ColorValueMsg {
    DropdownSelected(u32),
    ColorButtonChanged,
    Refresh,
}

impl ColorValueControl {
    fn handle_dropdown_selected(&mut self, index: u32) {
        let Some(item) = self.items.get(index as usize) else {
            return;
        };

        if item.id == HEADER_ID {
            return;
        }

        if item.id == CUSTOM_ID {
            self.select_custom();
        } else {
            self.color_button.set_visible(false);
            self.property.set(item.value.clone());
        }
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

    fn handle_color_button_changed(&mut self) {
        let rgba = self.color_button.rgba();
        let hex_str = rgba_to_hex(&rgba);

        let Ok(hex) = HexColor::new(&hex_str) else {
            return;
        };

        self.property.set(ColorValue::Custom(hex));
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
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .hexpand(false)
            .valign(gtk::Align::Center)
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
        let watcher = spawn_property_watcher(&property, move || {
            input_sender.send(ColorValueMsg::Refresh).is_ok()
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
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ColorValueMsg::DropdownSelected(index) => self.handle_dropdown_selected(index),
            ColorValueMsg::ColorButtonChanged => self.handle_color_button_changed(),
            ColorValueMsg::Refresh => self.refresh_from_property(),
        }
    }
}

fn build_dropdown(
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

fn build_color_button(
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
