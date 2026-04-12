//! Sidebar navigation for the settings window.
//!
//! Renders collapsible section headers and nav items with icons.
//! Emits the selected page ID when the user clicks a nav item.

use std::collections::{HashMap, HashSet};

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_i18n::t;

/// A nav entry that navigates directly to a page.
pub struct NavItem {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub icon: &'static str,
}

/// A labeled group of nav items, e.g. "Appearance".
pub struct NavSection {
    pub i18n_key: &'static str,
    pub items: Vec<NavItem>,
}

/// Init data for the sidebar.
pub struct SidebarInit {
    pub sections: Vec<NavSection>,
}

/// Sidebar component. Manages active highlight and section collapse state.
pub struct Sidebar {
    active_id: &'static str,
    collapsed: HashSet<&'static str>,
    nav_buttons: HashMap<&'static str, gtk4::Button>,
    section_items: HashMap<&'static str, gtk4::Box>,
    section_headers: HashMap<&'static str, gtk4::Button>,
}

#[derive(Debug)]
pub enum SidebarMsg {
    Navigate(&'static str),
    ToggleSection(&'static str),
    ResetAllRequested,
}

#[derive(Debug)]
pub enum SidebarOutput {
    PageSelected(&'static str),
    ResetAllRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for Sidebar {
    type Init = SidebarInit;
    type Input = SidebarMsg;
    type Output = SidebarOutput;

    view! {
        gtk4::Box {
            add_css_class: "sidebar",
            set_orientation: gtk4::Orientation::Vertical,
            set_vexpand: true,
            set_hexpand: false,

            gtk4::Box {
                add_css_class: "sidebar-header",
                set_orientation: gtk4::Orientation::Horizontal,
                set_valign: gtk4::Align::Center,

                gtk4::Image {
                    set_icon_name: Some("ld-settings-symbolic"),
                    add_css_class: "sidebar-icon",
                },

                gtk4::Label {
                    set_label: &t("settings-title"),
                    add_css_class: "sidebar-title",
                },
            },

            gtk4::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk4::PolicyType::Never,

                #[name = "nav"]
                gtk4::Box {
                    add_css_class: "sidebar-nav",
                    set_orientation: gtk4::Orientation::Vertical,
                },
            },

            gtk4::Box {
                add_css_class: "sidebar-footer",
                set_orientation: gtk4::Orientation::Horizontal,

                gtk4::Button {
                    add_css_class: "sidebar-reset-all",
                    set_cursor_from_name: Some("pointer"),
                    set_hexpand: true,
                    connect_clicked => SidebarMsg::ResetAllRequested,

                    gtk4::Box {
                        set_orientation: gtk4::Orientation::Horizontal,
                        set_halign: gtk4::Align::Start,

                        gtk4::Image {
                            set_icon_name: Some("ld-rotate-ccw-symbolic"),
                            add_css_class: "sidebar-reset-icon",
                        },

                        gtk4::Label {
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
        let mut nav_buttons: HashMap<&'static str, gtk4::Button> = HashMap::new();
        let mut section_items: HashMap<&'static str, gtk4::Box> = HashMap::new();
        let mut section_headers: HashMap<&'static str, gtk4::Button> = HashMap::new();

        let widgets = view_output!();

        for section in &init.sections {
            let section_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build();
            section_box.add_css_class("sidebar-section");

            let header = build_section_header(section.i18n_key, &sender);
            section_headers.insert(section.i18n_key, header.clone());
            section_box.append(&header);

            let items_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
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

        let default_active = "general";

        if let Some(button) = nav_buttons.get(default_active) {
            button.add_css_class("active");
        }

        let model = Self {
            active_id: default_active,
            collapsed: HashSet::new(),
            nav_buttons,
            section_items,
            section_headers,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SidebarMsg::Navigate(id) => {
                if let Some(prev) = self.nav_buttons.get(self.active_id) {
                    prev.remove_css_class("active");
                }

                self.active_id = id;

                if let Some(next) = self.nav_buttons.get(id) {
                    next.add_css_class("active");
                }

                let _ = sender.output(SidebarOutput::PageSelected(id));
            }

            SidebarMsg::ToggleSection(section_key) => {
                let Some(items_box) = self.section_items.get(section_key) else {
                    return;
                };
                let header = self.section_headers.get(section_key);

                if self.collapsed.contains(section_key) {
                    self.collapsed.remove(section_key);
                    items_box.set_visible(true);

                    if let Some(header) = header {
                        header.remove_css_class("collapsed");
                    }
                } else {
                    self.collapsed.insert(section_key);
                    items_box.set_visible(false);

                    if let Some(header) = header {
                        header.add_css_class("collapsed");
                    }
                }
            }

            SidebarMsg::ResetAllRequested => {
                let _ = sender.output(SidebarOutput::ResetAllRequested);
            }
        }
    }
}

fn build_section_header(i18n_key: &'static str, sender: &ComponentSender<Sidebar>) -> gtk4::Button {
    let label = gtk4::Label::builder()
        .label(t(i18n_key))
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();

    let chevron = gtk4::Image::builder()
        .icon_name("ld-chevron-down-symbolic")
        .build();
    chevron.add_css_class("sidebar-section-chevron");

    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    content.append(&label);
    content.append(&chevron);

    let button = gtk4::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("sidebar-section-title");
    button.set_cursor_from_name(Some("pointer"));

    let sender = sender.clone();
    button.connect_clicked(move |_| {
        sender.input(SidebarMsg::ToggleSection(i18n_key));
    });

    button
}

fn build_nav_item(item: &NavItem, sender: &ComponentSender<Sidebar>) -> gtk4::Button {
    let icon = gtk4::Image::builder().icon_name(item.icon).build();
    icon.add_css_class("sidebar-item-icon");

    let label = gtk4::Label::builder()
        .label(t(item.i18n_key))
        .hexpand(true)
        .halign(gtk4::Align::Start)
        .build();

    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    content.append(&icon);
    content.append(&label);

    let button = gtk4::Button::new();
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
