//! JSON Schema generation for configuration validation and editor support.

use std::{fs, io, path::Path};

use schemars::{JsonSchema, generate::SchemaGenerator};
use tracing::{debug, error, info};

use super::paths::{ConfigPaths, theme_schema_json};
use crate::{Config, infrastructure::themes::Palette};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Generates the JSON Schema for Wayle's root configuration.
///
/// The schema includes the package version in the `$id` field for version tracking.
///
/// Returns `None` if schema serialization fails.
pub fn generate_schema() -> Option<String> {
    generate_schema_for::<Config>(&format!("wayle-config-{VERSION}"))
}

/// Generates the JSON Schema for theme palette files.
///
/// Returns `None` if schema serialization fails.
pub fn generate_theme_schema() -> Option<String> {
    generate_schema_for::<Palette>(&format!("wayle-theme-{VERSION}"))
}

fn generate_schema_for<T: JsonSchema>(id: &str) -> Option<String> {
    let generator = SchemaGenerator::default();
    let schema = generator.into_root_schema_for::<T>();

    let mut json: serde_json::Value = match serde_json::to_value(&schema) {
        Ok(value) => value,
        Err(err) => {
            error!(error = %err, "failed to serialize schema to JSON value");
            return None;
        }
    };

    if let Some(obj) = json.as_object_mut() {
        obj.insert("$id".to_string(), serde_json::Value::String(id.to_owned()));
    }

    match serde_json::to_string(&json) {
        Ok(text) => Some(text),
        Err(err) => {
            error!(error = %err, "failed to serialize JSON to string");
            None
        }
    }
}

const TOMBI_CONFIG: &str = r#"[schema]
enabled = true

[[schemas]]
path = "./schema.json"
include = ["config.toml", "runtime.toml"]

[[schemas]]
path = "./themes/schema.json"
include = ["themes/*.toml"]
"#;

/// Ensures the schema and Tombi config files exist and are up-to-date.
///
/// Writes `schema.json`, `themes/schema.json`, and `tombi.toml` to `~/.config/wayle/` if:
/// - The files don't exist
/// - The schema exists but contains a different version
///
/// # Errors
///
/// Returns error if the files cannot be written or schema generation fails.
pub fn ensure_schema_current() -> io::Result<()> {
    ensure_root_schema()?;
    ensure_theme_schema()?;
    ensure_tombi_config()?;
    Ok(())
}

fn ensure_root_schema() -> io::Result<()> {
    let schema_path = ConfigPaths::schema_json();

    if let Some(parent) = schema_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    let needs_update = match fs::read_to_string(&schema_path) {
        Ok(existing) => !existing.contains(&format!("wayle-config-{VERSION}")),
        Err(_) => true,
    };

    if !needs_update {
        debug!(path = %schema_path.display(), "Root schema already current");
        return Ok(());
    }

    let Some(schema) = generate_schema() else {
        return Err(io::Error::other("root schema generation failed"));
    };
    fs::write(&schema_path, schema)?;
    info!(path = %schema_path.display(), version = VERSION, "Root schema generated");
    Ok(())
}

fn ensure_theme_schema() -> io::Result<()> {
    let schema_path = theme_schema_json();

    if let Some(parent) = schema_path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    let needs_update = match fs::read_to_string(&schema_path) {
        Ok(existing) => !existing.contains(&format!("wayle-theme-{VERSION}")),
        Err(_) => true,
    };

    if !needs_update {
        debug!(path = %schema_path.display(), "Theme schema already current");
        return Ok(());
    }

    let Some(schema) = generate_theme_schema() else {
        return Err(io::Error::other("theme schema generation failed"));
    };
    fs::write(&schema_path, schema)?;
    info!(path = %schema_path.display(), version = VERSION, "Theme schema generated");
    Ok(())
}

fn ensure_tombi_config() -> io::Result<()> {
    let tombi_path = ConfigPaths::tombi_config();

    let current = fs::read_to_string(&tombi_path).ok();

    if current.as_deref() == Some(TOMBI_CONFIG) {
        return Ok(());
    }

    fs::write(&tombi_path, TOMBI_CONFIG)?;
    info!(path = %tombi_path.display(), "Tombi config generated");
    Ok(())
}

/// Writes the root config schema to the specified path, regardless of version.
///
/// # Errors
///
/// Returns error if the schema file cannot be written or schema generation fails.
pub fn write_schema_to(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }

    let Some(schema) = generate_schema() else {
        return Err(io::Error::other("schema generation failed"));
    };
    fs::write(path, schema)
}
