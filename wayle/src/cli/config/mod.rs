/// Configuration command definitions
pub mod commands;
/// Default configuration output command
pub mod default;
/// Docs generator command
pub mod docs;
/// Get configuration value command
pub mod get;
/// Reset configuration value command
pub mod reset;
/// JSON Schema output command
pub mod schema;
/// Set configuration value command
pub mod set;

use commands::ConfigCommands;

use super::CliAction;

/// Executes configuration management commands.
///
/// # Errors
/// Returns error if the command execution fails.
pub async fn execute(command: ConfigCommands) -> CliAction {
    match command {
        ConfigCommands::Get { path } => get::execute(path).await,
        ConfigCommands::Set { path, value } => set::execute(path, value).await,
        ConfigCommands::Reset { path } => reset::execute(path).await,
        ConfigCommands::Schema { stdout } => schema::execute(stdout),
        ConfigCommands::Default { stdout } => default::execute(stdout),
        ConfigCommands::Docs { out, only } => docs::execute(out, only),
    }
}
