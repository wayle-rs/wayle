//! Reactive state for shell IPC.

use std::collections::{BTreeMap, HashSet};

use wayle_core::Property;

/// What a dropdown request asks the bars to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropdownAction {
    /// Open if closed, close if open (`wayle dropdown toggle`).
    #[default]
    Toggle,
    /// Open if not already open; no-op if it is (`wayle dropdown open`).
    Open,
    /// Close whatever dropdown is open; no-op if none (`wayle dropdown close`).
    /// The `identifier` is unused.
    Close,
}

/// Upper bound on the buffered request logs ([`ShellIpcState::dropdown_request`] /
/// [`ShellIpcState::systray_menu_request`]). Requests are human-driven and drained
/// every GTK frame, so the log is normally a handful of entries; this only caps
/// pathological bursts. The oldest entries are dropped once the cap is exceeded.
pub const REQUEST_LOG_CAP: usize = 64;

/// A pending dropdown request pushed from the CLI to the bars.
///
/// Requests accumulate in a bounded log (see [`ShellIpcState::dropdown_request`]) with
/// a strictly increasing `nonce`; each bar tracks the last nonce it has processed, so
/// it acts on every new request exactly once — no coalescing of a rapid burst, and no
/// replay of already-issued requests when a bar is (re)created. `nonce` starts at 1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DropdownRequest {
    /// Strictly increasing per request; used by bars as a de-dup/skip cursor.
    pub nonce: u64,
    /// Target connector; empty means all bars.
    pub monitor: String,
    /// The dropdown identifier to act on (unused for [`DropdownAction::Close`]).
    pub identifier: String,
    /// What to do with the dropdown.
    pub action: DropdownAction,
}

/// What a systray-menu request asks the bars to do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SystrayMenuAction {
    /// Open if closed, close if open (`wayle systray toggle`).
    #[default]
    Toggle,
    /// Open if not already open; no-op if it is (`wayle systray open`).
    Open,
}

/// A pending tray-menu request pushed from the CLI to the bars (`wayle systray
/// toggle`/`open <id>`). Buffered and cursor-tracked exactly like [`DropdownRequest`]
/// (see [`ShellIpcState::systray_menu_request`]); `nonce` starts at 1.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystrayMenuRequest {
    /// Strictly increasing per request; used by bars as a de-dup/skip cursor.
    pub nonce: u64,
    /// Target connector; empty means all bars.
    pub monitor: String,
    /// The tray item id whose menu to act on.
    pub id: String,
    /// What to do with the menu.
    pub action: SystrayMenuAction,
}

/// A buffered CLI request that carries a strictly-increasing de-dup/skip `nonce`.
pub(crate) trait Nonced {
    fn nonce(&self) -> u64;
}

impl Nonced for DropdownRequest {
    fn nonce(&self) -> u64 {
        self.nonce
    }
}

impl Nonced for SystrayMenuRequest {
    fn nonce(&self) -> u64 {
        self.nonce
    }
}

/// Append a request to a bounded reactive log: `build` receives the next nonce (one
/// past the newest entry, so nonces start at 1 and strictly increase), the result is
/// pushed, and the oldest entries beyond [`REQUEST_LOG_CAP`] are dropped. The single
/// home of the nonce-monotonicity + cap invariant the per-bar cursor drains rely on.
pub(crate) fn push_nonced<T>(log: &Property<Vec<T>>, build: impl FnOnce(u64) -> T)
where
    T: Nonced + Clone + Send + Sync + PartialEq + 'static,
{
    let mut entries = log.get();
    let nonce = entries.last().map_or(0, Nonced::nonce).wrapping_add(1);
    entries.push(build(nonce));
    let overflow = entries.len().saturating_sub(REQUEST_LOG_CAP);
    if overflow > 0 {
        entries.drain(0..overflow);
    }
    log.set(entries);
}

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

    /// Bounded log of recent `dropdown` requests (toggle/open/close), newest last.
    /// Each bar drains entries newer than its cursor and acts on those whose target
    /// monitor matches (or is empty, meaning all bars). A log — not a single value —
    /// so a rapid burst isn't coalesced and a (re)created bar doesn't replay old ones.
    pub dropdown_request: Property<Vec<DropdownRequest>>,

    /// Bounded log of recent `systray toggle`/`open` requests, newest last. Drained
    /// per-bar via a cursor exactly like [`Self::dropdown_request`].
    pub systray_menu_request: Property<Vec<SystrayMenuRequest>>,

    /// Live addressable dropdown identifiers per connector, published by each bar
    /// from its actual dropdown openers. This is the source of truth for
    /// `wayle dropdown list` — no config walk, no central module→dropdown table.
    pub dropdown_ids: Property<BTreeMap<String, Vec<String>>>,
}

impl ShellIpcState {
    pub(crate) fn new() -> Self {
        Self {
            hidden_bars: Property::new(HashSet::new()),
            connectors: Property::new(Vec::new()),
            dropdown_request: Property::new(Vec::new()),
            systray_menu_request: Property::new(Vec::new()),
            dropdown_ids: Property::new(BTreeMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_nonced_assigns_increasing_nonces_and_caps_the_log() {
        let log: Property<Vec<DropdownRequest>> = Property::new(Vec::new());
        let total = REQUEST_LOG_CAP + 5;

        for _ in 0..total {
            push_nonced(&log, |nonce| DropdownRequest {
                nonce,
                ..Default::default()
            });
        }

        let entries = log.get();
        // Capped to the most recent REQUEST_LOG_CAP entries (oldest front-dropped)...
        assert_eq!(entries.len(), REQUEST_LOG_CAP);
        assert_eq!(
            entries.first().unwrap().nonce,
            (total - REQUEST_LOG_CAP + 1) as u64
        );
        assert_eq!(entries.last().unwrap().nonce, total as u64);
        // ...and every retained nonce is exactly the previous + 1.
        assert!(entries.windows(2).all(|w| w[1].nonce == w[0].nonce + 1));
    }
}
