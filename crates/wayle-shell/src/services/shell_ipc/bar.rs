//! Bar visibility domain logic.

use std::collections::HashSet;

use tracing::{instrument, warn};
use wayle_core::monitor::MonitorSelector;
use zbus::fdo;

use super::state::ShellIpcState;

/// Bar visibility logic. Resolves monitor arguments against known monitors
/// before mutating [`ShellIpcState::hidden_bars`].
pub(crate) struct BarVisibility {
    state: ShellIpcState,
}

impl BarVisibility {
    pub(crate) fn new(state: ShellIpcState) -> Self {
        Self { state }
    }

    /// Connectors addressed by `monitor`, a connector name or selector.
    fn resolve(&self, monitor: &str) -> Vec<String> {
        let selector = match monitor.parse::<MonitorSelector>() {
            Ok(selector) => selector,
            Err(err) => {
                warn!(monitor, error = %err, "invalid monitor, ignoring");
                return Vec::new();
            }
        };

        let connectors: Vec<String> = self
            .state
            .monitors
            .get()
            .into_iter()
            .filter(|info| selector.matches(info))
            .map(|info| info.connector)
            .collect();

        if connectors.is_empty() {
            warn!(monitor, "no monitor matches, ignoring");
        }
        connectors
    }

    #[instrument(skip(self), fields(monitor))]
    pub(crate) fn hide(&self, monitor: &str) {
        let mut set = self.state.hidden_bars.get();

        if monitor.is_empty() {
            set.extend(self.state.connectors.get());
        } else {
            let targets = self.resolve(monitor);
            if targets.is_empty() {
                return;
            }
            set.extend(targets);
        }

        self.state.hidden_bars.set(set);
    }

    #[instrument(skip(self), fields(monitor))]
    pub(crate) fn show(&self, monitor: &str) {
        if monitor.is_empty() {
            self.state.hidden_bars.set(HashSet::new());
            return;
        }

        let mut set = self.state.hidden_bars.get();
        for connector in self.resolve(monitor) {
            set.remove(&connector);
        }
        self.state.hidden_bars.set(set);
    }

    /// Shows the targets if all are hidden, otherwise hides them.
    #[instrument(skip(self), fields(monitor))]
    pub(crate) fn toggle(&self, monitor: &str) -> fdo::Result<()> {
        if monitor.is_empty() {
            return self.toggle_all();
        }

        let targets = self.resolve(monitor);
        if targets.is_empty() {
            return Ok(());
        }

        let mut set = self.state.hidden_bars.get();

        if targets.iter().all(|connector| set.contains(connector)) {
            for connector in &targets {
                set.remove(connector);
            }
        } else {
            set.extend(targets);
        }

        self.state.hidden_bars.set(set);
        Ok(())
    }

    fn toggle_all(&self) -> fdo::Result<()> {
        let set = self.state.hidden_bars.get();

        let hidden_bars = if set.is_empty() {
            HashSet::from_iter(self.state.connectors.get())
        } else {
            HashSet::new()
        };

        self.state.hidden_bars.set(hidden_bars);

        Ok(())
    }
}
