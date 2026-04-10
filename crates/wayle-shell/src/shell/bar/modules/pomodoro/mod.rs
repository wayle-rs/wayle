mod factory;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::{modules::PomodoroConfig, styling::CssToken}};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{PomodoroCmd, PomodoroInit, PomodoroMsg},
};
use crate::shell::{
    PomodoroMode, PomodoroSnapshot, SharedPomodoroState, TimerState,
    bar::dropdowns::{self, DropdownRegistry},
};

pub(crate) struct PomodoroModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
    state: SharedPomodoroState,
}

#[relm4::component(pub(crate))]
impl Component for PomodoroModule {
    type Init = PomodoroInit;
    type Input = PomodoroMsg;
    type Output = ();
    type CommandOutput = PomodoroCmd;

    view! {
        gtk::Box {
            add_css_class: "pomodoro",

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
        let pomodoro_config = &config.modules.pomodoro;

        let initial_snapshot = init.state.snapshot();
        let initial_label = initial_snapshot.format_time();
        Self::apply_mode_css_class(&root, &initial_snapshot);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: Self::icon_for_snapshot(&initial_snapshot, pomodoro_config),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: pomodoro_config.icon_color.clone(),
                    label_color: pomodoro_config.label_color.clone(),
                    icon_background: pomodoro_config.icon_bg_color.clone(),
                    button_background: pomodoro_config.button_bg_color.clone(),
                    border_color: pomodoro_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: pomodoro_config.label_max_length.clone(),
                    show_icon: pomodoro_config.icon_show.clone(),
                    show_label: pomodoro_config.label_show.clone(),
                    show_border: pomodoro_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => PomodoroMsg::LeftClick,
                BarButtonOutput::RightClick => PomodoroMsg::RightClick,
                BarButtonOutput::MiddleClick => PomodoroMsg::MiddleClick,
                BarButtonOutput::ScrollUp => PomodoroMsg::ScrollUp,
                BarButtonOutput::ScrollDown => PomodoroMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, pomodoro_config, &init.state);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
            state: init.state,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.pomodoro;

        let action = match msg {
            PomodoroMsg::LeftClick => config.left_click.get(),
            PomodoroMsg::RightClick => config.right_click.get(),
            PomodoroMsg::MiddleClick => config.middle_click.get(),
            PomodoroMsg::ScrollUp => config.scroll_up.get(),
            PomodoroMsg::ScrollDown => config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            PomodoroCmd::StateChanged(snapshot) => {
                let pomodoro_config = &self.config.config().modules.pomodoro;
                let icon = Self::icon_for_snapshot(&snapshot, pomodoro_config);
                Self::apply_mode_css_class(root, &snapshot);
                self.bar_button
                    .emit(BarButtonInput::SetLabel(snapshot.format_time()));
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            PomodoroCmd::UpdateIcon(icon) => {
                if self.state.snapshot().timer_state == TimerState::Stopped {
                    self.bar_button.emit(BarButtonInput::SetIcon(icon));
                }
            }
            PomodoroCmd::UpdateDurations {
                work,
                short_break,
                long_break,
                cycles,
            } => {
                self.state
                    .update_durations(work, short_break, long_break, cycles);
            }
        }
    }
}

impl PomodoroModule {
    fn icon_for_snapshot(snapshot: &PomodoroSnapshot, config: &PomodoroConfig) -> String {
        match snapshot.timer_state {
            TimerState::Stopped => config.icon_name.get().clone(),
            TimerState::Running => String::from("ld-play-symbolic"),
            TimerState::Paused => String::from("ld-pause-symbolic"),
        }
    }

    fn apply_mode_css_class(root: &gtk::Box, snapshot: &PomodoroSnapshot) {
        match snapshot.mode {
            PomodoroMode::Work => {
                root.remove_css_class("pomodoro-short-break");
                root.remove_css_class("pomodoro-long-break");
                root.add_css_class("pomodoro-work");
            }
            PomodoroMode::ShortBreak => {
                root.remove_css_class("pomodoro-work");
                root.remove_css_class("pomodoro-long-break");
                root.add_css_class("pomodoro-short-break");
            }
            PomodoroMode::LongBreak => {
                root.remove_css_class("pomodoro-work");
                root.remove_css_class("pomodoro-short-break");
                root.add_css_class("pomodoro-long-break");
            }
        }
    }
}
