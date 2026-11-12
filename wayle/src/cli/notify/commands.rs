use clap::Subcommand;

/// Notification control subcommands.
#[derive(Subcommand, Debug)]
pub enum NotifyCommands {
    /// List all notifications
    List,

    /// Dismiss a notification by ID
    Dismiss {
        /// Notification ID to dismiss
        #[arg(value_name = "ID")]
        id: i64,
    },

    /// Dismiss all notifications
    DismissAll,

    /// Toggle Do Not Disturb mode
    Dnd,

    /// Show notification status
    Status,
}
