mod factory;
mod messages;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
    ColorValue,
};

pub(crate) use self::{
    factory::Factory,
    messages::{PowerCmd, PowerInit, PowerMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct PowerModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for PowerModule {
    type Init = PowerInit;
    type Input = PowerMsg;
    type Output = ();
    type CommandOutput = PowerCmd;

    view! {
        gtk::Box {
            add_css_class: "power",

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
        let power = &config.modules.power;

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: power.icon_name.get(),
                label: String::new(),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: power.icon_color.clone(),
                    label_color: ConfigProperty::new(ColorValue::Token(CssToken::FgDefault)),
                    icon_background: power.icon_bg_color.clone(),
                    button_background: ConfigProperty::new(ColorValue::Token(
                        CssToken::BgSurfaceElevated,
                    )),
                    border_color: power.border_color.clone(),
                    auto_icon_color: CssToken::Red,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: ConfigProperty::new(0),
                    show_icon: ConfigProperty::new(true),
                    show_label: ConfigProperty::new(false),
                    show_border: power.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => PowerMsg::LeftClick,
                BarButtonOutput::RightClick => PowerMsg::RightClick,
                BarButtonOutput::MiddleClick => PowerMsg::MiddleClick,
                BarButtonOutput::ScrollUp => PowerMsg::ScrollUp,
                BarButtonOutput::ScrollDown => PowerMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, power);

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            power.clone(),
        );

        let model = Self {
            bar_button,
            config: init.config,
            opener,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let power = &self.config.config().modules.power;

        let action = match msg {
            PowerMsg::LeftClick => power.left_click.get(),
            PowerMsg::RightClick => power.right_click.get(),
            PowerMsg::MiddleClick => power.middle_click.get(),
            PowerMsg::ScrollUp => power.scroll_up.get(),
            PowerMsg::ScrollDown => power.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(&mut self, msg: PowerCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            PowerCmd::IconConfigChanged => {
                let power = &self.config.config().modules.power;
                self.bar_button
                    .emit(BarButtonInput::SetIcon(power.icon_name.get()));
            }
        }
    }
}
