use clap::Subcommand;

/// Bar dropdown inspection and control subcommands.
#[derive(Subcommand, Debug)]
pub enum DropdownCommands {
    /// List addressable dropdown identifiers (e.g. `calendar@clock`)
    List {
        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },

    /// Toggle a dropdown by identifier (from `list`), e.g. `calendar@clock`
    Toggle {
        /// Dropdown identifier to toggle.
        identifier: String,

        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },

    /// Open a dropdown by identifier; no-op if it's already open
    Open {
        /// Dropdown identifier to open.
        identifier: String,

        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },

    /// Close whatever dropdown is open; no-op if none is open
    Close {
        /// Target monitor: omit for the active monitor, a connector name
        /// (e.g. "DP-1"), or "all" for every monitor.
        #[arg(long, value_name = "MONITOR")]
        monitor: Option<String>,
    },
}
