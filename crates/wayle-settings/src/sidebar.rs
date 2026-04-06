//! Sidebar navigation for the settings window.
//!
//! Renders section headers, nav items with icons, and expandable
//! module entries with sub-page children. Emits the selected page
//! ID when the user clicks a nav item.

use std::collections::HashMap;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_i18n::t;

/// A top-level nav entry. Items with children act as collapsible groups.
pub struct NavItem {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub icon: &'static str,
    pub children: Vec<NavChild>,
}

/// A sub-page entry nested under a NavItem.
pub struct NavChild {
    pub id: &'static str,
    pub i18n_key: &'static str,
}

/// A labeled group of nav items, e.g. "Appearance".
pub struct NavSection {
    pub i18n_key: &'static str,
    pub items: Vec<NavItem>,
}

/// Init data for the sidebar. All sections are passed up front.
pub struct SidebarInit {
    pub sections: Vec<NavSection>,
}

/// Sidebar component. Manages active highlight and expand/collapse state.
pub struct Sidebar {
    active_id: &'static str,
    expanded: Option<&'static str>,
    nav_buttons: HashMap<&'static str, gtk4::Button>,
    children_boxes: HashMap<&'static str, gtk4::Box>,
}

#[derive(Debug)]
pub enum SidebarMsg {
    Navigate(&'static str),
    ToggleExpand(&'static str),
}

#[derive(Debug)]
pub enum SidebarOutput {
    PageSelected(&'static str),
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
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut nav_buttons: HashMap<&'static str, gtk4::Button> = HashMap::new();
        let mut children_boxes: HashMap<&'static str, gtk4::Box> = HashMap::new();

        let widgets = view_output!();

        let default_active = "general";

        for section in &init.sections {
            let title = t(section.i18n_key);
            let section_title = gtk4::Label::builder()
                .label(&title)
                .halign(gtk4::Align::Start)
                .build();
            section_title.add_css_class("sidebar-section-title");

            let section_box = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Vertical)
                .build();
            section_box.add_css_class("sidebar-section");
            section_box.append(&section_title);

            for item in &section.items {
                let button = build_nav_item(item, &sender);
                nav_buttons.insert(item.id, button.clone());
                section_box.append(&button);

                if !item.children.is_empty() {
                    let cbox = build_children_box(item, &sender, &mut nav_buttons);
                    cbox.set_visible(false);
                    children_boxes.insert(item.id, cbox.clone());
                    section_box.append(&cbox);
                }
            }

            widgets.nav.append(&section_box);
        }

        if let Some(button) = nav_buttons.get(default_active) {
            button.add_css_class("active");
        }

        let model = Self {
            active_id: default_active,
            expanded: None,
            nav_buttons,
            children_boxes,
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

            SidebarMsg::ToggleExpand(id) => {
                let was_expanded = self.expanded == Some(id);
                self.collapse_expanded();

                if was_expanded {
                    return;
                }

                if let Some(cbox) = self.children_boxes.get(id) {
                    cbox.set_visible(true);
                }
                self.expanded = Some(id);
            }
        }
    }
}

impl Sidebar {
    fn collapse_expanded(&mut self) {
        let Some(prev_id) = self.expanded.take() else {
            return;
        };

        if let Some(cbox) = self.children_boxes.get(prev_id) {
            cbox.set_visible(false);
        }
    }
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

    if !item.children.is_empty() {
        let chevron = gtk4::Image::builder()
            .icon_name("ld-chevron-right-symbolic")
            .build();
        chevron.add_css_class("sidebar-item-chevron");
        content.append(&chevron);
    }

    let button = gtk4::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("sidebar-item");

    let item_id = item.id;
    let has_children = !item.children.is_empty();
    let sender = sender.clone();

    button.connect_clicked(move |_| {
        if has_children {
            sender.input(SidebarMsg::ToggleExpand(item_id));
        } else {
            sender.input(SidebarMsg::Navigate(item_id));
        }
    });

    button
}

fn build_children_box(
    item: &NavItem,
    sender: &ComponentSender<Sidebar>,
    nav_buttons: &mut HashMap<&'static str, gtk4::Button>,
) -> gtk4::Box {
    let children_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    children_box.add_css_class("sidebar-children");

    for child in &item.children {
        let dot = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        dot.add_css_class("sidebar-child-dot");
        dot.set_valign(gtk4::Align::Center);
        dot.set_vexpand(false);
        dot.set_hexpand(false);

        let label = gtk4::Label::builder()
            .label(t(child.i18n_key))
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .build();

        let content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .build();
        content.append(&dot);
        content.append(&label);

        let button = gtk4::Button::new();
        button.set_child(Some(&content));
        button.add_css_class("sidebar-child");

        nav_buttons.insert(child.id, button.clone());

        let child_id = child.id;
        let sender = sender.clone();
        button.connect_clicked(move |_| {
            sender.input(SidebarMsg::Navigate(child_id));
        });

        children_box.append(&button);
    }

    children_box
}
