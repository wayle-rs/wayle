use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;
use wayle_wlr_toplevel::WlrToplevelService;

use crate::{services::ShellIpcState, shell::bar::dropdowns::DropdownRegistry};

pub(crate) struct WindowSwitcherInit {
    pub settings: BarSettings,
    pub service: Arc<WlrToplevelService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
    pub ipc_state: ShellIpcState,
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
    /// A Mod+Tab cycle step arrived over IPC: open the dropdown if it
    /// isn't already visible. Advancing the highlighted selection itself
    /// is handled inside the dropdown component, which watches the same
    /// IPC property independently.
    EnsureDropdownOpen,
}
