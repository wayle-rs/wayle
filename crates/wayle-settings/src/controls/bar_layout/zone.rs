//! Zone types, chip widget builders, and drag-and-drop attachment.

use std::fmt;

use gtk4::{gdk, prelude::*};
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::{
        bar::{BarGroup, BarItem, ModuleRef},
        modules::CustomModuleDefinition,
    },
};
use wayle_i18n::t;

use super::card::LayoutCardMsg;

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

impl std::str::FromStr for ZoneId {
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
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::Left => "settings-layout-zone-left",
            Self::Center => "settings-layout-zone-center",
            Self::Right => "settings-layout-zone-right",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DragPayload {
    pub card_index: usize,
    pub zone: ZoneId,
    pub item_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DropLocation {
    pub card_index: usize,
    pub zone: ZoneId,
    pub position: usize,
}

pub(super) fn build_zone_row(
    zone: ZoneId,
    items: &[BarItem],
    card_index: usize,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<super::card::LayoutCard>,
) -> (gtk4::Box, gtk4::Box) {
    let row = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    row.add_css_class("layout-zone");

    let label = gtk4::Label::new(Some(&t(zone.i18n_key())));
    label.add_css_class("layout-zone-label");
    label.set_halign(gtk4::Align::Start);
    label.set_valign(gtk4::Align::Center);

    let chips_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .hexpand(true)
        .build();
    chips_box.add_css_class("layout-zone-items");

    rebuild_zone_chips(&chips_box, items, card_index, zone, custom_modules, sender);
    attach_drop_target(&chips_box, card_index, zone, sender);

    let add_button = gtk4::Button::builder()
        .icon_name("ld-plus-symbolic")
        .tooltip_text(t("settings-layout-add-module"))
        .build();
    add_button.add_css_class("zone-add-btn");
    add_button.set_cursor_from_name(Some("pointer"));
    add_button.set_valign(gtk4::Align::Center);

    super::module_picker::attach(
        &add_button,
        custom_modules.clone(),
        move |module| LayoutCardMsg::AddModule(zone, module),
        sender.input_sender().clone(),
    );

    let group_button = gtk4::Button::builder()
        .icon_name("ld-layers-symbolic")
        .tooltip_text(t("settings-layout-add-group"))
        .build();
    group_button.add_css_class("zone-add-btn");
    group_button.set_cursor_from_name(Some("pointer"));
    group_button.set_valign(gtk4::Align::Center);

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
    chips_box: &gtk4::Box,
    items: &[BarItem],
    card_index: usize,
    zone: ZoneId,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<super::card::LayoutCard>,
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
        let empty = gtk4::Label::new(Some(&t("settings-layout-zone-empty")));
        empty.add_css_class("layout-zone-empty");
        chips_box.append(&empty);
    }
}

fn build_module_chip(
    module_ref: &ModuleRef,
    card_index: usize,
    zone: ZoneId,
    item_index: usize,
    sender: &FactorySender<super::card::LayoutCard>,
) -> gtk4::Box {
    let chip = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    chip.add_css_class("module-chip");

    let label = gtk4::Label::new(Some(&module_ref.module().to_string()));

    let remove = build_chip_button("ld-x-symbolic", "chip-remove");
    let remove_sender = sender.input_sender().clone();
    remove.connect_clicked(move |_button| {
        let _ = remove_sender.send(LayoutCardMsg::RemoveItem(zone, item_index));
    });

    chip.append(&label);
    chip.append(&remove);

    attach_drag_source(&chip, card_index, zone, item_index);

    chip
}

fn build_group_chip(
    group: &BarGroup,
    card_index: usize,
    zone: ZoneId,
    item_index: usize,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<super::card::LayoutCard>,
) -> gtk4::Box {
    let chip = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    chip.add_css_class("group-chip");

    let name_entry = gtk4::Entry::builder()
        .text(&group.name)
        .width_chars(6)
        .max_width_chars(10)
        .hexpand(false)
        .build();
    name_entry.add_css_class("group-name-entry");

    let name_sender = sender.input_sender().clone();
    name_entry.connect_changed(move |entry| {
        let _ = name_sender.send(LayoutCardMsg::GroupNameChanged(
            zone,
            item_index,
            entry.text().to_string(),
        ));
    });

    chip.append(&name_entry);

    for (mod_index, module_ref) in group.modules.iter().enumerate() {
        let sub_chip = build_sub_module_chip(module_ref, zone, item_index, mod_index, sender);
        chip.append(&sub_chip);
    }

    let group_add = build_add_button(custom_modules, zone, item_index, sender);
    chip.append(&group_add);

    let remove = build_remove_button(zone, item_index, sender);
    chip.append(&remove);

    attach_drag_source(&chip, card_index, zone, item_index);

    chip
}

fn build_sub_module_chip(
    module_ref: &ModuleRef,
    zone: ZoneId,
    group_index: usize,
    mod_index: usize,
    sender: &FactorySender<super::card::LayoutCard>,
) -> gtk4::Box {
    let chip = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    chip.add_css_class("module-chip");

    let label = gtk4::Label::new(Some(&module_ref.module().to_string()));

    let remove = build_chip_button("ld-x-symbolic", "chip-remove");
    let remove_sender = sender.input_sender().clone();
    remove.connect_clicked(move |_button| {
        let _ = remove_sender.send(LayoutCardMsg::RemoveGroupModule(
            zone,
            group_index,
            mod_index,
        ));
    });

    chip.append(&label);
    chip.append(&remove);

    chip
}

fn build_add_button(
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    zone: ZoneId,
    item_index: usize,
    sender: &FactorySender<super::card::LayoutCard>,
) -> gtk4::Button {
    let button = build_chip_button("ld-plus-symbolic", "chip-add");

    super::module_picker::attach(
        &button,
        custom_modules.clone(),
        move |module| LayoutCardMsg::AddModuleToGroup(zone, item_index, module),
        sender.input_sender().clone(),
    );

    button
}

fn build_remove_button(
    zone: ZoneId,
    item_index: usize,
    sender: &FactorySender<super::card::LayoutCard>,
) -> gtk4::Button {
    let button = build_chip_button("ld-x-symbolic", "chip-remove");

    let remove_sender = sender.input_sender().clone();
    button.connect_clicked(move |_button| {
        let _ = remove_sender.send(LayoutCardMsg::RemoveItem(zone, item_index));
    });

    button
}

fn build_chip_button(icon: &str, css_class: &str) -> gtk4::Button {
    let button = gtk4::Button::builder()
        .icon_name(icon)
        .valign(gtk4::Align::Center)
        .build();
    button.add_css_class(css_class);
    button.set_cursor_from_name(Some("pointer"));

    button
}

fn attach_drag_source(widget: &gtk4::Box, card_index: usize, zone: ZoneId, item_index: usize) {
    let drag = gtk4::DragSource::new();
    drag.set_actions(gdk::DragAction::MOVE);

    let payload = DragPayload {
        card_index,
        zone,
        item_index,
    }
    .encode();

    drag.connect_prepare(move |_source, _x, _y| {
        Some(gdk::ContentProvider::for_value(&payload.to_value()))
    });

    drag.connect_drag_begin(|source, _drag| {
        if let Some(widget) = source.widget() {
            widget.add_css_class("chip-dragging");
        }
    });

    drag.connect_drag_end(|source, _drag, _delete| {
        if let Some(widget) = source.widget() {
            widget.remove_css_class("chip-dragging");
        }
    });

    widget.add_controller(drag);
}

fn attach_drop_target(
    chips_box: &gtk4::Box,
    card_index: usize,
    zone: ZoneId,
    sender: &FactorySender<super::card::LayoutCard>,
) {
    let drop = gtk4::DropTarget::new(gtk4::glib::Type::STRING, gdk::DragAction::MOVE);

    let motion_box = chips_box.clone();

    drop.connect_motion(move |_target, x, _y| {
        let position = compute_drop_position(&motion_box, x);
        clear_drop_highlight(&motion_box);
        highlight_drop_position(&motion_box, position);
        gdk::DragAction::MOVE
    });

    let leave_box = chips_box.clone();

    drop.connect_leave(move |_target| {
        clear_drop_highlight(&leave_box);
    });

    let drop_box = chips_box.clone();
    let drop_sender = sender.output_sender().clone();

    drop.connect_drop(move |_target, value, x, _y| {
        clear_drop_highlight(&drop_box);

        let Ok(payload_str) = value.get::<String>() else {
            return false;
        };

        let Some(from) = DragPayload::decode(&payload_str) else {
            return false;
        };

        let position = compute_drop_position(&drop_box, x);

        let to = DropLocation {
            card_index,
            zone,
            position,
        };

        let _ = drop_sender.send(super::card::LayoutCardOutput::ItemDropped(from, to));
        true
    });

    chips_box.add_controller(drop);
}

fn compute_drop_position(container: &gtk4::Box, drop_x: f64) -> usize {
    let mut position = 0;
    let mut child = container.first_child();

    while let Some(widget) = child {
        let Some(bounds) = widget.compute_bounds(container) else {
            child = widget.next_sibling();
            continue;
        };

        let center_x = f64::from(bounds.x() + bounds.width() / 2.0);

        if drop_x < center_x {
            return position;
        }

        position += 1;
        child = widget.next_sibling();
    }

    position
}

fn highlight_drop_position(container: &gtk4::Box, position: usize) {
    let mut index = 0;
    let mut child = container.first_child();
    let mut last_child: Option<gtk4::Widget> = None;

    while let Some(widget) = child {
        if index == position {
            widget.add_css_class("drop-before");
        }

        last_child = Some(widget.clone());
        index += 1;
        child = widget.next_sibling();
    }

    if position >= index
        && let Some(last) = last_child
    {
        last.add_css_class("drop-after");
    }
}

fn clear_drop_highlight(container: &gtk4::Box) {
    let mut child = container.first_child();

    while let Some(widget) = child {
        widget.remove_css_class("drop-before");
        widget.remove_css_class("drop-after");
        child = widget.next_sibling();
    }
}

impl DragPayload {
    const SEPARATOR: char = ':';

    fn encode(&self) -> String {
        format!(
            "{}{sep}{}{sep}{}",
            self.card_index,
            self.zone,
            self.item_index,
            sep = Self::SEPARATOR,
        )
    }

    fn decode(encoded: &str) -> Option<Self> {
        let mut parts = encoded.splitn(3, Self::SEPARATOR);

        let card_index = parts.next()?.parse().ok()?;
        let zone = parts.next()?.parse().ok()?;
        let item_index = parts.next()?.parse().ok()?;

        Some(Self {
            card_index,
            zone,
            item_index,
        })
    }
}
