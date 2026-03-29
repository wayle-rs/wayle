//! Wallust color extraction.
//!
//! Generates a wallust config from wayle's settings and runs a single extraction.

use std::path::PathBuf;

use tokio::{fs, process::Command};
use tracing::debug;
use wayle_core::paths::ConfigPaths;

use super::Tool;
use crate::error::Error;

const COLORS_TEMPLATE: &str = r#"{
    "wallpaper": "{{wallpaper}}",
    "background": "{{background}}",
    "foreground": "{{foreground}}",
    "cursor": "{{cursor}}",
    "color0": "{{color0}}",
    "color1": "{{color1}}",
    "color2": "{{color2}}",
    "color3": "{{color3}}",
    "color4": "{{color4}}",
    "color5": "{{color5}}",
    "color6": "{{color6}}",
    "color7": "{{color7}}",
    "color8": "{{color8}}",
    "color9": "{{color9}}",
    "color10": "{{color10}}",
    "color11": "{{color11}}",
    "color12": "{{color12}}",
    "color13": "{{color13}}",
    "color14": "{{color14}}",
    "color15": "{{color15}}"
}
"#;

/// Runs wallust color extraction on the given image.
///
/// # Errors
///
/// Returns error if wallust command fails or config paths are unavailable.
pub async fn extract(
    image_path: &str,
    palette: &str,
    saturation: u8,
    check_contrast: bool,
    backend: &str,
    colorspace: &str,
    apply_globally: bool,
) -> Result<(), Error> {
    let (config_path, templates_dir) =
        write_config(palette, saturation, check_contrast, backend, colorspace).await?;

    let mut cmd = Command::new("wallust");
    cmd.args(["run", image_path]);
    cmd.args(["-C", &config_path.to_string_lossy()]);
    cmd.args(["--templates-dir", &templates_dir.to_string_lossy()]);

    if !apply_globally {
        cmd.arg("-s");
    }

    let output = Tool::Wallust.run(cmd).await?;
    Tool::Wallust.check_success(&output)?;

    debug!(image = %image_path, "Wallust color extraction complete");
    Ok(())
}

async fn write_config(
    palette: &str,
    saturation: u8,
    check_contrast: bool,
    backend: &str,
    colorspace: &str,
) -> Result<(PathBuf, PathBuf), Error> {
    let data_dir = ConfigPaths::data_dir().map_err(|source| Error::ConfigPathError {
        context: "wayle data directory",
        source,
    })?;

    let wallust_dir = data_dir.join("wallust");
    let templates_dir = wallust_dir.join("templates");
    let config_path = wallust_dir.join("wallust.toml");
    let template_path = templates_dir.join("colors.json");

    fs::create_dir_all(&templates_dir)
        .await
        .map_err(|source| Error::ConfigPathError {
            context: "creating wallust templates directory",
            source,
        })?;

    let colors_output = ConfigPaths::wallust_colors().map_err(|source| Error::ConfigPathError {
        context: "wallust colors output path",
        source,
    })?;

    let saturation_line = if saturation > 0 {
        format!("saturation = {saturation}\n")
    } else {
        String::new()
    };

    let config_content = format!(
        r#"palette = "{palette}"
backend = "{backend}"
color_space = "{colorspace}"
check_contrast = {check_contrast}
dynamic_threshold = true
{saturation_line}
[templates]
colors = {{ template = "colors.json", target = "{}" }}
"#,
        colors_output.display()
    );

    fs::write(&config_path, config_content)
        .await
        .map_err(|source| Error::ConfigPathError {
            context: "writing wayle wallust config",
            source,
        })?;

    fs::write(&template_path, COLORS_TEMPLATE)
        .await
        .map_err(|source| Error::ConfigPathError {
            context: "writing colors.json template",
            source,
        })?;

    Ok((config_path, templates_dir))
}
