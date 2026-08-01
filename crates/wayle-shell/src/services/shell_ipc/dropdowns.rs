//! Dropdown enumeration + control domain logic.
//!
//! Backs `wayle dropdown list`/`toggle`/`open`/`close`. The identifiers are the
//! source of truth published by each live bar from its actual dropdown openers (see
//! the shell's `rebuild_dropdown_targets`), so this needs no config walk and no
//! central module→dropdown table — only the reactive [`ShellIpcState`].

use std::collections::HashSet;

use tracing::instrument;
use zbus::fdo;

use super::active_monitor::{ActiveMonitor, Target, resolve_request_monitor};
use super::state::{DropdownAction, DropdownRequest, ShellIpcState, push_nonced};

/// Enumeration + toggle/open/close of the dropdowns the live bars publish.
pub(crate) struct DropdownControl {
    state: ShellIpcState,
    active: ActiveMonitor,
}

impl DropdownControl {
    pub(crate) fn new(state: ShellIpcState, active: ActiveMonitor) -> Self {
        Self { state, active }
    }

    /// Every addressable dropdown identifier for `monitor`, resolved via the shared
    /// `--monitor` rules (`""` = the active monitor, `"all"` = all bars, else that
    /// connector). Within a bar the ids keep left-to-right layout order (the bar
    /// publishes them so); a multi-bar union is grouped by connector name (the ids map
    /// is a `BTreeMap`) and de-duplicated keeping first-seen order.
    ///
    /// An explicit unknown connector is a D-Bus error (as for the mutating paths), so a
    /// typo'd `--monitor` reports the bad name instead of silently listing nothing.
    pub(crate) fn list(&self, monitor: &str) -> fdo::Result<Vec<String>> {
        let (target, _) = self.resolve_scope(monitor)?;
        Ok(self.enumerate(&target))
    }

    /// Identifiers for a resolved [`Target`], drawn from the live published ids.
    /// Stale entries for connectors whose bar is gone are ignored.
    fn enumerate(&self, target: &Target) -> Vec<String> {
        let published = self.state.dropdown_ids.get();
        let active = self.state.connectors.get();

        let mut seen = HashSet::new();
        let mut ids = Vec::new();
        for (connector, connector_ids) in &published {
            if !active.iter().any(|c| c == connector) {
                continue;
            }
            let include = match target {
                Target::All => true,
                Target::One(one) => connector == one,
            };
            if include {
                for id in connector_ids {
                    if seen.insert(id.clone()) {
                        ids.push(id.clone());
                    }
                }
            }
        }
        ids
    }

    /// Toggle `identifier`'s dropdown on `monitor` (`wayle dropdown toggle`): open if
    /// closed, close if open, per bar.
    #[instrument(skip(self))]
    pub(crate) fn toggle(&self, monitor: &str, identifier: &str) -> fdo::Result<()> {
        self.act(monitor, identifier, DropdownAction::Toggle)
    }

    /// Open `identifier`'s dropdown on `monitor` (`wayle dropdown open`): open if
    /// closed, no-op if already open, per bar.
    #[instrument(skip(self))]
    pub(crate) fn open(&self, monitor: &str, identifier: &str) -> fdo::Result<()> {
        self.act(monitor, identifier, DropdownAction::Open)
    }

    /// Close whatever dropdown is open on `monitor` (`wayle dropdown close`); no-op
    /// per bar if none is open. No identifier is needed.
    #[instrument(skip(self))]
    pub(crate) fn close(&self, monitor: &str) -> fdo::Result<()> {
        let (_, request_monitor) = self.resolve_scope(monitor)?;
        self.publish(request_monitor, String::new(), DropdownAction::Close);
        Ok(())
    }

    /// Shared toggle/open path: resolve the scope, validate the identifier against
    /// the scoped published ids, then publish the request. An unknown identifier or
    /// an explicit unknown connector is a D-Bus error rather than a silent no-op; the
    /// action then happens on each matching bar via the reactive [`DropdownRequest`].
    fn act(&self, monitor: &str, identifier: &str, action: DropdownAction) -> fdo::Result<()> {
        let (target, request_monitor) = self.resolve_scope(monitor)?;
        if !self.enumerate(&target).iter().any(|known| known == identifier) {
            return Err(fdo::Error::InvalidArgs(format!(
                "unknown dropdown identifier '{identifier}'; run `wayle dropdown list`"
            )));
        }
        self.publish(request_monitor, identifier.to_owned(), action);
        Ok(())
    }

    /// Resolve `monitor` to its [`Target`] and the request's monitor string (empty
    /// for all bars, else the connector), erroring on an explicit unknown connector.
    fn resolve_scope(&self, monitor: &str) -> fdo::Result<(Target, String)> {
        let connectors = self.state.connectors.get();
        resolve_request_monitor(&self.active, &connectors, monitor)
    }

    /// Append a nonce'd [`DropdownRequest`] to the bounded log the bars drain. Every
    /// request gets its own strictly-increasing nonce and its own log entry, so a
    /// rapid burst is delivered in full (not coalesced) and each bar acts on it once.
    fn publish(&self, monitor: String, identifier: String, action: DropdownAction) {
        push_nonced(&self.state.dropdown_request, |nonce| DropdownRequest {
            nonce,
            monitor,
            identifier,
            action,
        });
    }
}
