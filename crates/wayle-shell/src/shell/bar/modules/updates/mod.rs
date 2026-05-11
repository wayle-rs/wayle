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
    messages::{UpdatesCmd, UpdatesInit, UpdatesMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct UpdatesModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for UpdatesModule {
    type Init = UpdatesInit;
    type Input = UpdatesMsg;
    type Output = ();
    type CommandOutput = UpdatesCmd;

    view! {
        gtk::Box {
            add_css_class: "updates",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let updates_config = &config.modules.updates;

        let initial_label = String::from("...");

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: updates_config.icon_name.get().clone(),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: updates_config.icon_color.clone(),
                    label_color: updates_config.label_color.clone(),
                    icon_background: updates_config.icon_bg_color.clone(),
                    button_background: updates_config.button_bg_color.clone(),
                    border_color: updates_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: updates_config.label_max_length.clone(),
                    show_icon: updates_config.icon_show.clone(),
                    show_label: updates_config.label_show.clone(),
                    show_border: updates_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => UpdatesMsg::LeftClick,
                BarButtonOutput::RightClick => UpdatesMsg::RightClick,
                BarButtonOutput::MiddleClick => UpdatesMsg::MiddleClick,
                BarButtonOutput::ScrollUp => UpdatesMsg::ScrollUp,
                BarButtonOutput::ScrollDown => UpdatesMsg::ScrollDown,
            });

        // Hide initially if hide-if-zero is set (will show when updates found)
        if updates_config.hide_if_zero.get() {
            root.set_visible(false);
        }

        watchers::spawn_watchers(&sender, updates_config);

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
        let updates_config = &self.config.config().modules.updates;

        let action = match msg {
            UpdatesMsg::LeftClick => updates_config.left_click.get(),
            UpdatesMsg::RightClick => updates_config.right_click.get(),
            UpdatesMsg::MiddleClick => updates_config.middle_click.get(),
            UpdatesMsg::ScrollUp => updates_config.scroll_up.get(),
            UpdatesMsg::ScrollDown => updates_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: UpdatesCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            UpdatesCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            UpdatesCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            UpdatesCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
            UpdatesCmd::UpdateVisibility(visible) => {
                root.set_visible(visible);
            }
        }
    }
}
