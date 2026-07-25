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
    messages::{ClockCmd, ClockInit, ClockMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct ClockModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
    last_label_len: usize,
}

#[relm4::component(pub(crate))]
impl Component for ClockModule {
    type Init = ClockInit;
    type Input = ClockMsg;
    type Output = ();
    type CommandOutput = ClockCmd;

    view! {
        gtk::Box {
            add_css_class: "clock",
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
        let clock = &config.modules.clock;
        let formatted_time = helpers::format_time(&clock.format.get());
        let initial_label_len = formatted_time.chars().count();

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: clock.icon_name.get().clone(),
                label: formatted_time,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: clock.icon_color.clone(),
                    label_color: clock.label_color.clone(),
                    icon_background: clock.icon_bg_color.clone(),
                    button_background: clock.button_bg_color.clone(),
                    border_color: clock.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: clock.label_max_length.clone(),
                    show_icon: clock.icon_show.clone(),
                    show_label: clock.label_show.clone(),
                    show_border: clock.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => ClockMsg::LeftClick,
                BarButtonOutput::RightClick => ClockMsg::RightClick,
                BarButtonOutput::MiddleClick => ClockMsg::MiddleClick,
                BarButtonOutput::ScrollUp => ClockMsg::ScrollUp,
                BarButtonOutput::ScrollDown => ClockMsg::ScrollDown,
            });

        // Only the time (HH:MM) is reserved. The date changes at most once a day, so
        // it gets no reserved space and simply renders at its natural width.
        bar_button.emit(BarButtonInput::SetLabelReserveTimeOnly(true));

        watchers::spawn_watchers(&sender, clock);

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
        let clock = &self.config.config().modules.clock;

        let action = match msg {
            ClockMsg::LeftClick => clock.left_click.get(),
            ClockMsg::RightClick => clock.right_click.get(),
            ClockMsg::MiddleClick => clock.middle_click.get(),
            ClockMsg::ScrollUp => clock.scroll_up.get(),
            ClockMsg::ScrollDown => clock.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: ClockCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            ClockCmd::UpdateTime(time) => {
                let new_len = time.chars().count();
                self.bar_button.emit(BarButtonInput::SetLabel(time));
                if new_len != self.last_label_len {
                    self.last_label_len = new_len;
                    force_window_resize(root);
                }
            }
            ClockCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
        }
    }
}
