//! SignalListItemFactory wiring for the ColorValue dropdown rows.

use relm4::{gtk, gtk::prelude::*};

use super::tokens::{AUTO_ID, CUSTOM_ID, ColorItem, HEADER_ID, TRANSPARENT_ID};

pub(super) fn setup_dropdown_factory(dropdown: &gtk::DropDown, items: &[ColorItem]) {
    let item_data: Vec<(&'static str, String)> = items
        .iter()
        .map(|color_item| (color_item.id, color_item.label.clone()))
        .collect();

    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        list_item.set_child(Some(&build_row_template()));
    });

    let data = item_data;

    factory.connect_bind(move |_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let position = list_item.position() as usize;

        let Some((id, label_text)) = data.get(position) else {
            return;
        };

        bind_row(list_item, id, label_text);
    });

    dropdown.set_factory(Some(&factory));
}

fn build_row_template() -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    row.add_css_class("color-value-row");

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("color-value-dot");
    dot.set_vexpand(false);
    dot.set_hexpand(false);
    dot.set_valign(gtk::Align::Center);
    dot.set_halign(gtk::Align::Center);

    let label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    label.add_css_class("color-value-label");

    row.append(&dot);
    row.append(&label);

    row
}

fn bind_row(list_item: &gtk::ListItem, id: &str, label_text: &str) {
    let Some(child) = list_item.child() else {
        return;
    };

    let Some(row) = child.downcast_ref::<gtk::Box>() else {
        return;
    };

    let Some(dot) = row.first_child() else {
        return;
    };

    let Some(label_widget) = dot.next_sibling() else {
        return;
    };

    let Some(label) = label_widget.downcast_ref::<gtk::Label>() else {
        return;
    };

    if id == HEADER_ID {
        dot.set_visible(false);
        label.set_label(label_text);
        label.set_css_classes(&["color-value-group-header"]);
        row.set_css_classes(&["color-value-header-row"]);
        return;
    }

    dot.set_visible(true);
    row.set_css_classes(&["color-value-row"]);
    label.set_css_classes(&["color-value-label"]);
    label.set_label(label_text);

    dot.set_css_classes(&["color-value-dot"]);

    match id {
        AUTO_ID => dot.set_visible(false),
        TRANSPARENT_ID => dot.add_css_class("transparent"),
        CUSTOM_ID => dot.add_css_class("custom"),
        _ => dot.add_css_class(&format!("token-{id}")),
    }
}
