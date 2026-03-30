mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_audio::AudioService;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::{
    WatcherToken,
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
};

pub(crate) use self::{
    factory::Factory,
    messages::{VolumeCmd, VolumeInit, VolumeMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct VolumeModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    active_device_watcher_token: WatcherToken,
    audio: Arc<AudioService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for VolumeModule {
    type Init = VolumeInit;
    type Input = VolumeMsg;
    type Output = ();
    type CommandOutput = VolumeCmd;

    view! {
        gtk::Box {
            add_css_class: "volume",
            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let volume_config = &config.modules.volume;

        let initial_icon = volume_config
            .level_icons
            .get()
            .first()
            .cloned()
            .unwrap_or_default();

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: String::from("--%"),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: volume_config.icon_color.clone(),
                    label_color: volume_config.label_color.clone(),
                    icon_background: volume_config.icon_bg_color.clone(),
                    button_background: volume_config.button_bg_color.clone(),
                    border_color: volume_config.border_color.clone(),
                    auto_icon_color: CssToken::Red,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: volume_config.label_max_length.clone(),
                    show_icon: volume_config.icon_show.clone(),
                    show_label: volume_config.label_show.clone(),
                    show_border: volume_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => VolumeMsg::LeftClick,
                BarButtonOutput::RightClick => VolumeMsg::RightClick,
                BarButtonOutput::MiddleClick => VolumeMsg::MiddleClick,
                BarButtonOutput::ScrollUp => VolumeMsg::ScrollUp,
                BarButtonOutput::ScrollDown => VolumeMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, volume_config, &init.audio);

        let model = Self {
            bar_button,
            config: init.config,
            active_device_watcher_token: WatcherToken::new(),
            audio: init.audio,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let volume_config = &self.config.config().modules.volume;

        let action = match msg {
            VolumeMsg::LeftClick => volume_config.left_click.get(),
            VolumeMsg::RightClick => volume_config.right_click.get(),
            VolumeMsg::MiddleClick => volume_config.middle_click.get(),
            VolumeMsg::ScrollUp => volume_config.scroll_up.get(),
            VolumeMsg::ScrollDown => volume_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: VolumeCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        let volume_config = &self.config.config().modules.volume;

        match msg {
            VolumeCmd::DeviceChanged(device) => {
                if let Some(device) = device {
                    self.update_display(volume_config, &device);
                    self.apply_thresholds(volume_config, &device);

                    let token = self.active_device_watcher_token.reset();
                    watchers::spawn_device_watchers(&sender, &device, token);
                }
            }
            VolumeCmd::VolumeOrMuteChanged | VolumeCmd::ConfigChanged => {
                if let Some(device) = self.audio.default_output.get() {
                    self.update_display(volume_config, &device);
                    self.apply_thresholds(volume_config, &device);
                }
            }
            VolumeCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
