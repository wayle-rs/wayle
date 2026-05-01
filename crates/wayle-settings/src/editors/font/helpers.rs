//! Builders for the font-family list model and list-item factory.

use relm4::{
    gtk,
    gtk::{pango, prelude::*},
};

pub(super) fn build_family_list() -> gtk::StringList {
    let probe = gtk::Label::new(None);

    let Some(font_map) = probe.pango_context().font_map() else {
        return gtk::StringList::new(&[]);
    };

    let mut families: Vec<String> = font_map
        .list_families()
        .iter()
        .map(|family: &pango::FontFamily| family.name().to_string())
        .collect();

    families.sort();

    let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
    gtk::StringList::new(&family_refs)
}

pub(super) fn build_name_expression() -> gtk::PropertyExpression {
    gtk::PropertyExpression::new(
        gtk::StringObject::static_type(),
        gtk::Expression::NONE,
        "string",
    )
}

pub(super) fn build_factory() -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .ellipsize(pango::EllipsizeMode::End)
            .hexpand(true)
            .xalign(0.0)
            .build();

        label.add_css_class("font-picker-item");
        list_item.set_child(Some(&label));
    });

    factory.connect_bind(|_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let Some(string_object) = list_item.item().and_downcast::<gtk::StringObject>() else {
            return;
        };

        let Some(child) = list_item.child() else {
            return;
        };

        let Some(label) = child.downcast_ref::<gtk::Label>() else {
            return;
        };

        label.set_label(&string_object.string());
    });

    factory
}
