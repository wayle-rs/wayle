use std::{fs, path::Path};

use tracing::{error, info};

use crate::{
    Config, Error,
    infrastructure::themes::{Palette, palettes::builtins},
    schemas::styling::ThemeEntry,
};

pub(crate) fn load_themes(config: &Config, themes_dir: &Path) {
    let mut all_themes: Vec<ThemeEntry> = builtins();

    let Ok(entries) = fs::read_dir(themes_dir) else {
        info!("Themes directory not found...");
        config.styling.available.set(all_themes);
        return;
    };

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "toml") {
            continue;
        }

        let Ok(theme) = get_theme_from_file(&path) else {
            error!(path = %path.display(), "cannot load theme");
            continue;
        };

        if all_themes
            .iter()
            .any(|existing| existing.name == theme.name)
        {
            error!(theme = %theme.name, "theme already exists, skipping");
            continue;
        }

        all_themes.push(theme);
    }

    config.styling.available.set(all_themes);
}

fn get_theme_from_file(path: &Path) -> Result<ThemeEntry, Error> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap_or_default();

    let content = fs::read_to_string(path).map_err(|source| Error::ThemeRead {
        path: path.into(),
        source,
    })?;

    let palette: Palette = toml::from_str(&content).map_err(|source| Error::ThemeParse {
        path: path.into(),
        source,
    })?;

    Ok(ThemeEntry {
        name,
        palette,
        builtin: false,
    })
}
