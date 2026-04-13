//! Chip widgets: module, group, sub-module, add, remove.

use relm4::{factory::FactorySender, gtk, gtk::prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{
        bar::{BarGroup, ModuleRef},
        modules::CustomModuleDefinition,
    },
};

use super::{
    super::{
        card::{LayoutCard, LayoutCardMsg},
        module_picker,
    },
    ZoneId,
    drag_drop::attach_drag_source,
};

const GROUP_NAME_WIDTH_CHARS: i32 = 6;
const GROUP_NAME_MAX_WIDTH_CHARS: i32 = 10;

pub(super) fn build_module_chip(
    module_ref: &ModuleRef,
    card_index: usize,
    zone: ZoneId,
    item_index: usize,
    sender: &FactorySender<LayoutCard>,
) -> gtk::Box {
    let chip = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Start)
        .build();
    chip.add_css_class("module-chip");

    let label = gtk::Label::new(Some(&module_ref.module().to_string()));

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

pub(super) fn build_group_chip(
    group: &BarGroup,
    card_index: usize,
    zone: ZoneId,
    item_index: usize,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &FactorySender<LayoutCard>,
) -> gtk::Box {
    let chip = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Start)
        .build();
    chip.add_css_class("group-chip");

    let name_entry = gtk::Entry::builder()
        .text(&group.name)
        .width_chars(GROUP_NAME_WIDTH_CHARS)
        .max_width_chars(GROUP_NAME_MAX_WIDTH_CHARS)
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
    sender: &FactorySender<LayoutCard>,
) -> gtk::Box {
    let chip = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .halign(gtk::Align::Start)
        .build();
    chip.add_css_class("module-chip");

    let label = gtk::Label::new(Some(&module_ref.module().to_string()));

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
    sender: &FactorySender<LayoutCard>,
) -> gtk::MenuButton {
    let button = gtk::MenuButton::builder()
        .icon_name("ld-plus-symbolic")
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class("chip-add");
    button.set_cursor_from_name(Some("pointer"));

    module_picker::attach(
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
    sender: &FactorySender<LayoutCard>,
) -> gtk::Button {
    let button = build_chip_button("ld-x-symbolic", "chip-remove");

    let remove_sender = sender.input_sender().clone();
    button.connect_clicked(move |_button| {
        let _ = remove_sender.send(LayoutCardMsg::RemoveItem(zone, item_index));
    });

    button
}

fn build_chip_button(icon: &str, css_class: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class(css_class);
    button.set_cursor_from_name(Some("pointer"));

    button
}
