//! Zone row frame. Zone = a left/center/right slot on a bar card.

mod drag_drop;

use std::{fmt, str::FromStr};

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarModule, modules::CustomModuleDefinition},
};
use wayle_i18n::t;

use self::drag_drop::attach_drop_target;
pub(crate) use self::drag_drop::{DragPayload, DropLocation, attach_drag_source};
use super::{
    card::{LayoutCard, LayoutCardMsg},
    module_picker,
};

const CHIP_ROW_SPACING: u32 = 4;
const CHIP_COLUMN_SPACING: u32 = 4;

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
    card_index: usize,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<LayoutCard>,
) -> (gtk::Box, gtk::FlowBox) {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    row.add_css_class("layout-zone");

    let label = gtk::Label::new(Some(&t(zone.i18n_key())));
    label.add_css_class("layout-zone-label");
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Center);

    let chips_box = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .homogeneous(false)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Center)
        .row_spacing(CHIP_ROW_SPACING)
        .column_spacing(CHIP_COLUMN_SPACING)
        .build();

    attach_drop_target(&chips_box, card_index, zone, sender);

    let chips_frame = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .hexpand(true)
        .build();
    chips_frame.add_css_class("layout-zone-items");
    chips_frame.append(&chips_box);

    let add_button = gtk::MenuButton::builder()
        .icon_name("ld-plus-symbolic")
        .tooltip_text(t("settings-layout-add-module"))
        .build();
    add_button.add_css_class("zone-add-btn");
    add_button.set_cursor_from_name(Some("pointer"));
    add_button.set_valign(gtk::Align::Center);

    module_picker::attach::<LayoutCardMsg>(
        &add_button,
        custom_modules.clone(),
        move |module: BarModule| LayoutCardMsg::AddModule(zone, module),
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
    row.append(&chips_frame);
    row.append(&add_button);
    row.append(&group_button);

    (row, chips_box)
}
