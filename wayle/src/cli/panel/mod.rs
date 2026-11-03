//! Panel management commands.

/// Panel command definitions.
pub mod commands;
mod hide;
mod inspect;
mod monitors;
mod proxy;
/// Restart command.
pub mod restart;
/// Settings command.
pub mod settings;
mod show;
/// Start command.
pub mod start;
/// Status command.
pub mod status;
/// Stop command.
pub mod stop;
mod toggle;

use commands::PanelCommands;

use crate::cli::CliAction;

/// Executes panel management commands.
///
/// # Errors
///
/// Returns error if the command execution fails.
pub async fn execute(command: PanelCommands) -> CliAction {
    match command {
        PanelCommands::Start => start::execute().await,
        PanelCommands::Stop => stop::execute().await,
        PanelCommands::Restart => restart::execute().await,
        PanelCommands::Status => status::execute().await,
        PanelCommands::Settings => settings::execute().await,
        PanelCommands::Inspect => inspect::execute().await,
        PanelCommands::Hide { monitor } => hide::execute(monitor).await,
        PanelCommands::Show { monitor } => show::execute(monitor).await,
        PanelCommands::Toggle { monitor } => toggle::execute(monitor).await,
        PanelCommands::Monitors => monitors::execute().await,
    }
}
