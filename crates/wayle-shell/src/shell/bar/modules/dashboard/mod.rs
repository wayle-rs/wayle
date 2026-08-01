mod factory;
mod helpers;
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

use self::helpers::build_icon;
pub(crate) use self::{
    factory::Factory,
    messages::{DashboardCmd, DashboardInit, DashboardMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct DashboardModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for DashboardModule {
    type Init = DashboardInit;
    type Input = DashboardMsg;
    type Output = ();
    type CommandOutput = DashboardCmd;

    view! {
        gtk::Box {
            add_css_class: "dashboard",

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
        let dashboard = &config.modules.dashboard;

        let icon = build_icon(&dashboard.icon_override.get());

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon,
                label: String::new(),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: dashboard.icon_color.clone(),
                    label_color: ConfigProperty::new(ColorValue::Token(CssToken::FgDefault)),
                    icon_background: dashboard.icon_bg_color.clone(),
                    button_background: ConfigProperty::new(ColorValue::Token(
                        CssToken::BgSurfaceElevated,
                    )),
                    border_color: dashboard.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: ConfigProperty::new(0),
                    show_icon: ConfigProperty::new(true),
                    show_label: ConfigProperty::new(false),
                    show_border: dashboard.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => DashboardMsg::LeftClick,
                BarButtonOutput::RightClick => DashboardMsg::RightClick,
                BarButtonOutput::MiddleClick => DashboardMsg::MiddleClick,
                BarButtonOutput::ScrollUp => DashboardMsg::ScrollUp,
                BarButtonOutput::ScrollDown => DashboardMsg::ScrollDown,
            });

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            dashboard.clone(),
        );

        watchers::spawn_watchers(&sender, dashboard);

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
        let dashboard = &self.config.config().modules.dashboard;

        let action = match msg {
            DashboardMsg::LeftClick => dashboard.left_click.get(),
            DashboardMsg::RightClick => dashboard.right_click.get(),
            DashboardMsg::MiddleClick => dashboard.middle_click.get(),
            DashboardMsg::ScrollUp => dashboard.scroll_up.get(),
            DashboardMsg::ScrollDown => dashboard.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(
        &mut self,
        msg: DashboardCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            DashboardCmd::IconConfigChanged => {
                let dashboard = &self.config.config().modules.dashboard;
                let icon = build_icon(&dashboard.icon_override.get());
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
