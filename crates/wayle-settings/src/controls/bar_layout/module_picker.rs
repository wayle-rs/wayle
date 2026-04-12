//! Searchable module picker popover. Attaches to a button and shows
//! a filtered list of all built-in and custom bar modules.

use std::rc::Rc;

use gtk4::prelude::*;
use relm4::Sender;
use serde::{Deserialize, de::value::StrDeserializer};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarModule, modules::CustomModuleDefinition},
};
use wayle_i18n::t;

use super::card::LayoutCardMsg;

pub(super) fn attach(
    button: &gtk4::MenuButton,
    custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    build_msg: impl Fn(BarModule) -> LayoutCardMsg + 'static,
    sender: Sender<LayoutCardMsg>,
) {
    let popover = gtk4::Popover::new();
    popover.add_css_class("module-picker-popover");
    button.set_popover(Some(&popover));

    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();

    let search = gtk4::SearchEntry::new();
    search.set_placeholder_text(Some(&t("settings-layout-search")));

    let list = gtk4::ListBox::new();
    list.add_css_class("module-picker-list");
    list.set_selection_mode(gtk4::SelectionMode::None);

    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    scrolled.add_css_class("module-picker-scroll");

    content.append(&search);
    content.append(&scrolled);
    popover.set_child(Some(&content));

    let build_msg: Rc<dyn Fn(BarModule) -> LayoutCardMsg> = Rc::new(build_msg);
    let custom_modules = Rc::new(custom_modules);

    populate_list(&list, "", &custom_modules, &sender, &popover, &build_msg);

    let filter_list = list.clone();
    let filter_sender = sender.clone();
    let filter_popover = popover.clone();
    let filter_build = Rc::clone(&build_msg);
    let filter_custom = Rc::clone(&custom_modules);

    search.connect_search_changed(move |search_entry| {
        let query = search_entry.text().to_string().to_lowercase();
        populate_list(
            &filter_list,
            &query,
            &filter_custom,
            &filter_sender,
            &filter_popover,
            &filter_build,
        );
    });

    let search_reset = search.clone();
    popover.connect_show(move |_popover| {
        search_reset.set_text("");
    });
}

fn all_module_names(custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>) -> Vec<String> {
    let builtin: Vec<String> = BarModule::builtin_names()
        .iter()
        .map(|module_name| module_name.to_string())
        .collect();

    let custom: Vec<String> = custom_modules
        .get()
        .iter()
        .map(|custom_module| format!("custom-{}", custom_module.id))
        .collect();

    [builtin, custom].concat()
}

fn populate_list(
    list: &gtk4::ListBox,
    filter: &str,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &Sender<LayoutCardMsg>,
    popover: &gtk4::Popover,
    build_msg: &Rc<dyn Fn(BarModule) -> LayoutCardMsg>,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for module_name in all_module_names(custom_modules) {
        if !filter.is_empty() && !module_name.contains(filter) {
            continue;
        }

        let row = gtk4::ListBoxRow::builder().selectable(false).build();
        row.set_cursor_from_name(Some("pointer"));

        let label = gtk4::Label::builder()
            .label(&module_name)
            .halign(gtk4::Align::Start)
            .build();
        label.add_css_class("module-picker-item");

        row.set_child(Some(&label));

        let click_sender = sender.clone();
        let click_popover = popover.clone();
        let click_build = Rc::clone(build_msg);

        let click = gtk4::GestureClick::new();
        click.connect_released(move |gesture, _n_press, _x, _y| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);

            let deserializer: StrDeserializer<'_, serde::de::value::Error> =
                StrDeserializer::new(&module_name);

            if let Ok(module) = BarModule::deserialize(deserializer) {
                let _ = click_sender.send(click_build(module));
                click_popover.popdown();
            }
        });

        row.add_controller(click);
        list.append(&row);
    }
}
