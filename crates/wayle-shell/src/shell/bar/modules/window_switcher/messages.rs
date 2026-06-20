use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;
use wayle_wlr_toplevel::WlrToplevelService;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct WindowSwitcherInit {
    pub settings: BarSettings,
    pub service: Arc<WlrToplevelService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum WindowSwitcherMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum WindowSwitcherCmd {
    UpdateLabel(String),
    UpdateIcon(String),
}
