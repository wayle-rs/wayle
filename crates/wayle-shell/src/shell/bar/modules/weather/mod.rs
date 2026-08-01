mod factory;
pub(crate) mod helpers;
mod messages;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::{
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
    utils::force_window_resize,
};

pub(crate) use self::{
    factory::Factory,
    messages::{WeatherCmd, WeatherInit, WeatherMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct WeatherModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for WeatherModule {
    type Init = WeatherInit;
    type Input = WeatherMsg;
    type Output = ();
    type CommandOutput = WeatherCmd;

    view! {
        gtk::Box {
            add_css_class: "weather",

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
        let weather_config = &config.modules.weather;

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: weather_config.icon_name.get().clone(),
                label: String::from("--"),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: weather_config.icon_color.clone(),
                    label_color: weather_config.label_color.clone(),
                    icon_background: weather_config.icon_bg_color.clone(),
                    button_background: weather_config.button_bg_color.clone(),
                    border_color: weather_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: weather_config.label_max_length.clone(),
                    show_icon: weather_config.icon_show.clone(),
                    show_label: weather_config.label_show.clone(),
                    show_border: weather_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => WeatherMsg::LeftClick,
                BarButtonOutput::RightClick => WeatherMsg::RightClick,
                BarButtonOutput::MiddleClick => WeatherMsg::MiddleClick,
                BarButtonOutput::ScrollUp => WeatherMsg::ScrollUp,
                BarButtonOutput::ScrollDown => WeatherMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, weather_config, &init.weather);

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            weather_config.clone(),
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
        let weather = &self.config.config().modules.weather;

        let action = match msg {
            WeatherMsg::LeftClick => weather.left_click.get(),
            WeatherMsg::RightClick => weather.right_click.get(),
            WeatherMsg::MiddleClick => weather.middle_click.get(),
            WeatherMsg::ScrollUp => weather.scroll_up.get(),
            WeatherMsg::ScrollDown => weather.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(&mut self, msg: WeatherCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            WeatherCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
                force_window_resize(root);
            }
            WeatherCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
