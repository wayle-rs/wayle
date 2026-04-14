//! Behavior methods for `LayoutCard`: zone accessors, chip factory
//! management, and message handlers.

use relm4::{factory::FactorySender, gtk, gtk::prelude::*, prelude::*};
use wayle_config::schemas::bar::{BarGroup, BarItem, BarLayout, BarModule, ModuleRef};
use wayle_i18n::t;

use super::{
    super::{
        chip::{Chip, ChipInit, ChipMsg, ChipOutput},
        zone::{self, ZoneId},
    },
    LayoutCard, LayoutCardMsg,
};

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

    pub(crate) fn zone_mut(&mut self, zone: ZoneId) -> &mut Vec<BarItem> {
        match zone {
            ZoneId::Left => &mut self.left,
            ZoneId::Center => &mut self.center,
            ZoneId::Right => &mut self.right,
        }
    }

    pub(super) fn chips_mut(&mut self, zone: ZoneId) -> Option<&mut FactoryVecDeque<Chip>> {
        match zone {
            ZoneId::Left => self.left_chips.as_mut(),
            ZoneId::Center => self.center_chips.as_mut(),
            ZoneId::Right => self.right_chips.as_mut(),
        }
    }

    fn zone_items(&self, zone: ZoneId) -> &[BarItem] {
        match zone {
            ZoneId::Left => &self.left,
            ZoneId::Center => &self.center,
            ZoneId::Right => &self.right,
        }
    }

    pub(super) fn rebuild_body(&mut self, sender: &FactorySender<Self>) {
        while let Some(child) = self.body.first_child() {
            self.body.remove(&child);
        }

        self.left_chips = None;
        self.center_chips = None;
        self.right_chips = None;

        if !self.show {
            let hidden_label = gtk::Label::new(Some(&t("settings-layout-hidden")));
            hidden_label.add_css_class("layout-hidden-label");
            self.body.append(&hidden_label);
            return;
        }

        let card_index = self.index.current_index();

        let (left_row, left_flow) =
            zone::build_zone_row(ZoneId::Left, card_index, &self.custom_modules, sender);
        let (center_row, center_flow) =
            zone::build_zone_row(ZoneId::Center, card_index, &self.custom_modules, sender);
        let (right_row, right_flow) =
            zone::build_zone_row(ZoneId::Right, card_index, &self.custom_modules, sender);

        self.body.append(&left_row);
        self.body.append(&center_row);
        self.body.append(&right_row);

        self.left_chips = Some(self.attach_chips(ZoneId::Left, left_flow, sender));
        self.center_chips = Some(self.attach_chips(ZoneId::Center, center_flow, sender));
        self.right_chips = Some(self.attach_chips(ZoneId::Right, right_flow, sender));
    }

    fn attach_chips(
        &self,
        zone: ZoneId,
        flow: gtk::FlowBox,
        sender: &FactorySender<Self>,
    ) -> FactoryVecDeque<Chip> {
        let mut chips =
            FactoryVecDeque::builder()
                .launch(flow)
                .forward(sender.input_sender(), move |output| match output {
                    ChipOutput::Remove(index) => LayoutCardMsg::RemoveItem(zone, index),
                    ChipOutput::GroupNameChanged(index, name) => {
                        LayoutCardMsg::GroupNameChanged(zone, index, name)
                    }
                    ChipOutput::AddModuleToGroup(index, module) => {
                        LayoutCardMsg::AddModuleToGroup(zone, index, module)
                    }
                    ChipOutput::RemoveGroupModule(index, mod_index) => {
                        LayoutCardMsg::RemoveGroupModule(zone, index, mod_index)
                    }
                });

        {
            let mut guard = chips.guard();
            for item in self.zone_items(zone).iter().cloned() {
                guard.push_back(ChipInit {
                    item,
                    card_index: self.index.clone(),
                    zone,
                    custom_modules: self.custom_modules.clone(),
                });
            }
        }

        chips
    }

    pub(super) fn on_monitor_changed(&mut self) -> bool {
        self.monitor = self.monitor_entry.text().to_string();
        true
    }

    pub(super) fn on_extends_changed(&mut self) -> bool {
        let text = self.extends_entry.text().to_string();
        self.extends = if text.is_empty() { None } else { Some(text) };
        true
    }

    pub(super) fn on_group_name_changed(
        &mut self,
        zone: ZoneId,
        chip_index: DynamicIndex,
        name: String,
    ) -> bool {
        let group_index = chip_index.current_index();
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        group.name = name;
        true
    }

    pub(super) fn on_show_toggled(&mut self, active: bool, sender: &FactorySender<Self>) -> bool {
        self.show = active;
        self.rebuild_body(sender);
        true
    }

    pub(super) fn on_add_module(&mut self, zone: ZoneId, module: BarModule) -> bool {
        let item = BarItem::Module(ModuleRef::Plain(module));
        self.zone_mut(zone).push(item.clone());
        self.push_chip(zone, item);
        true
    }

    pub(super) fn on_add_group(&mut self, zone: ZoneId) -> bool {
        let group = BarGroup {
            name: t("settings-layout-default-group"),
            modules: Vec::new(),
        };
        let item = BarItem::Group(group);
        self.zone_mut(zone).push(item.clone());
        self.push_chip(zone, item);
        true
    }

    pub(super) fn on_remove_item(&mut self, zone: ZoneId, chip_index: DynamicIndex) -> bool {
        let item_index = chip_index.current_index();
        let items = self.zone_mut(zone);
        if item_index >= items.len() {
            return false;
        }
        items.remove(item_index);
        self.remove_chip(zone, item_index);
        true
    }

    pub(super) fn on_remove_group_module(
        &mut self,
        zone: ZoneId,
        chip_index: DynamicIndex,
        module_index: usize,
    ) -> bool {
        let group_index = chip_index.current_index();
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        if module_index >= group.modules.len() {
            return false;
        }
        group.modules.remove(module_index);

        if !group.modules.is_empty() {
            let updated = items[group_index].clone();
            self.replace_chip(zone, group_index, updated);
            return true;
        }

        items.remove(group_index);
        self.remove_chip(zone, group_index);
        true
    }

    pub(super) fn on_add_module_to_group(
        &mut self,
        zone: ZoneId,
        chip_index: DynamicIndex,
        module: BarModule,
    ) -> bool {
        let group_index = chip_index.current_index();
        let items = self.zone_mut(zone);
        let Some(BarItem::Group(group)) = items.get_mut(group_index) else {
            return false;
        };
        group.modules.push(ModuleRef::Plain(module));
        let updated = items[group_index].clone();
        self.replace_chip(zone, group_index, updated);
        true
    }

    fn push_chip(&mut self, zone: ZoneId, item: BarItem) {
        let init = ChipInit {
            item,
            card_index: self.index.clone(),
            zone,
            custom_modules: self.custom_modules.clone(),
        };
        let Some(chips) = self.chips_mut(zone) else {
            return;
        };
        chips.guard().push_back(init);
    }

    fn remove_chip(&mut self, zone: ZoneId, index: usize) {
        let Some(chips) = self.chips_mut(zone) else {
            return;
        };
        chips.guard().remove(index);
    }

    fn replace_chip(&mut self, zone: ZoneId, index: usize, item: BarItem) {
        let Some(chips) = self.chips_mut(zone) else {
            return;
        };
        chips.send(index, ChipMsg::Replace(item));
    }
}
