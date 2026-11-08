use std::{collections::HashMap, sync::Arc};

use wayle_config::ConfigService;
use wayle_hyprland::{Address, HyprlandService, WorkspaceAddress, WorkspaceId};
use wayle_widgets::prelude::BarSettings;

use super::filtering::ActiveWorkspace;

pub(crate) struct WorkspacesInit {
    pub settings: BarSettings,
    pub hyprland: Option<Arc<HyprlandService>>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum WorkspacesMsg {
    WorkspaceClicked(WorkspaceAddress),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum WorkspacesCmd {
    WorkspacesChanged,
    ClientsChanged,
    ActiveWorkspaceChanged(ActiveWorkspace),
    MonitorFocused {
        monitor: String,
        workspace: Option<ActiveWorkspace>,
    },
    TitleChanged,
    ConfigChanged,
    HyprlandConfigReloaded,
    UrgentWindow(Address),
    WindowFocused(Address),
    BlinkTick,
    WorkspaceRulesLoaded(HashMap<WorkspaceId, String>),
}
