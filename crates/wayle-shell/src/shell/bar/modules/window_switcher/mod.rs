//! Window switcher bar module: an icon and an open-window counter, backed
//! by `wlr-foreign-toplevel-management-unstable-v1`. Left click opens a
//! dropdown listing every window (see `bar::dropdowns::window_switcher`).
//!
//! Distinct from `window_title`, which only displays the currently
//! focused window's title and has no list of its own.

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
    messages::{WindowSwitcherCmd, WindowSwitcherInit, WindowSwitcherMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct WindowSwitcherModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for WindowSwitcherModule {
    type Init = WindowSwitcherInit;
    type Input = WindowSwitcherMsg;
    type Output = ();
    type CommandOutput = WindowSwitcherCmd;

    view! {
        gtk::Box {
            add_css_class: "window-switcher",

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
        let switcher_config = &config.modules.window_switcher;

        let initial_count = helpers::count_visible(
            &init.service.toplevels.get(),
            &switcher_config.ignore_app_id.get(),
        );

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: switcher_config.icon.get(),
                label: helpers::count_label(initial_count, switcher_config.hide_when_empty.get()),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: switcher_config.icon_color.clone(),
                    label_color: switcher_config.label_color.clone(),
                    icon_background: switcher_config.icon_bg_color.clone(),
                    button_background: switcher_config.button_bg_color.clone(),
                    border_color: switcher_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: switcher_config.label_max_length.clone(),
                    show_icon: switcher_config.icon_show.clone(),
                    show_label: switcher_config.label_show.clone(),
                    show_border: switcher_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => WindowSwitcherMsg::LeftClick,
                BarButtonOutput::RightClick => WindowSwitcherMsg::RightClick,
                BarButtonOutput::MiddleClick => WindowSwitcherMsg::MiddleClick,
                BarButtonOutput::ScrollUp => WindowSwitcherMsg::ScrollUp,
                BarButtonOutput::ScrollDown => WindowSwitcherMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, switcher_config, &init.service);

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
        let config = &self.config.config().modules.window_switcher;

        let action = match msg {
            WindowSwitcherMsg::LeftClick => config.left_click.get(),
            WindowSwitcherMsg::RightClick => config.right_click.get(),
            WindowSwitcherMsg::MiddleClick => config.middle_click.get(),
            WindowSwitcherMsg::ScrollUp => config.scroll_up.get(),
            WindowSwitcherMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: WindowSwitcherCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            WindowSwitcherCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            WindowSwitcherCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
