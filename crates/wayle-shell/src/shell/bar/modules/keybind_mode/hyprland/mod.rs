mod methods;
mod watchers;

use std::sync::Arc;

use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::{
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
    utils::force_window_resize,
};

use super::{
    helpers,
    messages::{KeybindModeCmd, KeybindModeInit, KeybindModeMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct HyprlandKeybindMode {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    current_mode: String,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for HyprlandKeybindMode {
    type Init = KeybindModeInit;
    type Input = KeybindModeMsg;
    type Output = ();
    type CommandOutput = KeybindModeCmd;

    view! {
        gtk::Box {
            add_css_class: "keybind-mode",
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
        let mode_config = &config.modules.keybind_mode;

        let initial_mode = Self::initial_mode(&init.hyprland);
        let formatted_label = helpers::format_label(&mode_config.format.get(), &initial_mode);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: mode_config.icon_name.get().clone(),
                label: formatted_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: mode_config.icon_color.clone(),
                    label_color: mode_config.label_color.clone(),
                    icon_background: mode_config.icon_bg_color.clone(),
                    button_background: mode_config.button_bg_color.clone(),
                    border_color: mode_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: mode_config.label_max_length.clone(),
                    show_icon: mode_config.icon_show.clone(),
                    show_label: mode_config.label_show.clone(),
                    show_border: mode_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => KeybindModeMsg::LeftClick,
                BarButtonOutput::RightClick => KeybindModeMsg::RightClick,
                BarButtonOutput::MiddleClick => KeybindModeMsg::MiddleClick,
                BarButtonOutput::ScrollUp => KeybindModeMsg::ScrollUp,
                BarButtonOutput::ScrollDown => KeybindModeMsg::ScrollDown,
            });

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            mode_config.clone(),
        );

        watchers::spawn_watchers(&sender, mode_config, &init.hyprland);

        let model = Self {
            bar_button,
            config: init.config,
            current_mode: initial_mode,
            opener,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let mode_config = &self.config.config().modules.keybind_mode;

        let action = match msg {
            KeybindModeMsg::LeftClick => mode_config.left_click.get(),
            KeybindModeMsg::RightClick => mode_config.right_click.get(),
            KeybindModeMsg::MiddleClick => mode_config.middle_click.get(),
            KeybindModeMsg::ScrollUp => mode_config.scroll_up.get(),
            KeybindModeMsg::ScrollDown => mode_config.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(
        &mut self,
        msg: KeybindModeCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            KeybindModeCmd::ModeChanged { name, format } => {
                self.current_mode = name;
                self.update_display(&format, root);
            }
            KeybindModeCmd::FormatChanged => {
                let format = self.config.config().modules.keybind_mode.format.get();
                self.update_display(&format, root);
            }
            KeybindModeCmd::AutoHideChanged => {
                let auto_hide = self.config.config().modules.keybind_mode.auto_hide.get();
                let visible = helpers::compute_visibility(&self.current_mode, auto_hide);
                if let Some(parent) = root.parent() {
                    parent.set_visible(visible);
                }
                force_window_resize(root);
            }
            KeybindModeCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
