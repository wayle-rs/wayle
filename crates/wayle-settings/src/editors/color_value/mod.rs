//! Dropdown for ColorValue config properties. Shows grouped token options
//! with color dots, plus Auto, Transparent, and Custom (with ColorDialogButton).

mod conversion;
mod dropdown;
mod helpers;
mod methods;
mod row;
mod tokens;

use relm4::{
    gtk,
    gtk::{glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::color_value;
use wayle_config::{ConfigProperty, schemas::styling::ColorValue};

use self::{
    helpers::{build_color_button, build_dropdown},
    tokens::{ColorItem, build_items, find_index},
};
use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct ColorValueControl {
    pub(super) property: ConfigProperty<ColorValue>,
    pub(super) dropdown: gtk::DropDown,
    pub(super) dropdown_handler: SignalHandlerId,
    pub(super) color_button: gtk::ColorDialogButton,
    pub(super) color_button_handler: SignalHandlerId,
    items: Vec<ColorItem>,
    _watcher: WatcherHandle,
}

#[derive(Debug)]
pub(crate) enum ColorValueMsg {
    DropdownSelected(u32),
    ColorButtonChanged,
    Refresh,
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
