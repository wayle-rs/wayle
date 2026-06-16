use std::collections::BTreeSet;

use wayle_icons::{
    IconManager, IconOrigin, MissingIcon, SyncSummary,
    sync::{extract_referenced_icons, find_missing, install_missing},
};

use crate::{cli::CliAction, config::ConfigService};

/// Installs icons referenced by config but not yet on disk.
///
/// # Errors
///
/// Returns error if the config fails to load, fails to serialize, or the
/// icon manager cannot be constructed.
pub async fn execute(dry_run: bool) -> CliAction {
    let config_service = ConfigService::load()
        .await
        .map_err(|err| format!("Failed to load config: {err}"))?;

    let serialized = toml::Value::try_from(config_service.config())
        .map_err(|err| format!("Failed to serialize config: {err}"))?;

    let referenced = extract_referenced_icons(&serialized);

    let manager = IconManager::new().map_err(|err| err.to_string())?;
    let installed: BTreeSet<String> = manager.list().into_iter().collect();
    let missing = find_missing(&referenced, &installed);

    if missing.is_empty() {
        println!(
            "All {} referenced icons already installed.",
            referenced.len()
        );
        return Ok(());
    }

    if dry_run {
        print_dry_run(&missing);
        return Ok(());
    }

    let summary = install_missing(missing, &manager).await;
    print_summary(&summary);
    Ok(())
}

fn print_dry_run(missing: &[MissingIcon]) {
    println!("Would install {} icons:", missing.len());
    for icon in missing {
        match &icon.origin {
            IconOrigin::Source(source) => println!("  {} (from {source})", icon.name),
            IconOrigin::UserImported => {
                println!("  {} (no auto-install: import manually)", icon.name);
            }
        }
    }
}

fn print_summary(summary: &SyncSummary) {
    for name in &summary.installed {
        println!("Installed: {name}");
    }

    for skipped in &summary.skipped {
        println!("Skipped (manual import required): {skipped}");
    }

    for failure in &summary.failed {
        eprintln!("Failed: {} - {}", failure.name, failure.error);
    }

    println!(
        "\n{} installed, {} skipped, {} failed",
        summary.installed.len(),
        summary.skipped.len(),
        summary.failed.len()
    );
}
