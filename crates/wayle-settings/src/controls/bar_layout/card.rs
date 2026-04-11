//! Single monitor layout card. Shows monitor name, extends dropdown,
//! show/hide toggle, and three module zones (left, center, right).

use gtk4::prelude::*;
use relm4::{factory::FactoryView, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{
        bar::{BarGroup, BarItem, BarLayout, BarModule, ModuleRef},
        modules::CustomModuleDefinition,
    },
};
use wayle_i18n::t;

use super::zone::{self, DragPayload, DropLocation, ZoneId};

pub(super) struct LayoutCard {
    monitor: String,
    extends: Option<String>,
    pub show: bool,
    pub left: Vec<BarItem>,
    pub center: Vec<BarItem>,
    pub right: Vec<BarItem>,

    custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    index: DynamicIndex,
    monitor_entry: gtk4::Entry,
    extends_entry: gtk4::Entry,
    body: gtk4::Box,
    left_flow: gtk4::Box,
    center_flow: gtk4::Box,
    right_flow: gtk4::Box,
}

pub(super) struct LayoutCardInit {
    pub layout: BarLayout,
    pub custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
}

#[derive(Debug)]
pub(super) enum LayoutCardMsg {
    MonitorChanged,
    ExtendsChanged,
    ShowToggled(bool),
    AddModule(ZoneId, BarModule),
    AddGroup(ZoneId),
    RemoveItem(ZoneId, usize),
    RemoveGroupModule(ZoneId, usize, usize),
    AddModuleToGroup(ZoneId, usize, BarModule),
    GroupNameChanged(ZoneId, usize, String),
}

#[derive(Debug)]
pub(super) enum LayoutCardOutput {
    Remove(DynamicIndex),
    Changed,
    ItemDropped(DragPayload, DropLocation),
}

impl LayoutCard {
    pub fn to_layout(&self) -> BarLayout {
        BarLayout {
            monitor: self.monitor.clone(),
            extends: self.extends.clone(),
            show: self.show,
            left: self.left.clone(),
            center: self.center.clone(),
            right: self.right.clone(),
        }
    }

    fn zone_mut(&mut self, zone: ZoneId) -> &mut Vec<BarItem> {
        match zone {
            ZoneId::Left => &mut self.left,
            ZoneId::Center => &mut self.center,
            ZoneId::Right => &mut self.right,
        }
    }

    fn zone_flow(&self, zone: ZoneId) -> &gtk4::Box {
        match zone {
            ZoneId::Left => &self.left_flow,
            ZoneId::Center => &self.center_flow,
            ZoneId::Right => &self.right_flow,
        }
    }

    fn zone_items(&self, zone: ZoneId) -> &[BarItem] {
        match zone {
            ZoneId::Left => &self.left,
            ZoneId::Center => &self.center,
            ZoneId::Right => &self.right,
        }
    }

    fn rebuild_zone(&self, zone: ZoneId, sender: &FactorySender<Self>) {
        zone::rebuild_zone_chips(
            self.zone_flow(zone),
            self.zone_items(zone),
            self.index.current_index(),
            zone,
            &self.custom_modules,
            sender,
        );
    }

    fn rebuild_body(&mut self, sender: &FactorySender<Self>) {
        while let Some(child) = self.body.first_child() {
            self.body.remove(&child);
        }

        if !self.show {
            let hidden_label = gtk4::Label::new(Some(&t("settings-layout-hidden")));
            hidden_label.add_css_class("layout-hidden-label");
            self.body.append(&hidden_label);
            return;
        }

        let card_index = self.index.current_index();

        let (left_row, left_flow) = zone::build_zone_row(
            ZoneId::Left,
            &self.left,
            card_index,
            &self.custom_modules,
            sender,
        );
        let (center_row, center_flow) = zone::build_zone_row(
            ZoneId::Center,
            &self.center,
            card_index,
            &self.custom_modules,
            sender,
        );
        let (right_row, right_flow) = zone::build_zone_row(
            ZoneId::Right,
            &self.right,
            card_index,
            &self.custom_modules,
            sender,
        );

        self.left_flow = left_flow;
        self.center_flow = center_flow;
        self.right_flow = right_flow;

        self.body.append(&left_row);
        self.body.append(&center_row);
        self.body.append(&right_row);
    }
}

#[relm4::factory(pub(super))]
impl FactoryComponent for LayoutCard {
    type Init = LayoutCardInit;
    type Input = LayoutCardMsg;
    type Output = LayoutCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk4::Box;

    view! {
        #[root]
        gtk4::Box {
            add_css_class: "layout-card",
            set_orientation: gtk4::Orientation::Vertical,

            #[name = "header"]
            gtk4::Box {
                add_css_class: "layout-card-header",
                set_orientation: gtk4::Orientation::Horizontal,

                gtk4::Label {
                    add_css_class: "layout-extends-label",
                    set_label: &t("settings-layout-monitor-label"),
                },

                #[name = "monitor_entry"]
                gtk4::Entry {
                    add_css_class: "layout-monitor-input",
                    set_placeholder_text: Some("DP-1"),
                    set_hexpand: false,
                    connect_changed => LayoutCardMsg::MonitorChanged,
                },

                gtk4::Label {
                    add_css_class: "layout-extends-label",
                    set_label: &t("settings-layout-extends-label"),
                },

                #[name = "extends_entry"]
                gtk4::Entry {
                    add_css_class: "layout-extends-input",
                    set_placeholder_text: Some(&t("settings-layout-extends-none")),
                    set_hexpand: false,
                    connect_changed => LayoutCardMsg::ExtendsChanged,
                },

                #[name = "header_spacer"]
                gtk4::Box {
                    set_hexpand: true,
                },

                #[name = "show_switch"]
                gtk4::Switch {
                    add_css_class: "layout-show-toggle",
                    set_valign: gtk4::Align::Center,
                    set_cursor_from_name: Some("pointer"),
                    connect_state_set[sender] => move |_switch, active| {
                        sender.input(LayoutCardMsg::ShowToggled(active));
                        gtk4::glib::Propagation::Proceed
                    },
                },

                #[name = "delete_button"]
                gtk4::Button {
                    add_css_class: "ghost-icon",
                    set_icon_name: "ld-trash-2-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    set_valign: gtk4::Align::Center,
                    connect_clicked[sender, index] => move |_button| {
                        let _ = sender.output(LayoutCardOutput::Remove(index.clone()));
                    },
                },
            },

            #[name = "body"]
            gtk4::Box {
                add_css_class: "layout-card-body",
                set_orientation: gtk4::Orientation::Vertical,
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            monitor: init.layout.monitor,
            extends: init.layout.extends,
            show: init.layout.show,
            left: init.layout.left,
            center: init.layout.center,
            right: init.layout.right,

            custom_modules: init.custom_modules,
            index: index.clone(),
            monitor_entry: gtk4::Entry::new(),
            extends_entry: gtk4::Entry::new(),
            body: gtk4::Box::new(gtk4::Orientation::Vertical, 0),
            left_flow: gtk4::Box::new(gtk4::Orientation::Horizontal, 0),
            center_flow: gtk4::Box::new(gtk4::Orientation::Horizontal, 0),
            right_flow: gtk4::Box::new(gtk4::Orientation::Horizontal, 0),
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets.monitor_entry.set_text(&self.monitor);
        widgets
            .extends_entry
            .set_text(self.extends.as_deref().unwrap_or(""));
        widgets.show_switch.set_active(self.show);

        self.monitor_entry = widgets.monitor_entry.clone();
        self.extends_entry = widgets.extends_entry.clone();
        self.body = widgets.body.clone();

        self.rebuild_body(&sender);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            LayoutCardMsg::MonitorChanged => {
                self.monitor = self.monitor_entry.text().to_string();
                let _ = sender.output(LayoutCardOutput::Changed);
            }

            LayoutCardMsg::ExtendsChanged => {
                let text = self.extends_entry.text().to_string();
                self.extends = if text.is_empty() { None } else { Some(text) };
                let _ = sender.output(LayoutCardOutput::Changed);
            }

            LayoutCardMsg::GroupNameChanged(zone, group_index, name) => {
                let items = self.zone_mut(zone);

                if let Some(BarItem::Group(group)) = items.get_mut(group_index) {
                    group.name = name;
                    let _ = sender.output(LayoutCardOutput::Changed);
                }
            }

            LayoutCardMsg::ShowToggled(active) => {
                self.show = active;
                self.rebuild_body(&sender);
                let _ = sender.output(LayoutCardOutput::Changed);
            }

            LayoutCardMsg::AddModule(zone, module) => {
                let item = BarItem::Module(ModuleRef::Plain(module));
                self.zone_mut(zone).push(item);
                self.rebuild_zone(zone, &sender);
                let _ = sender.output(LayoutCardOutput::Changed);
            }

            LayoutCardMsg::AddGroup(zone) => {
                let group = BarGroup {
                    name: t("settings-layout-default-group"),
                    modules: Vec::new(),
                };
                self.zone_mut(zone).push(BarItem::Group(group));
                self.rebuild_zone(zone, &sender);
                let _ = sender.output(LayoutCardOutput::Changed);
            }

            LayoutCardMsg::RemoveItem(zone, item_index) => {
                let items = self.zone_mut(zone);

                if item_index < items.len() {
                    items.remove(item_index);
                    self.rebuild_zone(zone, &sender);
                    let _ = sender.output(LayoutCardOutput::Changed);
                }
            }

            LayoutCardMsg::RemoveGroupModule(zone, group_index, module_index) => {
                let items = self.zone_mut(zone);

                if let Some(BarItem::Group(group)) = items.get_mut(group_index)
                    && module_index < group.modules.len()
                {
                    group.modules.remove(module_index);

                    if group.modules.is_empty() {
                        items.remove(group_index);
                    }

                    self.rebuild_zone(zone, &sender);
                    let _ = sender.output(LayoutCardOutput::Changed);
                }
            }

            LayoutCardMsg::AddModuleToGroup(zone, group_index, module) => {
                let items = self.zone_mut(zone);

                if let Some(BarItem::Group(group)) = items.get_mut(group_index) {
                    group.modules.push(ModuleRef::Plain(module));
                    self.rebuild_zone(zone, &sender);
                    let _ = sender.output(LayoutCardOutput::Changed);
                }
            }
        }
    }
}
