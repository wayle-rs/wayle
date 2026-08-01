use crate::cli::dbus::{format_ipc_error, shell_ipc_proxy};
use crate::cli::CliAction;

pub async fn execute(monitor: Option<String>) -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;
    let monitor = monitor.as_deref().unwrap_or("");

    proxy
        .dropdown_close(monitor)
        .await
        .map_err(|err| format_ipc_error("close dropdown", err))?;

    match monitor {
        "" => println!("Closed any open dropdown on the active monitor"),
        "all" => println!("Closed any open dropdown on all monitors"),
        name => println!("Closed any open dropdown on {name}"),
    }

    Ok(())
}
