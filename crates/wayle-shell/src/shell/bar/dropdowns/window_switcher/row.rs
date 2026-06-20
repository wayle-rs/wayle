//! Per-window row in the window switcher dropdown.
//!
//! Rows are purely presentational - clicks are handled by the parent
//! `gtk::ListBox`'s `row-activated` signal (wired in `mod.rs`), which maps
//! the activated row's index back to a window key, mirroring the
//! `device_picker` pattern in the audio dropdown.

use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};

use super::messages::WindowInfo;

pub(super) struct WindowRow {
    title: String,
    app_id: String,
    is_active: bool,
    is_highlighted: bool,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for WindowRow {
    type Init = WindowInfo;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            add_css_class: "window-switcher-row",
            set_activatable: true,
            set_cursor_from_name: Some("pointer"),
            #[watch]
            set_css_classes: &self.css_classes(),

            gtk::Box {
                add_css_class: "window-switcher-row-content",
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,

                gtk::Label {
                    add_css_class: "window-switcher-row-title",
                    set_halign: gtk::Align::Start,
                    set_ellipsize: pango::EllipsizeMode::End,
                    #[watch]
                    set_label: &self.title,
                },

                gtk::Label {
                    add_css_class: "window-switcher-row-subtitle",
                    set_halign: gtk::Align::Start,
                    set_ellipsize: pango::EllipsizeMode::End,
                    #[watch]
                    set_visible: !self.app_id.is_empty(),
                    #[watch]
                    set_label: &self.app_id,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            title: init.title,
            app_id: init.app_id,
            is_active: init.is_active,
            is_highlighted: false,
        }
    }
}

impl WindowRow {
    fn css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["window-switcher-row"];
        if self.is_active {
            classes.push("active");
        }
        if self.is_highlighted {
            classes.push("highlighted");
        }
        classes
    }
}
