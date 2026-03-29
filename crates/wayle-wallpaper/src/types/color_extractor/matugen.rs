//! Matugen color extraction (Material You colors).

use std::path::Path;

use tokio::{fs, process::Command};
use tracing::{debug, warn};
use wayle_core::paths::ConfigPaths;

use super::Tool;
use crate::error::Error;

/// Matugen CLI arguments.
#[derive(Debug)]
pub enum Arg<'a> {
    /// `image <path>` - Extract colors from the specified image.
    Image(&'a Path),
    /// `--json <format>` - Output JSON in specified format.
    Json(&'static str),
    /// `--source-color-index <index>` - Avoid interactive source-color prompts.
    SourceColorIndex(u8),
    /// `--type <scheme>` - Color scheme type.
    Type(&'a str),
    /// `--contrast <value>` - Contrast level (-1.0 to 1.0).
    Contrast(f64),
    /// `-m <mode>` - Color mode (dark, light, amoled).
    Mode(&'a str),
}

impl Arg<'_> {
    fn apply(&self, cmd: &mut Command) {
        match self {
            Self::Image(path) => {
                cmd.args(["image", &path.to_string_lossy()]);
            }
            Self::Json(format) => {
                cmd.args(["--json", format]);
            }
            Self::SourceColorIndex(index) => {
                cmd.args(["--source-color-index", &index.to_string()]);
            }
            Self::Type(scheme) => {
                cmd.args(["--type", scheme]);
            }
            Self::Contrast(value) => {
                cmd.args(["--contrast", &value.to_string()]);
            }
            Self::Mode(mode) => {
                cmd.args(["-m", mode]);
            }
        }
    }
}

async fn run(args: &[Arg<'_>]) -> Result<Vec<u8>, Error> {
    let mut cmd = Command::new("matugen");

    for arg in args {
        arg.apply(&mut cmd);
    }

    let output = Tool::Matugen.run(cmd).await?;
    Tool::Matugen.check_success(&output)?;
    Ok(output.stdout)
}

/// Runs matugen color extraction on the given image.
///
/// Saves JSON output to wayle's cache for wayle-styling to consume.
///
/// # Errors
///
/// Returns error if matugen command fails.
pub async fn extract(
    image_path: &str,
    scheme: &str,
    contrast: f64,
    source_color: u8,
    mode: &str,
) -> Result<(), Error> {
    let path = Path::new(image_path);
    let stdout = run(&[
        Arg::Image(path),
        Arg::Json("hex"),
        Arg::SourceColorIndex(source_color),
        Arg::Type(scheme),
        Arg::Contrast(contrast),
        Arg::Mode(mode),
    ])
    .await?;
    save_output(&stdout).await;
    Ok(())
}

async fn save_output(stdout: &[u8]) {
    let Ok(cache_path) = ConfigPaths::matugen_colors() else {
        warn!("cannot get matugen cache path");
        return;
    };

    if let Err(err) = fs::write(&cache_path, stdout).await {
        warn!(error = %err, path = %cache_path.display(), "cannot save matugen colors");
        return;
    }

    debug!(path = %cache_path.display(), "Saved matugen colors");
}
