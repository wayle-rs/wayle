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
use wayle_widgets::WatcherToken;
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{MicrophoneCmd, MicrophoneInit, MicrophoneMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct MicrophoneModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    active_device_watcher_token: WatcherToken,
    audio: Arc<AudioService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for MicrophoneModule {
    type Init = MicrophoneInit;
    type Input = MicrophoneMsg;
    type Output = ();
    type CommandOutput = MicrophoneCmd;

    view! {
        gtk::Box {
            add_css_class: "microphone",

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
        let mic_config = &config.modules.microphone;

        let initial_icon = mic_config.icon_muted.get();

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: String::new(),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: mic_config.icon_color.clone(),
                    label_color: mic_config.label_color.clone(),
                    icon_background: mic_config.icon_bg_color.clone(),
                    button_background: mic_config.button_bg_color.clone(),
                    border_color: mic_config.border_color.clone(),
                    auto_icon_color: CssToken::Red,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: mic_config.label_max_length.clone(),
                    show_icon: mic_config.icon_show.clone(),
                    show_label: mic_config.label_show.clone(),
                    show_border: mic_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => MicrophoneMsg::LeftClick,
                BarButtonOutput::RightClick => MicrophoneMsg::RightClick,
                BarButtonOutput::MiddleClick => MicrophoneMsg::MiddleClick,
                BarButtonOutput::ScrollUp => MicrophoneMsg::ScrollUp,
                BarButtonOutput::ScrollDown => MicrophoneMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, mic_config, &init.audio);

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
        let config = &self.config.config().modules.microphone;

        let action = match msg {
            MicrophoneMsg::LeftClick => config.left_click.get(),
            MicrophoneMsg::RightClick => config.right_click.get(),
            MicrophoneMsg::MiddleClick => config.middle_click.get(),
            MicrophoneMsg::ScrollUp => config.scroll_up.get(),
            MicrophoneMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: MicrophoneCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let mic_config = &self.config.config().modules.microphone;

        match msg {
            MicrophoneCmd::DeviceChanged(device) => {
                if let Some(device) = device {
                    self.update_display(mic_config, &device);

                    let token = self.active_device_watcher_token.reset();
                    watchers::spawn_device_watchers(&sender, mic_config, &device, token);
                }
            }
            MicrophoneCmd::VolumeOrMuteChanged | MicrophoneCmd::IconConfigChanged => {
                if let Some(device) = self.audio.default_input.get() {
                    self.update_display(mic_config, &device);
                }
            }
            MicrophoneCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
