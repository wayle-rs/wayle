//! Reactive state for shell IPC.

use std::collections::HashSet;

use wayle_core::{Property, monitor::MonitorInfo};

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

    /// Identity of each monitor with a bar, used to resolve `desc:` and
    /// `sn:` selectors.
    pub monitors: Property<Vec<MonitorInfo>>,
}

impl ShellIpcState {
    pub(crate) fn new() -> Self {
        Self {
            hidden_bars: Property::new(HashSet::new()),
            connectors: Property::new(Vec::new()),
            monitors: Property::new(Vec::new()),
        }
    }
}
