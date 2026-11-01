//! Single monitor layout card. Shows monitor name, extends dropdown,
//! show/hide toggle, and three module zones (left, center, right).

mod methods;

use relm4::{
    factory::FactoryView,
    gtk,
    gtk::{glib, prelude::*},
    prelude::*,
};
use wayle_config::{
    ConfigProperty,
    schemas::{
        bar::{BarItem, BarLayout, BarModule},
        modules::CustomModuleDefinition,
    },
};
use wayle_i18n::t;

use super::{
    chip::Chip,
    zone::{DragPayload, DropLocation, ZoneId},
};

pub(super) struct LayoutCard {
    pub(super) monitor: String,
    pub(super) extends: Option<String>,
    pub(super) show: bool,
    pub(super) left: Vec<BarItem>,
    pub(super) center: Vec<BarItem>,
    pub(super) right: Vec<BarItem>,

    pub(super) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    pub(super) index: DynamicIndex,
    pub(super) monitor_entry: gtk::Entry,
    pub(super) extends_entry: gtk::Entry,
    pub(super) body: gtk::Box,
    pub(super) left_chips: Option<FactoryVecDeque<Chip>>,
    pub(super) center_chips: Option<FactoryVecDeque<Chip>>,
    pub(super) right_chips: Option<FactoryVecDeque<Chip>>,
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
    RemoveItem(ZoneId, DynamicIndex),
    RemoveGroupModule(ZoneId, DynamicIndex, usize),
    AddModuleToGroup(ZoneId, DynamicIndex, BarModule),
    GroupNameChanged(ZoneId, DynamicIndex, String),
}

#[derive(Debug)]
pub(super) enum LayoutCardOutput {
    Remove(DynamicIndex),
    Changed,
    ItemDropped(DragPayload, DropLocation),
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
            left_chips: None,
            center_chips: None,
            right_chips: None,
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
            LayoutCardMsg::GroupNameChanged(zone, chip_index, name) => {
                self.on_group_name_changed(zone, chip_index, name)
            }
            LayoutCardMsg::ShowToggled(active) => self.on_show_toggled(active, &sender),
            LayoutCardMsg::AddModule(zone, module) => self.on_add_module(zone, module),
            LayoutCardMsg::AddGroup(zone) => self.on_add_group(zone),
            LayoutCardMsg::RemoveItem(zone, chip_index) => self.on_remove_item(zone, chip_index),
            LayoutCardMsg::RemoveGroupModule(zone, chip_index, module_index) => {
                self.on_remove_group_module(zone, chip_index, module_index)
            }
            LayoutCardMsg::AddModuleToGroup(zone, chip_index, module) => {
                self.on_add_module_to_group(zone, chip_index, module)
            }
        };

        if changed {
            let _ = sender.output(LayoutCardOutput::Changed);
        }
    }
}
