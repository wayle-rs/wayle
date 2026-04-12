//! Visual editor for bar module layout. Per-monitor cards with draggable
//! module chips in left/center/right zones.

mod card;
mod module_picker;
mod row;
pub(crate) use row::*;

pub(super) mod zone;

use card::{LayoutCard, LayoutCardInit, LayoutCardOutput};
use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarLayout, modules::CustomModuleDefinition},
};
use wayle_i18n::t;
use zone::{DragPayload, DropLocation};

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct BarLayoutInit {
    pub property: ConfigProperty<Vec<BarLayout>>,
    pub custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
}

pub(crate) struct BarLayoutControl {
    property: ConfigProperty<Vec<BarLayout>>,
    custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    cards: FactoryVecDeque<LayoutCard>,
}

#[derive(Debug)]
pub(crate) enum BarLayoutMsg {
    Add,
    Remove(DynamicIndex),
    CardChanged,
    ItemDropped(DragPayload, DropLocation),
    Refresh,
}

impl SimpleComponent for BarLayoutControl {
    type Init = BarLayoutInit;
    type Input = BarLayoutMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .hexpand(true)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("bar-layout-control");

        let card_list = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        card_list.add_css_class("bar-layout-list");

        let mut cards = FactoryVecDeque::builder()
            .launch(card_list.clone())
            .forward(sender.input_sender(), |output| match output {
                LayoutCardOutput::Remove(index) => BarLayoutMsg::Remove(index),
                LayoutCardOutput::Changed => BarLayoutMsg::CardChanged,
                LayoutCardOutput::ItemDropped(from, to) => BarLayoutMsg::ItemDropped(from, to),
            });

        {
            let mut guard = cards.guard();
            for layout in init.property.get() {
                guard.push_back(LayoutCardInit {
                    layout,
                    custom_modules: init.custom_modules.clone(),
                });
            }
        }

        let add_icon = gtk4::Image::from_icon_name("ld-plus-symbolic");
        let add_label = gtk4::Label::new(Some(&t("settings-layout-add")));
        let add_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        add_content.append(&add_icon);
        add_content.append(&add_label);

        let add_button = gtk4::Button::builder()
            .child(&add_content)
            .halign(gtk4::Align::Start)
            .build();

        add_button.add_css_class("ghost");
        add_button.set_cursor_from_name(Some("pointer"));

        let input_sender = sender.input_sender().clone();
        add_button.connect_clicked(move |_button| {
            let _ = input_sender.send(BarLayoutMsg::Add);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            let _ = input_sender.send(BarLayoutMsg::Refresh);
        });

        root.append(&card_list);
        root.append(&add_button);

        let model = Self {
            property: init.property,
            custom_modules: init.custom_modules,
            cards,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BarLayoutMsg::Add => {
                self.cards.guard().push_back(LayoutCardInit {
                    layout: BarLayout::default(),
                    custom_modules: self.custom_modules.clone(),
                });
                self.commit(&sender);
            }

            BarLayoutMsg::Remove(index) => {
                self.cards.guard().remove(index.current_index());
                self.commit(&sender);
            }

            BarLayoutMsg::CardChanged => {
                self.commit(&sender);
            }

            BarLayoutMsg::ItemDropped(from, to) => {
                self.handle_drop(from, to);
                self.commit(&sender);
                self.rebuild_cards();
            }

            BarLayoutMsg::Refresh => {
                let incoming = self.property.get();

                let current: Vec<BarLayout> =
                    self.cards.iter().map(|card| card.to_layout()).collect();

                if incoming == current {
                    return;
                }

                self.rebuild_cards();
            }
        }
    }
}

impl BarLayoutControl {
    fn commit(&self, sender: &ComponentSender<Self>) {
        let layouts: Vec<BarLayout> = self.cards.iter().map(|card| card.to_layout()).collect();

        self.property.set(layouts);
        let _ = sender.output(ControlOutput::ValueChanged);
    }

    fn handle_drop(&mut self, from: DragPayload, to: DropLocation) {
        let mut guard = self.cards.guard();
        let same_card = from.card_index == to.card_index;

        if guard.get(from.card_index).is_none() {
            return;
        }

        if !same_card && guard.get(to.card_index).is_none() {
            return;
        }

        let source_card = &mut guard[from.card_index];
        let source_zone = source_card.zone_mut(from.zone);

        if from.item_index >= source_zone.len() {
            return;
        }

        let item = source_zone.remove(from.item_index);

        if same_card {
            let same_zone = from.zone == to.zone;
            let mut target_pos = to.position;

            if same_zone && from.item_index < target_pos {
                target_pos = target_pos.saturating_sub(1);
            }

            let target_zone = source_card.zone_mut(to.zone);
            let position = target_pos.min(target_zone.len());
            target_zone.insert(position, item);
        } else {
            let target_card = &mut guard[to.card_index];
            let target_zone = target_card.zone_mut(to.zone);
            let position = to.position.min(target_zone.len());
            target_zone.insert(position, item);
        }
    }

    fn rebuild_cards(&mut self) {
        let layouts = self.property.get();
        let mut guard = self.cards.guard();
        guard.clear();

        for layout in layouts {
            let layout = LayoutCardInit {
                layout,
                custom_modules: self.custom_modules.clone(),
            };
            guard.push_back(layout);
        }
    }
}
