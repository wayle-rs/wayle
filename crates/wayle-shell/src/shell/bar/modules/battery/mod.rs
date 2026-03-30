mod factory;
mod helpers;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{BatteryCmd, BatteryInit, BatteryMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct BatteryModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for BatteryModule {
    type Init = BatteryInit;
    type Input = BatteryMsg;
    type Output = ();
    type CommandOutput = BatteryCmd;

    view! {
        gtk::Box {
            add_css_class: "battery",

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
        let battery_config = &config.modules.battery;

        let initial_icon = battery_config
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
                    icon_color: battery_config.icon_color.clone(),
                    label_color: battery_config.label_color.clone(),
                    icon_background: battery_config.icon_bg_color.clone(),
                    button_background: battery_config.button_bg_color.clone(),
                    border_color: battery_config.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: battery_config.label_max_length.clone(),
                    show_icon: battery_config.icon_show.clone(),
                    show_label: battery_config.label_show.clone(),
                    show_border: battery_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => BatteryMsg::LeftClick,
                BarButtonOutput::RightClick => BatteryMsg::RightClick,
                BarButtonOutput::MiddleClick => BatteryMsg::MiddleClick,
                BarButtonOutput::ScrollUp => BatteryMsg::ScrollUp,
                BarButtonOutput::ScrollDown => BatteryMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, battery_config, &init.battery);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.battery;

        let action = match msg {
            BatteryMsg::LeftClick => config.left_click.get(),
            BatteryMsg::RightClick => config.right_click.get(),
            BatteryMsg::MiddleClick => config.middle_click.get(),
            BatteryMsg::ScrollUp => config.scroll_up.get(),
            BatteryMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: BatteryCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BatteryCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            BatteryCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            BatteryCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
