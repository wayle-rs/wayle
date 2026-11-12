use super::proxy::{connect, format_error};
use crate::cli::CliAction;

/// Executes the dismiss command.
///
/// # Errors
/// Returns error if D-Bus communication fails.
pub async fn execute(id: i64) -> CliAction {
    let (_connection, proxy) = connect().await?;

    proxy
        .dismiss(id)
        .await
        .map_err(|e| format_error("dismiss notification", e))?;

    println!("Dismissed notification {id}");

    Ok(())
}
