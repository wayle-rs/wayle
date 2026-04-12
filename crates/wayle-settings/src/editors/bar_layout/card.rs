//! Single monitor layout card. Shows monitor name, extends dropdown,
//! show/hide toggle, and three module zones (left, center, right).

use relm4::{
    factory::FactoryView,
    gtk,
    gtk::{glib, prelude::*},
    prelude::*,
};
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
    pub(super) show: bool,
    pub(super) left: Vec<BarItem>,
    pub(super) center: Vec<BarItem>,
    pub(super) right: Vec<BarItem>,

    custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    index: DynamicIndex,
    monitor_entry: gtk::Entry,
    extends_entry: gtk::Entry,
    body: gtk::Box,
    left_flow: gtk::Box,
    center_flow: gtk::Box,
    right_flow: gtk::Box,
}

pub(super) struct LayoutCardInit {
    pub(crate) layout: BarLayout,
    pub(crate) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
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
    pub(crate) fn to_layout(&self) -> BarLayout {
        BarLayout {
            monitor: self.monitor.clone(),
            extends: self.extends.clone(),
            show: self.show,
            left: self.left.clone(),
            center: self.center.clone(),
            right: self.right.clone(),
        }
    }

    pub(super) fn zone_mut(&mut self, zone: ZoneId) -> &mut Vec<BarItem> {
        match zone {
            ZoneId::Left => &mut self.left,
            ZoneId::Center => &mut self.center,
            ZoneId::Right => &mut self.right,
        }
    }

    fn zone_flow(&self, zone: ZoneId) -> &gtk::Box {
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
            let hidden_label = gtk::Label::new(Some(&t("settings-layout-hidden")));
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
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "layout-card",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Box {
                add_css_class: "layout-card-header",
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "layout-extends-label",
                    set_label: &t("settings-layout-monitor-label"),
                },

                #[name = "monitor_entry"]
                gtk::Entry {
                    add_css_class: "layout-monitor-input",
                    set_placeholder_text: Some("DP-1"),
                    set_hexpand: false,
                    connect_changed => LayoutCardMsg::MonitorChanged,
                },

                gtk::Label {
                    add_css_class: "layout-extends-label",
                    set_label: &t("settings-layout-extends-label"),
                },

                #[name = "extends_entry"]
                gtk::Entry {
                    add_css_class: "layout-extends-input",
                    set_placeholder_text: Some(&t("settings-layout-extends-none")),
                    set_hexpand: false,
                    connect_changed => LayoutCardMsg::ExtendsChanged,
                },

                #[name = "header_spacer"]
                gtk::Box {
                    set_hexpand: true,
                },

                #[name = "show_switch"]
                gtk::Switch {
                    add_css_class: "layout-show-toggle",
                    set_valign: gtk::Align::Center,
                    set_cursor_from_name: Some("pointer"),
                    connect_state_set[sender] => move |_switch, active| {
                        sender.input(LayoutCardMsg::ShowToggled(active));
                        glib::Propagation::Proceed
                    },
                },

                #[name = "delete_button"]
                gtk::Button {
                    add_css_class: "ghost-icon",
                    set_icon_name: "ld-trash-2-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender, index] => move |_button| {
                        let _ = sender.output(LayoutCardOutput::Remove(index.clone()));
                    },
                },
            },

            #[name = "body"]
            gtk::Box {
                add_css_class: "layout-card-body",
                set_orientation: gtk::Orientation::Vertical,
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
            monitor_entry: gtk::Entry::new(),
            extends_entry: gtk::Entry::new(),
            body: gtk::Box::new(gtk::Orientation::Vertical, 0),
            left_flow: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            center_flow: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            right_flow: gtk::Box::new(gtk::Orientation::Horizontal, 0),
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
        let changed = match msg {
            LayoutCardMsg::MonitorChanged => self.on_monitor_changed(),
            LayoutCardMsg::ExtendsChanged => self.on_extends_changed(),
            LayoutCardMsg::GroupNameChanged(zone, group_index, name) => {
                self.on_group_name_changed(zone, group_index, name)
            }
            LayoutCardMsg::ShowToggled(active) => self.on_show_toggled(active, &sender),
            LayoutCardMsg::AddModule(zone, module) => self.on_add_module(zone, module, &sender),
            LayoutCardMsg::AddGroup(zone) => self.on_add_group(zone, &sender),
            LayoutCardMsg::RemoveItem(zone, item_index) => {
                self.on_remove_item(zone, item_index, &sender)
            }
            LayoutCardMsg::RemoveGroupModule(zone, group_index, module_index) => {
                self.on_remove_group_module(zone, group_index, module_index, &sender)
            }
            LayoutCardMsg::AddModuleToGroup(zone, group_index, module) => {
                self.on_add_module_to_group(zone, group_index, module, &sender)
            }
        };

        if changed {
            let _ = sender.output(LayoutCardOutput::Changed);
        }
    }
}

impl LayoutCard {
    fn on_monitor_changed(&mut self) -> bool {
        self.monitor = self.monitor_entry.text().to_string();
        true
    }

    fn on_extends_changed(&mut self) -> bool {
        let text = self.extends_entry.text().to_string();
        self.extends = if text.is_empty() { None } else { Some(text) };
        true
    }

    fn on_group_name_changed(&mut self, zone: ZoneId, group_index: usize, name: String) -> bool {
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        group.name = name;
        true
    }

    fn on_show_toggled(&mut self, active: bool, sender: &FactorySender<Self>) -> bool {
        self.show = active;
        self.rebuild_body(sender);
        true
    }

    fn on_add_module(
        &mut self,
        zone: ZoneId,
        module: BarModule,
        sender: &FactorySender<Self>,
    ) -> bool {
        let item = BarItem::Module(ModuleRef::Plain(module));
        self.zone_mut(zone).push(item);
        self.rebuild_zone(zone, sender);
        true
    }

    fn on_add_group(&mut self, zone: ZoneId, sender: &FactorySender<Self>) -> bool {
        let group = BarGroup {
            name: t("settings-layout-default-group"),
            modules: Vec::new(),
        };
        self.zone_mut(zone).push(BarItem::Group(group));
        self.rebuild_zone(zone, sender);
        true
    }

    fn on_remove_item(
        &mut self,
        zone: ZoneId,
        item_index: usize,
        sender: &FactorySender<Self>,
    ) -> bool {
        let items = self.zone_mut(zone);
        if item_index >= items.len() {
            return false;
        }
        items.remove(item_index);
        self.rebuild_zone(zone, sender);
        true
    }

    fn on_remove_group_module(
        &mut self,
        zone: ZoneId,
        group_index: usize,
        module_index: usize,
        sender: &FactorySender<Self>,
    ) -> bool {
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        if module_index >= group.modules.len() {
            return false;
        }
        group.modules.remove(module_index);
        if group.modules.is_empty() {
            items.remove(group_index);
        }
        self.rebuild_zone(zone, sender);
        true
    }

    fn on_add_module_to_group(
        &mut self,
        zone: ZoneId,
        group_index: usize,
        module: BarModule,
        sender: &FactorySender<Self>,
    ) -> bool {
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        group.modules.push(ModuleRef::Plain(module));
        self.rebuild_zone(zone, sender);
        true
    }
}
