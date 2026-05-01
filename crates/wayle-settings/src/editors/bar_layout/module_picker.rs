//! Searchable module picker popover. Attaches to a button and shows
//! a filtered list of all built-in and custom bar modules.

use std::rc::Rc;

use relm4::{Sender, gtk, gtk::prelude::*};
use serde::{
    Deserialize,
    de::value::{Error as SerdeValueError, StrDeserializer},
};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarModule, modules::CustomModuleDefinition},
};
use wayle_i18n::t;

pub(super) fn attach<M>(
    button: &gtk::MenuButton,
    custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    build_msg: impl Fn(BarModule) -> M + 'static,
    sender: Sender<M>,
) where
    M: 'static,
{
    let popover = gtk::Popover::new();
    popover.add_css_class("module-picker-popover");
    button.set_popover(Some(&popover));

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(&t("settings-layout-search")));
    search.set_valign(gtk::Align::Center);

    if let Some(icon) = search.first_child() {
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::Center);
        icon.add_css_class("module-picker-search-icon");
    }

    let list = gtk::ListBox::new();
    list.add_css_class("module-picker-list");
    list.set_selection_mode(gtk::SelectionMode::None);

    let scrolled = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    scrolled.add_css_class("module-picker-scroll");

    content.append(&search);
    content.append(&scrolled);
    popover.set_child(Some(&content));

    let build_msg: Rc<dyn Fn(BarModule) -> M> = Rc::new(build_msg);
    let custom_modules = Rc::new(custom_modules);

    populate_list(&list, "", &custom_modules, &sender, &popover, &build_msg);

    let filter_list = list.clone();
    let filter_sender = sender.clone();
    let filter_popover = popover.clone();
    let filter_build = Rc::clone(&build_msg);
    let filter_custom = Rc::clone(&custom_modules);

    search.connect_search_changed(move |search_entry| {
        let query = search_entry.text().to_lowercase();
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

fn populate_list<M: 'static>(
    list: &gtk::ListBox,
    filter: &str,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
    sender: &Sender<M>,
    popover: &gtk::Popover,
    build_msg: &Rc<dyn Fn(BarModule) -> M>,
) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    for module_name in all_module_names(custom_modules) {
        if !filter.is_empty() && !module_name.contains(filter) {
            continue;
        }

        let row = gtk::ListBoxRow::builder().selectable(false).build();
        row.set_cursor_from_name(Some("pointer"));

        let label = gtk::Label::builder()
            .label(&module_name)
            .halign(gtk::Align::Start)
            .build();
        label.add_css_class("module-picker-item");

        row.set_child(Some(&label));

        let click_sender = sender.clone();
        let click_popover = popover.clone();
        let click_build = Rc::clone(build_msg);

        let click = gtk::GestureClick::new();
        click.connect_released(move |gesture, _n_press, _x, _y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);

            let deserializer: StrDeserializer<'_, SerdeValueError> =
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
