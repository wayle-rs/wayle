mod factory;
mod helpers;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

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
    messages::{WorldClockCmd, WorldClockInit, WorldClockMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct WorldClockModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
    last_label_len: usize,
}

#[relm4::component(pub(crate))]
impl Component for WorldClockModule {
    type Init = WorldClockInit;
    type Input = WorldClockMsg;
    type Output = ();
    type CommandOutput = WorldClockCmd;

    view! {
        gtk::Box {
            add_css_class: "world-clock",

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
        let world_clock = &config.modules.world_clock;
        let label = watchers::render_label(&world_clock.format.get());
        let initial_label_len = label.chars().count();

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: world_clock.icon_name.get().clone(),
                label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: world_clock.icon_color.clone(),
                    label_color: world_clock.label_color.clone(),
                    icon_background: world_clock.icon_bg_color.clone(),
                    button_background: world_clock.button_bg_color.clone(),
                    border_color: world_clock.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: world_clock.label_max_length.clone(),
                    show_icon: world_clock.icon_show.clone(),
                    show_label: world_clock.label_show.clone(),
                    show_border: world_clock.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => WorldClockMsg::LeftClick,
                BarButtonOutput::RightClick => WorldClockMsg::RightClick,
                BarButtonOutput::MiddleClick => WorldClockMsg::MiddleClick,
                BarButtonOutput::ScrollUp => WorldClockMsg::ScrollUp,
                BarButtonOutput::ScrollDown => WorldClockMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, world_clock);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
            last_label_len: initial_label_len,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let world_clock = &self.config.config().modules.world_clock;

        let action = match msg {
            WorldClockMsg::LeftClick => world_clock.left_click.get(),
            WorldClockMsg::RightClick => world_clock.right_click.get(),
            WorldClockMsg::MiddleClick => world_clock.middle_click.get(),
            WorldClockMsg::ScrollUp => world_clock.scroll_up.get(),
            WorldClockMsg::ScrollDown => world_clock.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: WorldClockCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            WorldClockCmd::UpdateLabel(label) => {
                let new_len = label.chars().count();
                self.bar_button.emit(BarButtonInput::SetLabel(label));
                if new_len != self.last_label_len {
                    self.last_label_len = new_len;
                    force_window_resize(root);
                }
            }
            WorldClockCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
