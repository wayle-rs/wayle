//! The agnostic [`KeyboardInput`] component: model, view, and message routing.

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

use super::{
    helpers,
    messages::{KeyboardInputCmd, KeyboardInputInit, KeyboardInputMsg},
    watchers,
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

const UNKNOWN_LAYOUT: &str = "?";

pub(crate) struct KeyboardInput {
    pub(super) bar_button: Controller<BarButton>,
    pub(super) config: Arc<ConfigService>,
    pub(super) current_layout: String,
    pub(super) dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for KeyboardInput {
    type Init = KeyboardInputInit;
    type Input = KeyboardInputMsg;
    type Output = ();
    type CommandOutput = KeyboardInputCmd;

    view! {
        gtk::Box {
            add_css_class: "keyboard-input",

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
        let keyboard_input = &config.modules.keyboard_input;

        let initial_layout = init
            .source
            .snapshot()
            .map(|layout| layout.label)
            .unwrap_or_else(|| UNKNOWN_LAYOUT.to_string());

        let formatted_label = helpers::format_label(
            &initial_layout,
            &keyboard_input.format.get(),
            &keyboard_input.layout_alias_map.get(),
        );

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: keyboard_input.icon_name.get().clone(),
                label: formatted_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: keyboard_input.icon_color.clone(),
                    label_color: keyboard_input.label_color.clone(),
                    icon_background: keyboard_input.icon_bg_color.clone(),
                    button_background: keyboard_input.button_bg_color.clone(),
                    border_color: keyboard_input.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: keyboard_input.label_max_length.clone(),
                    show_icon: keyboard_input.icon_show.clone(),
                    show_label: keyboard_input.label_show.clone(),
                    show_border: keyboard_input.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => KeyboardInputMsg::LeftClick,
                BarButtonOutput::RightClick => KeyboardInputMsg::RightClick,
                BarButtonOutput::MiddleClick => KeyboardInputMsg::MiddleClick,
                BarButtonOutput::ScrollUp => KeyboardInputMsg::ScrollUp,
                BarButtonOutput::ScrollDown => KeyboardInputMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, keyboard_input, init.source);

        let model = Self {
            bar_button,
            config: init.config,
            current_layout: initial_layout,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let keyboard_input = &self.config.config().modules.keyboard_input;

        let action = match msg {
            KeyboardInputMsg::LeftClick => keyboard_input.left_click.get(),
            KeyboardInputMsg::RightClick => keyboard_input.right_click.get(),
            KeyboardInputMsg::MiddleClick => keyboard_input.middle_click.get(),
            KeyboardInputMsg::ScrollUp => keyboard_input.scroll_up.get(),
            KeyboardInputMsg::ScrollDown => keyboard_input.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: KeyboardInputCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            KeyboardInputCmd::LayoutChanged(layout) => {
                self.current_layout = layout
                    .map(|current| current.label)
                    .unwrap_or_else(|| UNKNOWN_LAYOUT.to_string());
                self.update_label(root);
            }
            KeyboardInputCmd::LayoutAliasMapChanged | KeyboardInputCmd::FormatChanged => {
                self.update_label(root);
            }
            KeyboardInputCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
