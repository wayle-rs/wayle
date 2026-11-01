//! Factory item for one bar-layout chip (module or group).

mod helpers;
mod methods;

use relm4::{factory::FactoryView, gtk, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{
        bar::{BarItem, BarModule},
        modules::CustomModuleDefinition,
    },
};

use super::zone::ZoneId;

pub(super) struct Chip {
    pub(super) item: BarItem,
    pub(super) card_index: DynamicIndex,
    pub(super) self_index: DynamicIndex,
    pub(super) zone: ZoneId,
    pub(super) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    pub(super) root: gtk::Box,
}

pub(super) struct ChipInit {
    pub(super) item: BarItem,
    pub(super) card_index: DynamicIndex,
    pub(super) zone: ZoneId,
    pub(super) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
}

#[derive(Debug)]
pub(super) enum ChipMsg {
    Replace(BarItem),
}

#[derive(Debug)]
pub(super) enum ChipOutput {
    Remove(DynamicIndex),
    GroupNameChanged(DynamicIndex, String),
    AddModuleToGroup(DynamicIndex, BarModule),
    RemoveGroupModule(DynamicIndex, usize),
}

impl FactoryComponent for Chip {
    type Init = ChipInit;
    type Input = ChipMsg;
    type Output = ChipOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;
    type Root = gtk::Box;
    type Widgets = ();
    type Index = DynamicIndex;

    fn init_root(&self) -> Self::Root {
        self.root.clone()
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            item: init.item,
            card_index: init.card_index,
            self_index: index.clone(),
            zone: init.zone,
            custom_modules: init.custom_modules,
            root: gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .halign(gtk::Align::Start)
                .build(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        self.render(&root, &sender);
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            ChipMsg::Replace(item) => {
                self.item = item;
                let root = self.root.clone();
                self.render(&root, &sender);
            }
        }
    }
}
