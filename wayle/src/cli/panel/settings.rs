//! Panel settings command.

use std::{
    io::ErrorKind,
    process::{Command, Stdio},
};

use tracing::info;

use crate::cli::CliAction;

/// Resolves the `wayle-settings` binary next to the running `wayle`
/// executable, falling back to `$PATH` when that lookup fails.
///
/// Prevents a custom/dev build of `wayle` from silently launching an
/// unrelated, possibly schema-incompatible `wayle-settings` found earlier on
/// `$PATH` (e.g. a system package), which can corrupt `runtime.toml` when the
/// two builds disagree on the config schema.
fn settings_binary_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("wayle-settings")))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| std::path::PathBuf::from("wayle-settings"))
}

/// Launches the settings application.
///
/// # Errors
///
/// Returns error if settings application cannot be launched.
pub async fn execute() -> CliAction {
    info!("Launching Wayle settings");

    Command::new(settings_binary_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => {
                "wayle-settings not found. Is Wayle installed correctly?".to_string()
            }
            ErrorKind::PermissionDenied => {
                "Permission denied when starting wayle-settings".to_string()
            }
            _ => format!("Failed to launch settings: {e}"),
        })?;

    Ok(())
}
