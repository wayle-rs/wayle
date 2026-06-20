use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_wlr_toplevel::WlrToplevelService;

use crate::services::ShellIpcState;

pub(crate) struct WindowSwitcherDropdownInit {
    pub service: Arc<WlrToplevelService>,
    pub config: Arc<ConfigService>,
    pub ipc_state: ShellIpcState,
}

/// Plain-data view of one window, used to populate `WindowRow`.
#[derive(Debug, Clone)]
pub(crate) struct WindowInfo {
    pub key: u32,
    pub title: String,
    pub app_id: String,
    pub is_active: bool,
    pub is_highlighted: bool,
}

#[derive(Debug)]
pub(crate) enum WindowSwitcherDropdownMsg {
    RowClicked(u32),
}

#[derive(Debug)]
pub(crate) enum WindowSwitcherDropdownCmd {
    ToplevelsChanged,
    ConfigChanged,
    CycleStep,
    CycleCommit,
}
