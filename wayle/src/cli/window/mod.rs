//! Window switcher control commands.

/// Window subcommand definitions.
pub mod commands;
mod cycle_commit;
mod cycle_step;

use commands::WindowCommands;

use crate::cli::CliAction;

/// Executes window switcher control commands.
///
/// # Errors
///
/// Returns error if the command execution fails.
pub async fn execute(command: WindowCommands) -> CliAction {
    match command {
        WindowCommands::CycleStep => cycle_step::execute().await,
        WindowCommands::CycleCommit => cycle_commit::execute().await,
    }
}
