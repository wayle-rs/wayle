mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::rc::Rc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ClickAction, ConfigProperty,
    schemas::{
        modules::{CustomModuleDefinition, ExecutionMode},
        styling::{ColorValue, CssToken},
    },
};
use wayle_widgets::{
    WatcherToken,
    prelude::{BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonOutput},
    utils::force_window_resize,
};

use self::helpers::{ParsedOutput, format_label};
pub(crate) use self::{
    factory::Factory,
    messages::{CustomCmd, CustomInit, CustomMsg},
};
use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct CustomModule {
    bar_button: Controller<BarButton>,
    definition: CustomModuleDefinition,
    definition_present: bool,
    #[allow(dead_code)]
    dropdowns: Rc<DropdownRegistry>,
    poller_token: WatcherToken,
    watcher_token: WatcherToken,
    command_token: WatcherToken,
    scroll_debounce_token: WatcherToken,
    show_icon: ConfigProperty<bool>,
    show_label: ConfigProperty<bool>,
    show_border: ConfigProperty<bool>,
    label_max_chars: ConfigProperty<u32>,
    icon_color: ConfigProperty<ColorValue>,
    label_color: ConfigProperty<ColorValue>,
    icon_bg_color: ConfigProperty<ColorValue>,
    button_bg_color: ConfigProperty<ColorValue>,
    border_color: ConfigProperty<ColorValue>,
    dynamic_classes: Vec<String>,
    last_output: String,
}

#[relm4::component(pub(crate))]
impl Component for CustomModule {
    type Init = CustomInit;
    type Input = CustomMsg;
    type Output = ();
    type CommandOutput = CustomCmd;

    view! {
        gtk::Box {
            add_css_class: "custom",
            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let definition = init.definition;

        let show_icon = ConfigProperty::new(definition.icon_show);
        let show_label = ConfigProperty::new(definition.label_show);
        let show_border = ConfigProperty::new(definition.border_show);
        let label_max_chars = ConfigProperty::new(definition.label_max_length);
        let icon_color = ConfigProperty::new(definition.icon_color.clone());
        let label_color = ConfigProperty::new(definition.label_color.clone());
        let icon_bg_color = ConfigProperty::new(definition.icon_bg_color.clone());
        let button_bg_color = ConfigProperty::new(definition.button_bg_color.clone());
        let border_color = ConfigProperty::new(definition.border_color.clone());

        let initial_label = format_label(&definition, &ParsedOutput::default());

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: definition.icon_name.clone(),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: icon_color.clone(),
                    label_color: label_color.clone(),
                    icon_background: icon_bg_color.clone(),
                    button_background: button_bg_color.clone(),
                    border_color: border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: label_max_chars.clone(),
                    show_icon: show_icon.clone(),
                    show_label: show_label.clone(),
                    show_border: show_border.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => CustomMsg::LeftClick,
                BarButtonOutput::RightClick => CustomMsg::RightClick,
                BarButtonOutput::MiddleClick => CustomMsg::MiddleClick,
                BarButtonOutput::ScrollUp => CustomMsg::ScrollUp,
                BarButtonOutput::ScrollDown => CustomMsg::ScrollDown,
            });

        let custom_modules = &init.config.config().modules.custom;
        let mut poller_token = WatcherToken::new();
        let mut watcher_token = WatcherToken::new();
        let mut command_token = WatcherToken::new();

        match definition.mode {
            ExecutionMode::Poll => {
                watchers::spawn_command_poller(&sender, &definition, poller_token.reset());
                watchers::run_definition_command(&sender, &definition, command_token.reset());
            }
            ExecutionMode::Watch => {
                watchers::spawn_command_watcher(&sender, &definition, watcher_token.reset());
            }
        }
        watchers::spawn_config_watcher(&sender, custom_modules, definition.id.clone());

        let model = Self {
            bar_button,
            definition,
            definition_present: true,
            dropdowns: init.dropdowns,
            poller_token,
            watcher_token,
            command_token,
            scroll_debounce_token: WatcherToken::new(),
            show_icon,
            show_label,
            show_border,
            label_max_chars,
            icon_color,
            label_color,
            icon_bg_color,
            button_bg_color,
            border_color,
            dynamic_classes: Vec::new(),
            last_output: String::new(),
        };

        if helpers::should_hide("", model.definition.hide_if_empty) {
            root.set_visible(false);
        }

        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        let is_scroll = matches!(msg, CustomMsg::ScrollUp | CustomMsg::ScrollDown);

        let action = match msg {
            CustomMsg::LeftClick => &self.definition.left_click,
            CustomMsg::RightClick => &self.definition.right_click,
            CustomMsg::MiddleClick => &self.definition.middle_click,
            CustomMsg::ScrollUp => &self.definition.scroll_up,
            CustomMsg::ScrollDown => &self.definition.scroll_down,
        };

        match action {
            ClickAction::Dropdown(name) => {
                crate::shell::bar::dropdowns::dispatch_click(
                    action,
                    &self.dropdowns,
                    &self.bar_button,
                );
                // Schedule on_action to run when the dropdown closes,
                // so the bar label refreshes after a selection.
                if let Some(on_action) = &self.definition.on_action {
                    let on_action = on_action.clone();
                    let module_id = self.definition.id.clone();
                    let token = self.command_token.reset();
                    let sender = sender.clone();
                    self.dropdowns.on_next_close(name, move || {
                        watchers::run_command_async(&sender, &module_id, on_action, token);
                    });
                }
                return;
            }
            ClickAction::Shell(cmd) => {
                watchers::spawn_action(cmd);
            }
            ClickAction::None => return,
        }

        let Some(on_action) = &self.definition.on_action else {
            return;
        };

        if is_scroll {
            let token = self.scroll_debounce_token.reset();
            watchers::spawn_scroll_debounce(&sender, token);
        } else {
            let token = self.command_token.reset();
            watchers::run_command_async(&sender, &self.definition.id, on_action.clone(), token);
        }
    }

    fn update_cmd(&mut self, msg: CustomCmd, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            CustomCmd::PollTrigger => {
                if let Some(command) = &self.definition.command {
                    let token = self.command_token.reset();
                    watchers::run_command_async(
                        &sender,
                        &self.definition.id,
                        command.clone(),
                        token,
                    );
                }
            }
            CustomCmd::ScrollDebounceExpired => {
                if let Some(on_action) = &self.definition.on_action {
                    let token = self.command_token.reset();
                    watchers::run_command_async(
                        &sender,
                        &self.definition.id,
                        on_action.clone(),
                        token,
                    );
                }
            }
            CustomCmd::CommandCancelled => {}
            CustomCmd::CommandOutput(output) | CustomCmd::WatchOutput(output) => {
                self.apply_output(&output, root);
                force_window_resize(root);
            }
            CustomCmd::DefinitionRemoved => {
                self.handle_definition_removed(root);
            }
            CustomCmd::DefinitionChanged(boxed_definition) => {
                self.handle_definition_changed(&sender, root, *boxed_definition);
            }
        }
    }
}
