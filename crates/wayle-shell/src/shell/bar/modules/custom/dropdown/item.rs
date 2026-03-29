use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};

pub(super) struct DropdownOptionItem {
    label: String,
    is_active: bool,
    interactive: bool,
    ellipsize: pango::EllipsizeMode,
}

pub(super) struct DropdownOptionInit {
    pub label: String,
    pub is_active: bool,
    /// When false, the row is a non-interactive placeholder (e.g., "no items").
    pub interactive: bool,
    pub ellipsize: pango::EllipsizeMode,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for DropdownOptionItem {
    type Init = DropdownOptionInit;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            add_css_class: "custom-dropdown-item",
            #[watch]
            set_activatable: self.interactive,
            #[watch]
            set_selectable: self.interactive,
            #[watch]
            set_cursor_from_name: if self.interactive { Some("pointer") } else { None },
            #[watch]
            set_css_classes: &self.css_classes(),

            gtk::Box {
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "custom-dropdown-item-label",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                    #[watch]
                    set_ellipsize: self.ellipsize,
                    #[watch]
                    set_label: &self.label,
                },

                gtk::Image {
                    add_css_class: "custom-dropdown-item-check",
                    set_icon_name: Some("tb-check-symbolic"),
                    #[watch]
                    set_visible: self.is_active,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            label: init.label,
            is_active: init.is_active,
            interactive: init.interactive,
            ellipsize: init.ellipsize,
        }
    }
}

impl DropdownOptionItem {
    fn css_classes(&self) -> Vec<&str> {
        let mut classes = vec!["custom-dropdown-item"];
        if self.is_active {
            classes.push("selected");
        }
        if !self.interactive {
            classes.push("disabled");
        }
        classes
    }
}
