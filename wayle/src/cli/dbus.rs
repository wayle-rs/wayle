//! Shared D-Bus utilities for CLI commands.

use wayle_ipc::shell_ipc::ShellIpcProxy;
use zbus::{Connection, Error as ZbusError, fdo::Error as FdoError};

/// Establishes a D-Bus session connection.
///
/// # Errors
/// Returns error if the session bus is unavailable.
pub async fn session() -> Result<Connection, String> {
    Connection::session()
        .await
        .map_err(|e| format!("Failed to connect to D-Bus session bus: {e}"))
}

/// Creates a `ShellIpcProxy` (`com.wayle.Shell1`) for shell commands (bar
/// visibility, dropdown list/toggle, systray open, …).
///
/// # Errors
/// Returns error if the D-Bus connection or proxy creation fails.
pub async fn shell_ipc_proxy() -> Result<(Connection, ShellIpcProxy<'static>), String> {
    let connection = session().await?;

    let proxy = ShellIpcProxy::new(&connection)
        .await
        .map_err(|err| format!("cannot create shell IPC proxy: {err}"))?;

    Ok((connection, proxy))
}

/// Formats a shell IPC (`com.wayle.Shell1`) D-Bus error for CLI output.
pub fn format_ipc_error(operation: &str, error: ZbusError) -> String {
    format_error("Shell", operation, error)
}

/// Formats D-Bus errors into user-friendly messages.
///
/// Provides helpful guidance for common issues like services not running.
pub fn format_error(service_name: &str, operation: &str, error: ZbusError) -> String {
    match &error {
        ZbusError::FDO(fdo) => match fdo.as_ref() {
            FdoError::ServiceUnknown(_) | FdoError::NameHasNoOwner(_) => {
                format!("{service_name} service not running. Start wayle shell first.")
            }
            FdoError::NoReply(_) | FdoError::Timeout(_) => {
                format!("{operation} timed out - service not responding")
            }
            _ => format!("Failed to {operation}: {error}"),
        },
        ZbusError::MethodError(name, msg, _) => {
            if name.as_str().contains("ServiceUnknown") {
                format!("{service_name} service not running. Start wayle shell first.")
            } else {
                format!(
                    "Failed to {operation}: {}",
                    msg.as_deref().unwrap_or(name.as_str())
                )
            }
        }
        _ => format!("Failed to {operation}: {error}"),
    }
}
