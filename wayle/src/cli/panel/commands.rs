use clap::Subcommand;

/// Panel management subcommands.
#[derive(Subcommand, Debug)]
pub enum PanelCommands {
    /// Start the panel daemon
    Start,

    /// Stop the panel daemon
    Stop,

    /// Restart the panel daemon
    Restart,

    /// Check panel status
    Status,

    /// Open panel settings
    Settings,

    /// Open GTK Inspector for debugging
    Inspect,

    /// Hide the bar on a monitor
    Hide {
        /// Connector name ("DP-1"), "desc:<prefix>" or "sn:<serial>". Omit to hide all.
        monitor: Option<String>,
    },

    /// Show the bar on a monitor
    Show {
        /// Connector name ("DP-1"), "desc:<prefix>" or "sn:<serial>". Omit to show all.
        monitor: Option<String>,
    },

    /// Toggle bar visibility on a monitor
    Toggle {
        /// Connector name ("DP-1"), "desc:<prefix>" or "sn:<serial>". Omit to toggle all.
        monitor: Option<String>,
    },

    /// List monitors with selectors usable in config
    Monitors,
}
