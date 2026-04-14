//! Stateless chip widget helpers.

use relm4::{gtk, gtk::prelude::*};

pub(super) const GROUP_NAME_WIDTH_CHARS: i32 = 6;
pub(super) const GROUP_NAME_MAX_WIDTH_CHARS: i32 = 10;

pub(super) fn build_chip_button(icon: &str, css_class: &str) -> gtk::Button {
    let button = gtk::Button::builder()
        .icon_name(icon)
        .valign(gtk::Align::Center)
        .build();
    button.add_css_class(css_class);
    button.set_cursor_from_name(Some("pointer"));

    button
}
