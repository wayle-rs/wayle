//! Per-monitor wallpaper editor. Each monitor gets a card with name,
//! wallpaper file picker, and fit mode dropdown. Add/remove monitors.

mod card;

use card::{MonitorCard, MonitorCardOutput};
use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::wallpaper::{FitMode, MonitorWallpaperConfig},
};
use wayle_i18n::t;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct MonitorWallpaperControl {
    property: ConfigProperty<Vec<MonitorWallpaperConfig>>,
    cards: FactoryVecDeque<MonitorCard>,
}

#[derive(Debug)]
pub(crate) enum MonitorWallpaperMsg {
    Add,
    Remove(DynamicIndex),
    CardChanged,
    Refresh,
}

impl SimpleComponent for MonitorWallpaperControl {
    type Init = ConfigProperty<Vec<MonitorWallpaperConfig>>;
    type Input = MonitorWallpaperMsg;
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
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("monitor-wallpaper-control");

        let card_list = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        card_list.add_css_class("monitor-wallpaper-list");

        let mut cards = FactoryVecDeque::builder()
            .launch(card_list.clone())
            .forward(sender.input_sender(), |output| match output {
                MonitorCardOutput::Remove(index) => MonitorWallpaperMsg::Remove(index),
                MonitorCardOutput::Changed => MonitorWallpaperMsg::CardChanged,
            });

        {
            let mut guard = cards.guard();
            for config in property.get() {
                guard.push_back(config);
            }
        }

        let add_icon = gtk4::Image::from_icon_name("ld-plus-symbolic");
        let add_label = gtk4::Label::new(Some(&t("settings-monitor-add")));
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
            let _ = input_sender.send(MonitorWallpaperMsg::Add);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            let _ = input_sender.send(MonitorWallpaperMsg::Refresh);
        });

        root.append(&card_list);
        root.append(&add_button);

        let model = Self { property, cards };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            MonitorWallpaperMsg::Add => {
                let new_config = MonitorWallpaperConfig {
                    name: String::new(),
                    fit_mode: FitMode::Fill,
                    wallpaper: String::new(),
                };

                self.cards.guard().push_back(new_config);
                self.commit(&sender);
            }

            MonitorWallpaperMsg::Remove(index) => {
                self.cards.guard().remove(index.current_index());
                self.commit(&sender);
            }

            MonitorWallpaperMsg::CardChanged => {
                self.commit(&sender);
            }

            MonitorWallpaperMsg::Refresh => {
                let incoming = self.property.get();

                let current: Vec<MonitorWallpaperConfig> =
                    self.cards.iter().map(|card| card.to_config()).collect();

                if incoming == current {
                    return;
                }

                let mut guard = self.cards.guard();
                guard.clear();

                for config in incoming {
                    guard.push_back(config);
                }
            }
        }
    }
}

impl MonitorWallpaperControl {
    fn commit(&self, sender: &ComponentSender<Self>) {
        let configs: Vec<MonitorWallpaperConfig> =
            self.cards.iter().map(|card| card.to_config()).collect();

        self.property.set(configs);
        let _ = sender.output(ControlOutput::ValueChanged);
    }
}
