use std::{collections::HashMap, sync::Arc};

use wayle_config::ConfigService;
use wayle_hyprland::{Address, HyprlandService};
use wayle_widgets::prelude::BarSettings;

use super::WorkspaceId;

pub(crate) struct WorkspacesInit {
    pub settings: BarSettings,
    pub hyprland: Option<Arc<HyprlandService>>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum WorkspacesMsg {
    WorkspaceClicked(WorkspaceId),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum WorkspacesCmd {
    WorkspacesChanged,
    ClientsChanged,
    ActiveWorkspaceChanged(WorkspaceId),
    MonitorFocused {
        monitor: String,
        workspace_id: WorkspaceId,
    },
    TitleChanged,
    ConfigChanged,
    HyprlandConfigReloaded,
    UrgentWindow(Address),
    WindowFocused(Address),
    BlinkTick,
    WorkspaceRulesLoaded(HashMap<WorkspaceId, String>),
}
