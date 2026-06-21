//! Reactive state for shell IPC.

use std::collections::HashSet;

use wayle_core::Property;

/// Shared reactive state exposed to shell components via `ShellIpcService`.
///
/// Bar watchers subscribe to these properties to react to IPC commands.
#[derive(Clone)]
pub struct ShellIpcState {
    /// Connectors whose bars are currently hidden via CLI.
    pub hidden_bars: Property<HashSet<String>>,

    /// All active monitor connectors. Updated by the shell when bars are
    /// created or destroyed.
    pub connectors: Property<Vec<String>>,

    /// Incremented on each `wayle window cycle-step` call. The window
    /// switcher dropdown watches this to advance its highlighted selection
    /// (opening itself if not already visible). A counter rather than an
    /// `Option<()>` so consecutive identical calls each still notify
    /// watchers.
    pub window_cycle_step: Property<u64>,

    /// Incremented on each `wayle window cycle-commit` call. The window
    /// switcher dropdown watches this to activate the highlighted window
    /// and close itself.
    pub window_cycle_commit: Property<u64>,

    /// Incremented on each `wayle window cycle-cancel` call. The window
    /// switcher dropdown watches this to restore the window that was
    /// active before the cycle started and close itself.
    pub window_cycle_cancel: Property<u64>,
}

impl ShellIpcState {
    pub(crate) fn new() -> Self {
        Self {
            hidden_bars: Property::new(HashSet::new()),
            connectors: Property::new(Vec::new()),
            window_cycle_step: Property::new(0),
            window_cycle_commit: Property::new(0),
            window_cycle_cancel: Property::new(0),
        }
    }
}
