mod factory;
mod messages;
mod watchers;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::messages::{PomodoroDropdownCmd, PomodoroDropdownInit, PomodoroDropdownInput};
use crate::{
    i18n::t,
    shell::{PomodoroMode, PomodoroSnapshot, SharedPomodoroState, TimerState},
};

const BASE_WIDTH: f32 = 340.0;

pub(crate) struct PomodoroDropdown {
    scaled_width: i32,
    state: SharedPomodoroState,
    snapshot: PomodoroSnapshot,
}

#[relm4::component(pub(crate))]
impl Component for PomodoroDropdown {
    type Init = PomodoroDropdownInit;
    type Input = PomodoroDropdownInput;
    type Output = ();
    type CommandOutput = PomodoroDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "pomodoro-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("ld-clock-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-pomodoro-title"),
                    },
                    #[template_child]
                    actions {
                        set_visible: false,
                    },
                },

                #[template]
                DropdownContent {

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 0,

                        gtk::Box {
                            add_css_class: "pomodoro-hero",
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 16,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,

                            gtk::Box {
                                add_css_class: "pomodoro-hero-meta",
                                set_orientation: gtk::Orientation::Vertical,
                                set_valign: gtk::Align::Center,

                                gtk::Label {
                                    add_css_class: "pomodoro-hero-time",
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    #[watch]
                                    set_label: &model.snapshot.format_time(),
                                },
                            },

                            gtk::Box {
                                add_css_class: "pomodoro-hero-controls",
                                set_orientation: gtk::Orientation::Horizontal,
                                set_valign: gtk::Align::Center,

                                gtk::Button {
                                    add_css_class: "pomodoro-hero-toggle",
                                    set_valign: gtk::Align::Center,
                                    #[watch]
                                    set_tooltip_text: Some(&model.primary_action_tooltip()),
                                    connect_clicked => PomodoroDropdownInput::ToggleRunning,

                                    gtk::Image {
                                        set_valign: gtk::Align::Center,
                                        #[watch]
                                        set_icon_name: Some(model.primary_action_icon()),
                                        set_pixel_size: 40,
                                    },
                                },
                            },
                        },

                        gtk::Separator {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_margin_top: 12,
                            set_margin_bottom: 12,
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,

                            gtk::Label {
                                add_css_class: "section-label",
                                set_label: &t!("dropdown-pomodoro-mode-section"),
                                set_halign: gtk::Align::Start,
                            },

                            #[name = "mode_segmented"]
                            gtk::Box {
                                add_css_class: "profile-seg",
                                set_homogeneous: true,

                                #[name = "work_btn"]
                                gtk::ToggleButton {
                                    add_css_class: "profile-seg-btn",
                                    set_cursor_from_name: Some("pointer"),
                                    set_hexpand: true,
                                    #[watch]
                                    #[block_signal(work_handler)]
                                    set_active: model.snapshot.mode == PomodoroMode::Work,
                                    connect_toggled[sender] => move |btn| {
                                        if btn.is_active() {
                                            sender.input(PomodoroDropdownInput::SwitchToWork);
                                        }
                                    } @work_handler,

                                    gtk::Box {
                                        add_css_class: "profile-seg-btn-content",
                                        set_halign: gtk::Align::Center,

                                        gtk::Label {
                                            set_label: &t!("dropdown-pomodoro-mode-work"),
                                        },
                                    },
                                },

                                #[name = "short_break_btn"]
                                gtk::ToggleButton {
                                    add_css_class: "profile-seg-btn",
                                    set_cursor_from_name: Some("pointer"),
                                    set_hexpand: true,
                                    set_group: Some(&work_btn),
                                    #[watch]
                                    #[block_signal(short_handler)]
                                    set_active: model.snapshot.mode == PomodoroMode::ShortBreak,
                                    connect_toggled[sender] => move |btn| {
                                        if btn.is_active() {
                                            sender.input(PomodoroDropdownInput::SwitchToShortBreak);
                                        }
                                    } @short_handler,

                                    gtk::Box {
                                        add_css_class: "profile-seg-btn-content",
                                        set_halign: gtk::Align::Center,

                                        gtk::Label {
                                            set_label: &t!("dropdown-pomodoro-mode-short-break"),
                                        },
                                    },
                                },

                                #[name = "long_break_btn"]
                                gtk::ToggleButton {
                                    add_css_class: "profile-seg-btn",
                                    set_cursor_from_name: Some("pointer"),
                                    set_hexpand: true,
                                    set_group: Some(&work_btn),
                                    #[watch]
                                    #[block_signal(long_handler)]
                                    set_active: model.snapshot.mode == PomodoroMode::LongBreak,
                                    connect_toggled[sender] => move |btn| {
                                        if btn.is_active() {
                                            sender.input(PomodoroDropdownInput::SwitchToLongBreak);
                                        }
                                    } @long_handler,

                                    gtk::Box {
                                        add_css_class: "profile-seg-btn-content",
                                        set_halign: gtk::Align::Center,

                                        gtk::Label {
                                            set_label: &t!("dropdown-pomodoro-mode-long-break"),
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let pomodoro_config = &config.modules.pomodoro;
        let scale = config.styling.scale.get().value();

        let model = Self {
            scaled_width: super::scaled_dimension(BASE_WIDTH, scale),
            snapshot: init.state.snapshot(),
            state: init.state,
        };

        watchers::spawn_watchers(&sender, &init.config, pomodoro_config, &model.state);

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.snapshot = match msg {
            PomodoroDropdownInput::ToggleRunning => {
                if self.snapshot.timer_state == TimerState::Running {
                    self.state.pause()
                } else {
                    self.state.start()
                }
            }
            PomodoroDropdownInput::SwitchToWork => self.state.switch_mode(PomodoroMode::Work),
            PomodoroDropdownInput::SwitchToShortBreak => {
                self.state.switch_mode(PomodoroMode::ShortBreak)
            }
            PomodoroDropdownInput::SwitchToLongBreak => {
                self.state.switch_mode(PomodoroMode::LongBreak)
            }
        };
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            PomodoroDropdownCmd::StateChanged(snapshot) => {
                self.snapshot = snapshot;
            }
            PomodoroDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = super::scaled_dimension(BASE_WIDTH, scale);
            }
            PomodoroDropdownCmd::UpdateDurations {
                work,
                short_break,
                long_break,
                cycles,
            } => {
                self.snapshot = self
                    .state
                    .update_durations(work, short_break, long_break, cycles);
            }
        }
    }
}

impl PomodoroDropdown {
    fn primary_action_icon(&self) -> &'static str {
        if self.snapshot.timer_state == TimerState::Running {
            "media-playback-pause-symbolic"
        } else {
            "media-playback-start-symbolic"
        }
    }

    fn primary_action_tooltip(&self) -> String {
        if self.snapshot.timer_state == TimerState::Running {
            t!("dropdown-pomodoro-pause")
        } else {
            t!("dropdown-pomodoro-start")
        }
    }
}
