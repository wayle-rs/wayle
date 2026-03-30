use std::{collections::HashMap, sync::Arc};

use wayle_config::ConfigService;
use wayle_hyprland::{Address, HyprlandService, WorkspaceId};
use wayle_widgets::prelude::BarSettings;

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
    // --- Preview lifecycle ---
    /// Pointer entered a workspace button.
    ButtonHoverEnter(WorkspaceId),
    /// Pointer left a workspace button.
    ButtonHoverLeave(WorkspaceId),
    /// Pointer entered the preview popover content.
    PopoverEnter,
    /// Pointer left the preview popover content.
    PopoverLeave,
    /// Dwell timer fired — open preview for this workspace.
    PreviewDwellFired(WorkspaceId),
    /// Close timer fired — close the popover if pointer is gone.
    PreviewCloseTimerFired,
    /// Immediate show (right-click path, no dwell).
    WorkspacePreviewRequest(WorkspaceId),
    /// Popover's `connect_closed` signal — cleanup.
    WorkspacePreviewClosed,
    /// Preview component requested close (e.g., window clicked).
    WorkspacePreviewDismiss,
    FocusWindow(String),
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
