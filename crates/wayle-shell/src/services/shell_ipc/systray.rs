//! Systray control domain logic.
//!
//! Backs `wayle systray toggle`/`open <id>`: publishes a nonce'd request that each
//! bar's systray module watches, toggling or opening the matching item's menu.
//! Best-effort — an id not present on a given bar is a no-op there (tray items live
//! per-bar in the shell, not in this daemon, so there is nothing to validate here).

use tracing::instrument;
use zbus::fdo;

use super::active_monitor::{ActiveMonitor, resolve_request_monitor};
use super::state::{ShellIpcState, SystrayMenuAction, SystrayMenuRequest, push_nonced};

/// Publishes tray-menu toggle/open requests to the bars.
pub(crate) struct SystrayControl {
    state: ShellIpcState,
    active: ActiveMonitor,
}

impl SystrayControl {
    pub(crate) fn new(state: ShellIpcState, active: ActiveMonitor) -> Self {
        Self { state, active }
    }

    /// Toggle the tray item `id`'s menu on `monitor` (`wayle systray toggle`): open
    /// if closed, close if open, per bar.
    #[instrument(skip(self))]
    pub(crate) fn toggle(&self, id: &str, monitor: &str) -> fdo::Result<()> {
        self.act(id, monitor, SystrayMenuAction::Toggle)
    }

    /// Open the tray item `id`'s menu on `monitor` (`wayle systray open`): open if
    /// closed, no-op if already open, per bar.
    #[instrument(skip(self))]
    pub(crate) fn open(&self, id: &str, monitor: &str) -> fdo::Result<()> {
        self.act(id, monitor, SystrayMenuAction::Open)
    }

    /// Resolve the `--monitor` scope (`""` = active monitor, `"all"` = all bars, else
    /// that connector; error on an explicit unknown connector), then publish a
    /// nonce'd request. Best-effort per bar (an id not present there is a no-op).
    fn act(&self, id: &str, monitor: &str, action: SystrayMenuAction) -> fdo::Result<()> {
        let connectors = self.state.connectors.get();
        let (_, request_monitor) = resolve_request_monitor(&self.active, &connectors, monitor)?;

        push_nonced(&self.state.systray_menu_request, |nonce| SystrayMenuRequest {
            nonce,
            monitor: request_monitor,
            id: id.to_owned(),
            action,
        });
        Ok(())
    }
}
