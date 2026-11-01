mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_brightness::BrightnessService;
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
    messages::{BrightnessCmd, BrightnessInit, BrightnessMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct BrightnessModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    active_device_watcher_token: WatcherToken,
    brightness: Arc<BrightnessService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for BrightnessModule {
    type Init = BrightnessInit;
    type Input = BrightnessMsg;
    type Output = ();
    type CommandOutput = BrightnessCmd;

    view! {
        gtk::Box {
            add_css_class: "brightness",

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
        let brightness_config = &config.modules.brightness;

        let initial_icon = brightness_config
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
                    icon_color: brightness_config.icon_color.clone(),
                    label_color: brightness_config.label_color.clone(),
                    icon_background: brightness_config.icon_bg_color.clone(),
                    button_background: brightness_config.button_bg_color.clone(),
                    border_color: brightness_config.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: brightness_config.label_max_length.clone(),
                    show_icon: brightness_config.icon_show.clone(),
                    show_label: brightness_config.label_show.clone(),
                    show_border: brightness_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => BrightnessMsg::LeftClick,
                BarButtonOutput::RightClick => BrightnessMsg::RightClick,
                BarButtonOutput::MiddleClick => BrightnessMsg::MiddleClick,
                BarButtonOutput::ScrollUp => BrightnessMsg::ScrollUp,
                BarButtonOutput::ScrollDown => BrightnessMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, brightness_config, &init.brightness);

        let model = Self {
            bar_button,
            config: init.config,
            active_device_watcher_token: WatcherToken::new(),
            brightness: init.brightness,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let brightness_config = &self.config.config().modules.brightness;

        let action = match msg {
            BrightnessMsg::LeftClick => brightness_config.left_click.get(),
            BrightnessMsg::RightClick => brightness_config.right_click.get(),
            BrightnessMsg::MiddleClick => brightness_config.middle_click.get(),
            BrightnessMsg::ScrollUp => brightness_config.scroll_up.get(),
            BrightnessMsg::ScrollDown => brightness_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: BrightnessCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let brightness_config = &self.config.config().modules.brightness;

        match msg {
            BrightnessCmd::DeviceChanged(device) => {
                let Some(device) = device else {
                    return;
                };
                self.update_display(brightness_config, &device);
                self.apply_thresholds(brightness_config, &device);

                let token = self.active_device_watcher_token.reset();
                watchers::spawn_device_watchers(&sender, &device, token);
            }
            BrightnessCmd::BrightnessChanged | BrightnessCmd::ConfigChanged => {
                let Some(device) = self.brightness.primary.get() else {
                    return;
                };
                self.update_display(brightness_config, &device);
                self.apply_thresholds(brightness_config, &device);
            }
            BrightnessCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
