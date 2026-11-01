//! Sidebar navigation for the settings window.
//!
//! Renders collapsible section headers and nav items with icons.
//! Emits the selected page ID when the user clicks a nav item.

mod helpers;
mod methods;

use std::collections::{HashMap, HashSet};

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_i18n::t;

use self::helpers::{build_nav_item, build_section_header};

pub(crate) struct NavItem {
    pub(crate) id: &'static str,
    pub(crate) i18n_key: &'static str,
    pub(crate) icon: &'static str,
}

pub(crate) struct NavSection {
    pub(crate) i18n_key: &'static str,
    pub(crate) items: Vec<NavItem>,
}

pub(crate) struct SidebarInit {
    pub(crate) sections: Vec<NavSection>,
}

pub(crate) struct Sidebar {
    pub(super) active_id: &'static str,
    pub(super) collapsed: HashSet<&'static str>,
    pub(super) nav_buttons: HashMap<&'static str, gtk::Button>,
    pub(super) section_items: HashMap<&'static str, gtk::Box>,
    pub(super) section_headers: HashMap<&'static str, gtk::Button>,
}

#[derive(Debug)]
pub(crate) enum SidebarMsg {
    Navigate(&'static str),
    ToggleSection(&'static str),
    ResetAllRequested,
}

#[derive(Debug)]
pub(crate) enum SidebarOutput {
    PageSelected(&'static str),
    ResetAllRequested,
}

const DEFAULT_ACTIVE_ID: &str = "bar-general";

#[relm4::component(pub(crate))]
impl SimpleComponent for Sidebar {
    type Init = SidebarInit;
    type Input = SidebarMsg;
    type Output = SidebarOutput;

    view! {
        gtk::Box {
            add_css_class: "sidebar",
            set_orientation: gtk::Orientation::Vertical,
            set_vexpand: true,
            set_hexpand: false,

            gtk::Box {
                add_css_class: "sidebar-header",
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,

                gtk::Image {
                    set_icon_name: Some("ld-settings-symbolic"),
                    add_css_class: "sidebar-icon",
                },

                gtk::Label {
                    set_label: &t("settings-title"),
                    add_css_class: "sidebar-title",
                },
            },

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[name = "nav"]
                gtk::Box {
                    add_css_class: "sidebar-nav",
                    set_orientation: gtk::Orientation::Vertical,
                },
            },

            gtk::Box {
                add_css_class: "sidebar-footer",
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Button {
                    add_css_class: "sidebar-reset-all",
                    set_cursor_from_name: Some("pointer"),
                    set_hexpand: true,
                    connect_clicked => SidebarMsg::ResetAllRequested,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_halign: gtk::Align::Start,

                        gtk::Image {
                            set_icon_name: Some("ld-rotate-ccw-symbolic"),
                            add_css_class: "sidebar-reset-icon",
                        },

                        gtk::Label {
                            set_label: &t("settings-reset-all"),
                            add_css_class: "sidebar-reset-label",
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut nav_buttons: HashMap<&'static str, gtk::Button> = HashMap::new();
        let mut section_items: HashMap<&'static str, gtk::Box> = HashMap::new();
        let mut section_headers: HashMap<&'static str, gtk::Button> = HashMap::new();

        let widgets = view_output!();

        for section in &init.sections {
            let section_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();
            section_box.add_css_class("sidebar-section");

            let header = build_section_header(section.i18n_key, &sender);
            section_headers.insert(section.i18n_key, header.clone());
            section_box.append(&header);

            let items_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .build();
            items_box.add_css_class("sidebar-section-items");

            for item in &section.items {
                let button = build_nav_item(item, &sender);
                nav_buttons.insert(item.id, button.clone());
                items_box.append(&button);
            }

            section_items.insert(section.i18n_key, items_box.clone());
            section_box.append(&items_box);
            widgets.nav.append(&section_box);
        }

        if let Some(button) = nav_buttons.get(DEFAULT_ACTIVE_ID) {
            button.add_css_class("active");
        }

        let model = Self {
            active_id: DEFAULT_ACTIVE_ID,
            collapsed: HashSet::new(),
            nav_buttons,
            section_items,
            section_headers,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SidebarMsg::Navigate(id) => self.on_navigate(id, &sender),
            SidebarMsg::ToggleSection(section_key) => self.on_toggle_section(section_key),
            SidebarMsg::ResetAllRequested => {
                let _ = sender.output(SidebarOutput::ResetAllRequested);
            }
        }
    }
}
