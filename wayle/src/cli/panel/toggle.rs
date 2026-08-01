use crate::cli::dbus::{format_ipc_error, shell_ipc_proxy};
use crate::cli::CliAction;

pub async fn execute(monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .bar_toggle(monitor)
        .await
        .map_err(|err| format_ipc_error("toggle bar", err))?;

    if monitor.is_empty() {
        println!("All bars toggled");
    } else {
        println!("Bar toggled on {monitor}");
    }

    Ok(())
}
