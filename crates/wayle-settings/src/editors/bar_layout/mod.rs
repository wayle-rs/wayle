//! Visual editor for bar module layout. Per-monitor cards with draggable
//! module chips in left/center/right zones.

mod card;
mod chip;
mod methods;
mod module_picker;
mod row;
pub(super) mod zone;

use card::{LayoutCard, LayoutCardInit, LayoutCardOutput};
use relm4::{gtk, gtk::prelude::*, prelude::*};
pub(crate) use row::bar_layout;
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarLayout, modules::CustomModuleDefinition},
};
use wayle_i18n::t;
use zone::{DragPayload, DropLocation};

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct BarLayoutInit {
    pub(crate) property: ConfigProperty<Vec<BarLayout>>,
    pub(crate) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
}

pub(crate) struct BarLayoutControl {
    pub(super) property: ConfigProperty<Vec<BarLayout>>,
    pub(super) custom_modules: ConfigProperty<Vec<CustomModuleDefinition>>,
    cards: FactoryVecDeque<LayoutCard>,
    _watcher: WatcherHandle,
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
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("bar-layout-control");

        let card_list = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
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

        let add_icon = gtk::Image::from_icon_name("ld-plus-symbolic");
        let add_label = gtk::Label::new(Some(&t("settings-layout-add")));
        let add_content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        add_content.append(&add_icon);
        add_content.append(&add_label);

        let add_button = gtk::Button::builder()
            .child(&add_content)
            .halign(gtk::Align::Start)
            .build();

        add_button.add_css_class("ghost");
        add_button.set_cursor_from_name(Some("pointer"));

        let input_sender = sender.input_sender().clone();
        add_button.connect_clicked(move |_button| {
            let _ = input_sender.send(BarLayoutMsg::Add);
        });

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&init.property, move || {
            input_sender.send(BarLayoutMsg::Refresh).is_ok()
        });

        root.append(&card_list);
        root.append(&add_button);

        let model = Self {
            property: init.property,
            custom_modules: init.custom_modules,
            cards,
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BarLayoutMsg::Add => self.on_add(),
            BarLayoutMsg::Remove(index) => self.on_remove(index),
            BarLayoutMsg::CardChanged => self.commit(),
            BarLayoutMsg::ItemDropped(from, to) => self.on_item_dropped(from, to),
            BarLayoutMsg::Refresh => self.on_refresh(),
        }
    }
}
