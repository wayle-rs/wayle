use crate::{
    cli::CliAction,
    config::{ConfigService, ConfigServiceCli},
};

/// Removes the runtime override for a configuration path.
///
/// # Errors
///
/// Returns error if config loading fails or the path is invalid.
pub async fn execute(path: String) -> CliAction {
    let config_service = ConfigService::load(None)
        .await
        .map_err(|e| format!("cannot load config: {e}"))?;

    let cleared = config_service
        .reset_by_path(&path)
        .map_err(|e| format!("cannot reset '{path}': {e}"))?;

    if cleared {
        config_service
            .save()
            .await
            .map_err(|e| format!("cannot save config: {e}"))?;

        let effective = config_service
            .get_by_path(&path)
            .map_err(|e| format!("cannot read value: {e}"))?;

        println!("Reset {path} (now using: {effective})");
    } else {
        println!("No runtime override at {path}");
    }

    Ok(())
}
