use crate::cli::dbus::{format_ipc_error, shell_ipc_proxy};
use crate::cli::CliAction;

pub async fn execute(identifier: String, monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .dropdown_toggle(monitor, &identifier)
        .await
        .map_err(|err| format_ipc_error("toggle dropdown", err))?;

    match monitor {
        "" => println!("Toggled {identifier} on the active monitor"),
        "all" => println!("Toggled {identifier} on all monitors"),
        name => println!("Toggled {identifier} on {name}"),
    }

    Ok(())
}
