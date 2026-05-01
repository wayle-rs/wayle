//! Stateless widget builders for sidebar nav buttons and section headers.

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_i18n::t;

use super::{NavItem, Sidebar, SidebarMsg};

pub(super) fn build_section_header(
    i18n_key: &'static str,
    sender: &ComponentSender<Sidebar>,
) -> gtk::Button {
    let label = gtk::Label::builder()
        .label(t(i18n_key))
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();

    let chevron = gtk::Image::builder()
        .icon_name("ld-chevron-down-symbolic")
        .build();
    chevron.add_css_class("sidebar-section-chevron");

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    content.append(&label);
    content.append(&chevron);

    let button = gtk::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("sidebar-section-title");
    button.set_cursor_from_name(Some("pointer"));

    let sender = sender.clone();
    button.connect_clicked(move |_| {
        sender.input(SidebarMsg::ToggleSection(i18n_key));
    });

    button
}

pub(super) fn build_nav_item(item: &NavItem, sender: &ComponentSender<Sidebar>) -> gtk::Button {
    let icon = gtk::Image::builder().icon_name(item.icon).build();
    icon.add_css_class("sidebar-item-icon");

    let label = gtk::Label::builder()
        .label(t(item.i18n_key))
        .hexpand(true)
        .halign(gtk::Align::Start)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    content.append(&icon);
    content.append(&label);

    let button = gtk::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("sidebar-item");
    button.set_cursor_from_name(Some("pointer"));

    let item_id = item.id;
    let sender = sender.clone();

    button.connect_clicked(move |_| {
        sender.input(SidebarMsg::Navigate(item_id));
    });

    button
}
