//! Bar dropdown inspection and control commands.

/// Dropdown command definitions.
pub mod commands;
mod close;
mod list;
mod open;
mod toggle;

use commands::DropdownCommands;

use crate::cli::CliAction;

/// Executes bar dropdown commands.
///
/// # Errors
///
/// Returns error if the command execution fails.
pub async fn execute(command: DropdownCommands) -> CliAction {
    match command {
        DropdownCommands::List { monitor } => list::execute(monitor).await,
        DropdownCommands::Toggle {
            identifier,
            monitor,
        } => toggle::execute(identifier, monitor).await,
        DropdownCommands::Open {
            identifier,
            monitor,
        } => open::execute(identifier, monitor).await,
        DropdownCommands::Close { monitor } => close::execute(monitor).await,
    }
}
