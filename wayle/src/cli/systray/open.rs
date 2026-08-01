use crate::cli::{
    CliAction,
    dbus::{format_ipc_error, shell_ipc_proxy},
};

/// Executes the open command: opens a tray item's menu by ID; no-op if it is
/// already open.
///
/// Unlike `activate`/`list` (which talk to the systray service), opening the menu
/// is a shell UI action, so this goes through the shell IPC (`com.wayle.Shell1`).
/// It is best-effort per bar: an id not currently present is a no-op. The
/// `monitor` argument follows the shared rules (omit = active monitor, a connector
/// name, or "all").
///
/// # Errors
/// Returns error if D-Bus communication fails.
pub async fn execute(id: String, monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .systray_open(&id, monitor)
        .await
        .map_err(|err| format_ipc_error("open tray menu", err))?;

    match monitor {
        "" => println!("Opened tray menu: {id} on the active monitor"),
        "all" => println!("Opened tray menu: {id} on all monitors"),
        name => println!("Opened tray menu: {id} on {name}"),
    }

    Ok(())
}
