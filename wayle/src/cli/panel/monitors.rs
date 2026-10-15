use super::proxy::{format_ipc_error, shell_ipc_proxy};
use crate::cli::CliAction;

pub async fn execute() -> CliAction {
    let (_connection, proxy) = shell_ipc_proxy().await?;

    let monitors = proxy
        .monitors()
        .await
        .map_err(|err| format_ipc_error("list monitors", err))?;

    for (connector, description, serial) in monitors {
        println!("{connector}");
        if !serial.is_empty() {
            println!("  sn:{serial}");
        }
        if !description.is_empty() {
            println!("  desc:{description}");
        }
    }

    Ok(())
}
