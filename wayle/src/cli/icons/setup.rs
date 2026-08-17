use std::{
    env, fs,
    path::{Path, PathBuf},
};

use wayle_icons::IconRegistry;

use crate::cli::CliAction;

/// Bundled icons in a source checkout, for development builds. Compile-time
/// only: in a release binary this resolves to the build machine's checkout.
const DEV_RESOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/icons/hicolor/scalable/actions"
);

/// Bundled icons relative to the executable or a system data directory.
const ICONS_SUBPATH: &str = "icons/hicolor/scalable/actions";

/// Candidate locations for the bundled icons, in lookup order: next to the
/// executable (extracted release archive), system data directories (distro
/// packages), then the source checkout (development builds).
fn resource_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(exe_dir) = env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(exe_dir.join(ICONS_SUBPATH));
    }

    candidates.push(PathBuf::from("/usr/share/wayle").join(ICONS_SUBPATH));
    candidates.push(PathBuf::from("/usr/local/share/wayle").join(ICONS_SUBPATH));
    candidates.push(PathBuf::from(DEV_RESOURCES_DIR));

    candidates
}

/// Installs bundled icons from the resources directory.
///
/// # Errors
///
/// Returns error if no bundled icons are found or copy fails.
pub fn execute() -> CliAction {
    let candidates = resource_candidates();
    let Some(source_dir) = candidates.iter().find(|path| path.exists()) else {
        return Err(format!(
            "Bundled icons not found. Searched:\n{}",
            candidates
                .iter()
                .map(|path| format!("  {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ));
    };

    let registry = IconRegistry::new().map_err(|err| err.to_string())?;
    let dest_dir = registry.icons_dir();

    fs::create_dir_all(&dest_dir)
        .map_err(|err| format!("Failed to create icons directory: {err}"))?;

    let entries = fs::read_dir(source_dir)
        .map_err(|err| format!("Failed to read resources directory: {err}"))?;

    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(filename) = path.file_name() else {
            continue;
        };
        if path.extension().is_some_and(|ext| ext == "svg") {
            let dest_path = dest_dir.join(filename);
            fs::copy(&path, &dest_path)
                .map_err(|err| format!("Failed to copy {}: {err}", path.display()))?;
            println!(
                "Installed: {}",
                filename.to_string_lossy().trim_end_matches(".svg")
            );
            count += 1;
        }
    }

    println!("\n{count} icons installed to {}", dest_dir.display());
    Ok(())
}
