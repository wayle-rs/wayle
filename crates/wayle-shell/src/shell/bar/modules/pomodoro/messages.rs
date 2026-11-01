use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::{PomodoroSnapshot, SharedPomodoroState, bar::dropdowns::DropdownRegistry};

pub(crate) struct PomodoroInit {
    pub settings: BarSettings,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
    pub state: SharedPomodoroState,
}

#[derive(Debug)]
pub(crate) enum PomodoroMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum PomodoroCmd {
    StateChanged(PomodoroSnapshot),
    UpdateIcon(String),
    UpdateDurations {
        work: u32,
        short_break: u32,
        long_break: u32,
        cycles: u32,
    },
}
