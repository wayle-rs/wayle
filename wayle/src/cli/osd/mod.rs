/// OSD command definitions.
pub mod commands;
mod devices;
mod proxy;
mod show;

use commands::OsdCommands;

use super::CliAction;

/// Executes on-screen display commands.
///
/// # Errors
/// Returns error if the command execution fails.
pub async fn execute(command: OsdCommands) -> CliAction {
    match command {
        OsdCommands::Speaker { device } => show::speaker(device.as_deref()).await,
        OsdCommands::Mic { device } => show::microphone(device.as_deref()).await,
        OsdCommands::Brightness { device } => show::brightness(device.as_deref()).await,
        OsdCommands::Devices => devices::execute().await,
    }
}
