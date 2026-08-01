/// Activate tray item command
pub mod activate;
/// System tray command definitions
pub mod commands;
/// List tray items command
pub mod list;
/// Open tray item menu command
pub mod open;
mod proxy;
/// Status command
pub mod status;
/// Toggle tray item menu command
pub mod toggle;

use commands::SystrayCommands;

use super::CliAction;

/// Executes system tray control commands.
///
/// # Errors
/// Returns error if the command execution fails.
pub async fn execute(command: SystrayCommands) -> CliAction {
    match command {
        SystrayCommands::List => list::execute().await,
        SystrayCommands::Activate { id } => activate::execute(id).await,
        SystrayCommands::Toggle { id, monitor } => toggle::execute(id, monitor).await,
        SystrayCommands::Open { id, monitor } => open::execute(id, monitor).await,
        SystrayCommands::Status => status::execute().await,
    }
}
