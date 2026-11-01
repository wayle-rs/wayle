/// Icon command definitions
pub mod commands;
/// Export installed icons
pub mod export;
/// Import local SVG files
pub mod import;
/// Install icons from CDN
pub mod install;
/// List installed icons
pub mod list;
/// Open icons directory
pub mod open;
/// Remove installed icons
pub mod remove;
/// Install bundled icons
pub mod setup;
/// List available icon sources
pub mod sources;
/// Install icons referenced by config but not yet on disk
pub mod sync;

use commands::IconsCommands;

use super::CliAction;

/// Executes icon management commands.
///
/// # Errors
///
/// Returns error if the command execution fails.
pub async fn execute(command: IconsCommands) -> CliAction {
    match command {
        IconsCommands::Setup => setup::execute(),
        IconsCommands::Install { source, slugs } => install::execute(source, slugs).await,
        IconsCommands::Import { path, name } => import::execute(path, name),
        IconsCommands::Remove { names } => remove::execute(names),
        IconsCommands::Sources => sources::execute(),
        IconsCommands::List {
            source,
            interactive,
        } => list::execute(source, interactive),
        IconsCommands::Open => open::execute(),
        IconsCommands::Export { destination } => export::execute(destination),
        IconsCommands::Sync { dry_run } => sync::execute(dry_run).await,
    }
}
