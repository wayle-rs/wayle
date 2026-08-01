use crate::cli::{
    CliAction,
    dbus::{format_ipc_error, shell_ipc_proxy},
};

/// Executes the toggle command: toggles a tray item's menu by ID (open if closed,
/// close if open).
///
/// Unlike `activate`/`list` (which talk to the systray service), toggling the menu
/// is a shell UI action, so this goes through the shell IPC (`com.wayle.Shell1`).
/// Best-effort per bar: an id not currently present is a no-op. The `monitor`
/// argument follows the shared rules (omit = active monitor, a connector name, or
/// "all").
///
/// # Errors
/// Returns error if D-Bus communication fails.
pub async fn execute(id: String, monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .systray_toggle(&id, monitor)
        .await
        .map_err(|err| format_ipc_error("toggle tray menu", err))?;

    match monitor {
        "" => println!("Toggled tray menu: {id} on the active monitor"),
        "all" => println!("Toggled tray menu: {id} on all monitors"),
        name => println!("Toggled tray menu: {id} on {name}"),
    }

    Ok(())
}
