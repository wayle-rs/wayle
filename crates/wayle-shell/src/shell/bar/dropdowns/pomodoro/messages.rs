use std::sync::Arc;

use wayle_config::ConfigService;

use crate::shell::{SharedPomodoroState, bar::pomodoro::PomodoroSnapshot};

pub(crate) struct PomodoroDropdownInit {
    pub config: Arc<ConfigService>,
    pub state: SharedPomodoroState,
}

#[derive(Debug)]
pub(crate) enum PomodoroDropdownInput {
    ToggleRunning,
    SwitchToWork,
    SwitchToShortBreak,
    SwitchToLongBreak,
}

#[derive(Debug)]
pub(crate) enum PomodoroDropdownCmd {
    StateChanged(PomodoroSnapshot),
    ScaleChanged(f32),
    UpdateDurations {
        work: u32,
        short_break: u32,
        long_break: u32,
        cycles: u32,
    },
}
