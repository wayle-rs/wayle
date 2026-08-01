//! D-Bus interface adapter for shell IPC.

use zbus::{fdo, interface};

use super::{
    active_monitor::ActiveMonitor, bar::BarVisibility, dropdowns::DropdownControl,
    state::ShellIpcState, systray::SystrayControl,
};

/// D-Bus daemon that dispatches shell commands to domain handlers.
pub(crate) struct ShellIpcDaemon {
    bar: BarVisibility,
    dropdowns: DropdownControl,
    systray: SystrayControl,
    state: ShellIpcState,
}

impl ShellIpcDaemon {
    pub(crate) fn new(state: ShellIpcState, active: ActiveMonitor) -> Self {
        Self {
            bar: BarVisibility::new(state.clone()),
            dropdowns: DropdownControl::new(state.clone(), active.clone()),
            systray: SystrayControl::new(state.clone(), active),
            state,
        }
    }
}

#[interface(name = "com.wayle.Shell1")]
impl ShellIpcDaemon {
    /// Hides the bar on a monitor. Empty string hides all bars.
    pub async fn bar_hide(&self, monitor: &str) {
        self.bar.hide(monitor);
    }

    /// Shows the bar on a monitor. Empty string shows all bars.
    pub async fn bar_show(&self, monitor: &str) {
        self.bar.show(monitor);
    }

    /// Toggles bar visibility on a monitor. Empty string toggles all.
    pub async fn bar_toggle(&self, monitor: &str) -> fdo::Result<()> {
        self.bar.toggle(monitor)
    }

    /// Addressable dropdown identifiers for a monitor. Empty string targets the active
    /// monitor's bar (falling back to a union of all bars when it has none); `"all"`
    /// unions across all bars; an explicit unknown connector is an error.
    pub async fn dropdown_list(&self, monitor: &str) -> fdo::Result<Vec<String>> {
        self.dropdowns.list(monitor)
    }

    /// Toggles a dropdown by identifier on a monitor. Empty string targets all
    /// bars. Errors if the identifier is unknown.
    pub async fn dropdown_toggle(&self, monitor: &str, identifier: &str) -> fdo::Result<()> {
        self.dropdowns.toggle(monitor, identifier)
    }

    /// Opens a dropdown by identifier on a monitor (empty = all bars); no-op on a
    /// bar where it is already open. Errors if the identifier is unknown.
    pub async fn dropdown_open(&self, monitor: &str, identifier: &str) -> fdo::Result<()> {
        self.dropdowns.open(monitor, identifier)
    }

    /// Closes whatever dropdown is open on a monitor (empty = all bars); no-op where
    /// none is open.
    pub async fn dropdown_close(&self, monitor: &str) -> fdo::Result<()> {
        self.dropdowns.close(monitor)
    }

    /// Toggles a tray item's menu by id on a monitor (empty = all bars): open if
    /// closed, close if open. Best-effort: an id not present on a bar is a no-op.
    pub async fn systray_toggle(&self, id: &str, monitor: &str) -> fdo::Result<()> {
        self.systray.toggle(id, monitor)
    }

    /// Opens a tray item's menu by id on a monitor (empty = all bars); no-op on a
    /// bar where it is already open. Best-effort: an id not present is a no-op.
    pub async fn systray_open(&self, id: &str, monitor: &str) -> fdo::Result<()> {
        self.systray.open(id, monitor)
    }

    /// Currently hidden monitor connectors.
    #[zbus(property)]
    pub async fn bar_hidden(&self) -> Vec<String> {
        let mut result: Vec<String> = self.state.hidden_bars.get().into_iter().collect();
        result.sort();
        result
    }

    /// All active monitor connectors.
    #[zbus(property)]
    pub async fn connectors(&self) -> Vec<String> {
        self.state.connectors.get()
    }
}
