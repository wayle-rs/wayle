//! D-Bus proxy utilities for OSD commands.

use wayle_ipc::shell_ipc::ShellIpcProxy;
use zbus::{Connection, Error as ZbusError};

use crate::cli::dbus;

/// Creates a ShellIpcProxy for OSD commands.
///
/// # Errors
/// Returns error if D-Bus connection or proxy creation fails.
pub async fn connect() -> Result<(Connection, ShellIpcProxy<'static>), String> {
    dbus::shell_ipc_proxy().await
}

/// Transforms zbus errors into user-friendly messages.
pub fn format_error(operation: &str, error: ZbusError) -> String {
    dbus::format_ipc_error(operation, error)
}
