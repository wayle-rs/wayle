mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_idle_inhibit::IdleInhibitor;
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{IdleInhibitCmd, IdleInhibitInit, IdleInhibitMsg},
};
use crate::{
    services::idle_inhibit::IdleInhibitState,
    shell::bar::dropdowns::DropdownOpener,
};

pub(crate) struct IdleInhibitModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    state: IdleInhibitState,
    inhibitor: Option<IdleInhibitor>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for IdleInhibitModule {
    type Init = IdleInhibitInit;
    type Input = IdleInhibitMsg;
    type Output = ();
    type CommandOutput = IdleInhibitCmd;

    view! {
        gtk::Box {
            add_css_class: "idle-inhibit",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = &init.config.config().modules.idle_inhibit;

        let state = init.idle_inhibit.state();

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: config.icon_inactive.get().clone(),
                label: String::new(),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: config.icon_color.clone(),
                    label_color: config.label_color.clone(),
                    icon_background: config.icon_bg_color.clone(),
                    button_background: config.button_bg_color.clone(),
                    border_color: config.border_color.clone(),
                    auto_icon_color: CssToken::Green,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: config.label_max_length.clone(),
                    show_icon: config.icon_show.clone(),
                    show_label: config.label_show.clone(),
                    show_border: config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => IdleInhibitMsg::LeftClick,
                BarButtonOutput::RightClick => IdleInhibitMsg::RightClick,
                BarButtonOutput::MiddleClick => IdleInhibitMsg::MiddleClick,
                BarButtonOutput::ScrollUp => IdleInhibitMsg::ScrollUp,
                BarButtonOutput::ScrollDown => IdleInhibitMsg::ScrollDown,
            });

        watchers::spawn_config_watchers(&sender, config);
        watchers::spawn_state_watchers(&sender, &state);

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            config.clone(),
        );

        let model = Self {
            bar_button,
            config: init.config,
            state,
            inhibitor: None,
            opener,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.idle_inhibit;

        let action = match msg {
            IdleInhibitMsg::LeftClick => config.left_click.get(),
            IdleInhibitMsg::RightClick => config.right_click.get(),
            IdleInhibitMsg::MiddleClick => config.middle_click.get(),
            IdleInhibitMsg::ScrollUp => config.scroll_up.get(),
            IdleInhibitMsg::ScrollDown => config.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(
        &mut self,
        msg: IdleInhibitCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            IdleInhibitCmd::ConfigChanged | IdleInhibitCmd::StateChanged => {
                self.sync_inhibitor();
                let config = &self.config.config().modules.idle_inhibit;
                self.update_display(config);
            }
        }
    }
}

impl Drop for IdleInhibitModule {
    fn drop(&mut self) {
        self.inhibitor.take();
    }
}
