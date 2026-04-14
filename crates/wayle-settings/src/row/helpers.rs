//! Layout helper for slotting a control widget into a setting row.

use relm4::{gtk, gtk::prelude::*};

pub(super) fn apply_control_layout(
    control: &gtk::Widget,
    root: &gtk::Box,
    slot: &gtk::Box,
    full_width: bool,
) {
    if full_width {
        control.set_hexpand(true);
        root.set_orientation(gtk::Orientation::Vertical);
        root.add_css_class("vertical");
        slot.set_hexpand(true);
        return;
    }

    control.set_hexpand(false);
    control.set_valign(gtk::Align::Center);
}
