//! The agnostic [`WindowTitle`] component: model, view, and message routing.

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

use super::{
    helpers::{self, IconContext},
    messages::{WindowTitleCmd, WindowTitleInit, WindowTitleMsg},
    watchers,
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct WindowTitle {
    pub(super) bar_button: Controller<BarButton>,
    pub(super) config: Arc<ConfigService>,
    pub(super) current_title: String,
    pub(super) current_app_id: String,
    pub(super) dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for WindowTitle {
    type Init = WindowTitleInit;
    type Input = WindowTitleMsg;
    type Output = ();
    type CommandOutput = WindowTitleCmd;

    view! {
        gtk::Box {
            add_css_class: "window-title",

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
        let window_title = &config.modules.window_title;

        let (initial_title, initial_app_id) = init
            .source
            .snapshot()
            .map(|window| (window.title, window.app_id))
            .unwrap_or_default();

        let formatted_label =
            helpers::format_label(&window_title.format.get(), &initial_title, &initial_app_id);
        let initial_icon = helpers::resolve_icon(&IconContext {
            title: &initial_title,
            app_id: &initial_app_id,
            user_mappings: &window_title.icon_mappings.get(),
            fallback: &window_title.icon_name.get(),
        });

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: formatted_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: window_title.icon_color.clone(),
                    label_color: window_title.label_color.clone(),
                    icon_background: window_title.icon_bg_color.clone(),
                    button_background: window_title.button_bg_color.clone(),
                    border_color: window_title.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: window_title.label_max_length.clone(),
                    show_icon: window_title.icon_show.clone(),
                    show_label: window_title.label_show.clone(),
                    show_border: window_title.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => WindowTitleMsg::LeftClick,
                BarButtonOutput::RightClick => WindowTitleMsg::RightClick,
                BarButtonOutput::MiddleClick => WindowTitleMsg::MiddleClick,
                BarButtonOutput::ScrollUp => WindowTitleMsg::ScrollUp,
                BarButtonOutput::ScrollDown => WindowTitleMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, window_title, init.source);

        let model = Self {
            bar_button,
            config: init.config,
            current_title: initial_title,
            current_app_id: initial_app_id,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let window_title = &self.config.config().modules.window_title;

        let action = match msg {
            WindowTitleMsg::LeftClick => window_title.left_click.get(),
            WindowTitleMsg::RightClick => window_title.right_click.get(),
            WindowTitleMsg::MiddleClick => window_title.middle_click.get(),
            WindowTitleMsg::ScrollUp => window_title.scroll_up.get(),
            WindowTitleMsg::ScrollDown => window_title.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: WindowTitleCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            WindowTitleCmd::WindowChanged { focused, format } => {
                let (title, app_id) = focused
                    .map(|window| (window.title, window.app_id))
                    .unwrap_or_default();
                self.current_title = title;
                self.current_app_id = app_id;
                self.update_display(&format, root);
            }
            WindowTitleCmd::FormatChanged => {
                let format = self.config.config().modules.window_title.format.get();
                self.update_label(&format, root);
            }
            WindowTitleCmd::IconConfigChanged => {
                let window_title = &self.config.config().modules.window_title;
                let icon = helpers::resolve_icon(&IconContext {
                    title: &self.current_title,
                    app_id: &self.current_app_id,
                    user_mappings: &window_title.icon_mappings.get(),
                    fallback: &window_title.icon_name.get(),
                });
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
