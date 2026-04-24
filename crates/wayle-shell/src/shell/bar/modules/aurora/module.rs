mod factory;
pub(crate) mod helpers;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken, schemas::modules::AuroraConfig};
use wayle_widgets::{
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
    utils::force_window_resize,
};

pub(crate) use self::{
    factory::Factory,
    messages::{AuroraCmd, AuroraInit, AuroraMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct AuroraModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for AuroraModule {
    type Init = AuroraInit;
    type Input = AuroraMsg;
    type Output = ();
    type CommandOutput = AuroraCmd;

    view! {
        gtk::Box {
            add_css_class: "aurora",

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
        let aurora_config = &config.modules.aurora;

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: aurora_config.icon_name.get().clone(),
                label: String::from("--"),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: aurora_config.icon_color.clone(),
                    label_color: aurora_config.label_color.clone(),
                    icon_background: aurora_config.icon_bg_color.clone(),
                    button_background: aurora_config.button_bg_color.clone(),
                    border_color: aurora_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: aurora_config.label_max_length.clone(),
                    show_icon: aurora_config.icon_show.clone(),
                    show_label: aurora_config.label_show.clone(),
                    show_border: aurora_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => AuroraMsg::LeftClick,
                BarButtonOutput::RightClick => AuroraMsg::RightClick,
                BarButtonOutput::MiddleClick => AuroraMsg::MiddleClick,
                BarButtonOutput::ScrollUp => AuroraMsg::ScrollUp,
                BarButtonOutput::ScrollDown => AuroraMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, aurora_config, &init.weather);

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
        let aurora = &self.config.config().modules.aurora;

        let action = match msg {
            AuroraMsg::LeftClick => aurora.left_click.get(),
            AuroraMsg::RightClick => aurora.right_click.get(),
            AuroraMsg::MiddleClick => aurora.middle_click.get(),
            AuroraMsg::ScrollUp => aurora.scroll_up.get(),
            AuroraMsg::ScrollDown => aurora.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: AuroraCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            AuroraCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
                force_window_resize(root);
            }
            AuroraCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
