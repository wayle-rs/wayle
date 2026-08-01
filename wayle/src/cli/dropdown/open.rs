use crate::cli::dbus::{format_ipc_error, shell_ipc_proxy};
use crate::cli::CliAction;

pub async fn execute(identifier: String, monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .dropdown_open(monitor, &identifier)
        .await
        .map_err(|err| format_ipc_error("open dropdown", err))?;

    match monitor {
        "" => println!("Opened {identifier} on the active monitor"),
        "all" => println!("Opened {identifier} on all monitors"),
        name => println!("Opened {identifier} on {name}"),
    }

    Ok(())
}
