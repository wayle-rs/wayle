//! Resolving the compositor's currently focused monitor for the `--monitor`
//! default.
//!
//! "Active monitor" is inherently a compositor concept — there is no portable
//! Wayland query for the focused output — so this reads on demand from whichever
//! optional WM integration is present (Hyprland, niri, or Mango; a session runs
//! exactly one). When no WM service is loaded, or the compositor reports no
//! focused output, resolution yields `None` and callers broadcast to all bars.

use std::sync::Arc;

use wayle_hyprland::HyprlandService;
use wayle_mango::MangoService;
use wayle_niri::NiriService;
use zbus::fdo;

/// On-demand resolver for the compositor's focused output connector.
///
/// Cheap to clone — holds only optional `Arc` service handles. Reading is
/// synchronous (a reactive snapshot), so it is always current at request time
/// with no watcher to keep in sync.
#[derive(Clone, Default)]
pub(crate) struct ActiveMonitor {
    hyprland: Option<Arc<HyprlandService>>,
    niri: Option<Arc<NiriService>>,
    mango: Option<Arc<MangoService>>,
}

impl ActiveMonitor {
    pub(crate) fn new(
        hyprland: Option<Arc<HyprlandService>>,
        niri: Option<Arc<NiriService>>,
        mango: Option<Arc<MangoService>>,
    ) -> Self {
        Self {
            hyprland,
            niri,
            mango,
        }
    }

    /// The connector of the compositor's focused output, if determinable.
    pub(crate) fn current(&self) -> Option<String> {
        if let Some(hyprland) = &self.hyprland {
            let focused = hyprland
                .monitors
                .get()
                .into_iter()
                .find(|monitor| monitor.focused.get())
                .map(|monitor| monitor.name.get());
            if focused.is_some() {
                return focused;
            }
        }

        if let Some(niri) = &self.niri {
            // Exactly one workspace across all outputs holds global focus; its
            // output is the focused monitor.
            let focused = niri
                .workspaces
                .get()
                .values()
                .find(|workspace| workspace.is_focused.get())
                .and_then(|workspace| workspace.output.get());
            if focused.is_some() {
                return focused;
            }
        }

        if let Some(mango) = &self.mango {
            let focused = mango
                .monitors
                .get()
                .into_iter()
                .find(|monitor| monitor.is_active)
                .map(|monitor| monitor.name);
            if focused.is_some() {
                return focused;
            }
        }

        None
    }
}

/// Which bars a `--monitor` argument targets.
pub(crate) enum Target {
    /// Broadcast to every active bar.
    All,
    /// A single connector.
    One(String),
}

/// Resolve a wire `monitor` argument to concrete bar target(s):
///
/// - `"all"` → [`Target::All`].
/// - `""` (the no-flag default) → the compositor's active monitor when it has a
///   bar, else [`Target::All`] (the broadcast fallback).
/// - any other value → [`Target::One`] for that connector (the caller validates
///   it names an active bar).
pub(crate) fn resolve_target(
    active: &ActiveMonitor,
    connectors: &[String],
    monitor: &str,
) -> Target {
    if monitor == "all" {
        return Target::All;
    }

    if monitor.is_empty() {
        return match active.current() {
            Some(connector) if connectors.iter().any(|c| c == &connector) => Target::One(connector),
            _ => Target::All,
        };
    }

    Target::One(monitor.to_owned())
}

/// Resolve a wire `monitor` argument to its [`Target`] *and* the request's monitor
/// string (empty for a broadcast, else the connector), erroring on an explicit unknown
/// connector with the active-connector list. Shared by the dropdown and systray control
/// paths so their `--monitor` handling and error wording can't drift.
pub(crate) fn resolve_request_monitor(
    active: &ActiveMonitor,
    connectors: &[String],
    monitor: &str,
) -> fdo::Result<(Target, String)> {
    let target = resolve_target(active, connectors, monitor);
    let request_monitor = match &target {
        Target::All => String::new(),
        Target::One(connector) => {
            if !connectors.iter().any(|c| c == connector) {
                return Err(fdo::Error::InvalidArgs(format!(
                    "unknown monitor connector '{connector}' (active: {})",
                    if connectors.is_empty() {
                        "none".to_owned()
                    } else {
                        connectors.join(", ")
                    }
                )));
            }
            connector.clone()
        }
    };
    Ok((target, request_monitor))
}
