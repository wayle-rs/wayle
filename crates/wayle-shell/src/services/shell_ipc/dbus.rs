//! D-Bus interface adapter for shell IPC.

use zbus::{fdo, interface};

use super::{bar::BarVisibility, state::ShellIpcState};

/// D-Bus daemon that dispatches shell commands to domain handlers.
pub(crate) struct ShellIpcDaemon {
    bar: BarVisibility,
    state: ShellIpcState,
}

impl ShellIpcDaemon {
    pub(crate) fn new(state: ShellIpcState) -> Self {
        Self {
            bar: BarVisibility::new(state.clone()),
            state,
        }
    }
}

#[interface(name = "com.wayle.Shell1")]
impl ShellIpcDaemon {
    /// Hides the bar on monitors matching a connector name or `desc:`/`sn:`
    /// selector. Empty string hides all bars.
    pub async fn bar_hide(&self, monitor: &str) {
        self.bar.hide(monitor);
    }

    /// Shows the bar on monitors matching a connector name or selector.
    /// Empty string shows all bars.
    pub async fn bar_show(&self, monitor: &str) {
        self.bar.show(monitor);
    }

    /// Toggles bar visibility on monitors matching a connector name or
    /// selector. Empty string toggles all.
    pub async fn bar_toggle(&self, monitor: &str) -> fdo::Result<()> {
        self.bar.toggle(monitor)
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

    /// Active monitors as `(connector, description, serial)`. Serial is
    /// empty when the monitor has none.
    #[zbus(property)]
    pub async fn monitors(&self) -> Vec<(String, String, String)> {
        self.state
            .monitors
            .get()
            .into_iter()
            .map(|monitor| {
                let description = monitor.description();
                (
                    monitor.connector,
                    description,
                    monitor.serial.unwrap_or_default(),
                )
            })
            .collect()
    }
}
