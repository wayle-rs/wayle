//! Panel start command.

use std::{
    env,
    io::ErrorKind,
    process::{Command, Stdio},
};

use tracing::info;

use super::proxy::is_running;
use crate::cli::CliAction;

/// Starts the Wayle GUI panel as a detached daemon.
///
/// If the panel is already running, reports that and returns success.
///
/// # Errors
///
/// Returns error if the binary cannot be found or executed.
pub async fn execute() -> CliAction {
    if is_running().await.unwrap_or(false) {
        println!("Panel is already running");
        return Ok(());
    }

    info!("Starting Wayle panel");

    let current_exe =
        env::current_exe().map_err(|err| format!("Failed to resolve executable: {err}"))?;

    Command::new(current_exe)
        .arg("shell")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| match err.kind() {
            ErrorKind::PermissionDenied => "Permission denied when starting panel".to_string(),
            _ => format!("Failed to start panel: {err}"),
        })?;

    println!("Panel started");
    Ok(())
}
