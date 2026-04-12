//! Zone row and chip rebuild. Zone = a left/center/right slot on a bar card.

mod chips;
mod drag_drop;

use std::{fmt, str::FromStr};

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarItem, modules::CustomModuleDefinition},
};
use wayle_i18n::t;

pub(crate) use self::drag_drop::{DragPayload, DropLocation};
use self::{
    chips::{build_group_chip, build_module_chip},
    drag_drop::attach_drop_target,
};
use super::{
    card::{LayoutCard, LayoutCardMsg},
    module_picker,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoneId {
    Left,
    Center,
    Right,
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "left"),
            Self::Center => write!(f, "center"),
            Self::Right => write!(f, "right"),
        }
    }
}

impl FromStr for ZoneId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

impl ZoneId {
    pub(crate) fn i18n_key(&self) -> &'static str {
        match self {
            Self::Left => "settings-layout-zone-left",
            Self::Center => "settings-layout-zone-center",
            Self::Right => "settings-layout-zone-right",
        }
    }
}

pub(super) fn build_zone_row(
    zone: ZoneId,
    items: &[BarItem],
    card_index: usize,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<LayoutCard>,
) -> (gtk::Box, gtk::Box) {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    row.add_css_class("layout-zone");

    let label = gtk::Label::new(Some(&t(zone.i18n_key())));
    label.add_css_class("layout-zone-label");
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Center);

    let chips_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();
    chips_box.add_css_class("layout-zone-items");

    rebuild_zone_chips(&chips_box, items, card_index, zone, custom_modules, sender);
    attach_drop_target(&chips_box, card_index, zone, sender);

    let add_button = gtk::MenuButton::builder()
        .icon_name("ld-plus-symbolic")
        .tooltip_text(t("settings-layout-add-module"))
        .build();
    add_button.add_css_class("zone-add-btn");
    add_button.set_cursor_from_name(Some("pointer"));
    add_button.set_valign(gtk::Align::Center);

    module_picker::attach(
        &add_button,
        custom_modules.clone(),
        move |module| LayoutCardMsg::AddModule(zone, module),
        sender.input_sender().clone(),
    );

    let group_button = gtk::Button::builder()
        .icon_name("ld-layers-symbolic")
        .tooltip_text(t("settings-layout-add-group"))
        .build();
    group_button.add_css_class("zone-add-btn");
    group_button.set_cursor_from_name(Some("pointer"));
    group_button.set_valign(gtk::Align::Center);

    let group_sender = sender.input_sender().clone();
    group_button.connect_clicked(move |_button| {
        let _ = group_sender.send(LayoutCardMsg::AddGroup(zone));
    });

    row.append(&label);
    row.append(&chips_box);
    row.append(&add_button);
    row.append(&group_button);

    (row, chips_box)
}

pub(super) fn rebuild_zone_chips(
    chips_box: &gtk::Box,
    items: &[BarItem],
    card_index: usize,
    zone: ZoneId,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<LayoutCard>,
) {
    while let Some(child) = chips_box.first_child() {
        chips_box.remove(&child);
    }

    for (item_index, item) in items.iter().enumerate() {
        let chip = match item {
            BarItem::Module(module_ref) => {
                build_module_chip(module_ref, card_index, zone, item_index, sender)
            }
            BarItem::Group(group) => {
                build_group_chip(group, card_index, zone, item_index, custom_modules, sender)
            }
        };

        chips_box.append(&chip);
    }

    if items.is_empty() {
        let empty = gtk::Label::new(Some(&t("settings-layout-zone-empty")));
        empty.add_css_class("layout-zone-empty");
        chips_box.append(&empty);
    }
}
