//! Rendering routines for `Chip`. Builds module- or group-variant widgets
//! into the factory root and wires their signals back out as `ChipOutput`.

use relm4::{factory::FactorySender, gtk, gtk::prelude::*};
use wayle_config::schemas::bar::{BarGroup, BarItem, ModuleRef};

use super::{
    super::{module_picker, zone::attach_drag_source},
    Chip, ChipOutput,
    helpers::{GROUP_NAME_MAX_WIDTH_CHARS, GROUP_NAME_WIDTH_CHARS, build_chip_button},
};

impl Chip {
    pub(super) fn render(&self, root: &gtk::Box, sender: &FactorySender<Self>) {
        while let Some(child) = root.first_child() {
            root.remove(&child);
        }

        root.set_css_classes(&[]);

        match &self.item {
            BarItem::Module(module_ref) => self.render_module(root, module_ref, sender),
            BarItem::Group(group) => self.render_group(root, group, sender),
        }

        attach_drag_source(
            root,
            self.card_index.clone(),
            self.zone,
            self.self_index.clone(),
        );
    }

    fn render_module(&self, root: &gtk::Box, module_ref: &ModuleRef, sender: &FactorySender<Self>) {
        root.add_css_class("module-chip");

        let label = gtk::Label::new(Some(&module_ref.module().to_string()));

        let remove = build_chip_button("ld-x-symbolic", "chip-remove");
        let remove_sender = sender.output_sender().clone();
        let self_index = self.self_index.clone();
        remove.connect_clicked(move |_button| {
            let _ = remove_sender.send(ChipOutput::Remove(self_index.clone()));
        });

        root.append(&label);
        root.append(&remove);
    }

    fn render_group(&self, root: &gtk::Box, group: &BarGroup, sender: &FactorySender<Self>) {
        root.add_css_class("group-chip");

        let name_entry = gtk::Entry::builder()
            .text(&group.name)
            .width_chars(GROUP_NAME_WIDTH_CHARS)
            .max_width_chars(GROUP_NAME_MAX_WIDTH_CHARS)
            .hexpand(false)
            .build();
        name_entry.add_css_class("group-name-entry");

        let name_sender = sender.output_sender().clone();
        let name_index = self.self_index.clone();
        name_entry.connect_changed(move |entry| {
            let _ = name_sender.send(ChipOutput::GroupNameChanged(
                name_index.clone(),
                entry.text().to_string(),
            ));
        });

        root.append(&name_entry);

        for (mod_index, module_ref) in group.modules.iter().enumerate() {
            let sub_chip = self.build_sub_chip(module_ref, mod_index, sender);
            root.append(&sub_chip);
        }

        let add_button = self.build_group_add_button(sender);
        root.append(&add_button);

        let remove = build_chip_button("ld-x-symbolic", "chip-remove");
        let remove_sender = sender.output_sender().clone();
        let remove_index = self.self_index.clone();
        remove.connect_clicked(move |_button| {
            let _ = remove_sender.send(ChipOutput::Remove(remove_index.clone()));
        });
        root.append(&remove);
    }

    fn build_sub_chip(
        &self,
        module_ref: &ModuleRef,
        mod_index: usize,
        sender: &FactorySender<Self>,
    ) -> gtk::Box {
        let chip = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .halign(gtk::Align::Start)
            .build();
        chip.add_css_class("module-chip");

        let label = gtk::Label::new(Some(&module_ref.module().to_string()));

        let remove = build_chip_button("ld-x-symbolic", "chip-remove");
        let remove_sender = sender.output_sender().clone();
        let group_index = self.self_index.clone();
        remove.connect_clicked(move |_button| {
            let _ = remove_sender.send(ChipOutput::RemoveGroupModule(
                group_index.clone(),
                mod_index,
            ));
        });

        chip.append(&label);
        chip.append(&remove);

        chip
    }

    fn build_group_add_button(&self, sender: &FactorySender<Self>) -> gtk::MenuButton {
        let button = gtk::MenuButton::builder()
            .icon_name("ld-plus-symbolic")
            .valign(gtk::Align::Center)
            .build();
        button.add_css_class("chip-add");
        button.set_cursor_from_name(Some("pointer"));

        let group_index = self.self_index.clone();
        module_picker::attach(
            &button,
            self.custom_modules.clone(),
            move |module| ChipOutput::AddModuleToGroup(group_index.clone(), module),
            sender.output_sender().clone(),
        );

        button
    }
}
