use clap::Subcommand;

/// System tray control subcommands.
#[derive(Subcommand, Debug)]
pub enum SystrayCommands {
    /// List all system tray items
    List,

    /// Activate a tray item by ID
    Activate {
        /// Tray item ID to activate
        #[arg(value_name = "ID")]
        id: String,
    },

    /// Toggle a tray item's right-click menu by ID (open if closed, close if open)
    Toggle {
        /// Tray item ID whose menu to toggle
        #[arg(value_name = "ID")]
        id: String,

        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },

    /// Open a tray item's right-click menu by ID; no-op if it's already open
    Open {
        /// Tray item ID whose menu to open
        #[arg(value_name = "ID")]
        id: String,

        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },

    /// Show system tray status
    Status,
}
