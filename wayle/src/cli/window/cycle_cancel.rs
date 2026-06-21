use crate::cli::{CliAction, dbus};

pub async fn execute() -> CliAction {
    let (_connection, proxy) = dbus::shell_ipc_proxy().await?;

    proxy
        .window_cycle_cancel()
        .await
        .map_err(|err| dbus::format_error("Shell", "cancel window switcher cycle", err))?;

    Ok(())
}
