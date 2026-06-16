mod circular_detection;
mod file_creation;
mod merging;

use std::{
    fs,
    path::{Path, PathBuf},
};

use circular_detection::CircularDetector;
use file_creation::create_default_config_file;
use merging::merge_toml_configs;
use toml::Value;

use super::error::{Error, IoOperation};
use crate::Config;

impl Config {
    /// Loads and deserializes configuration with imports resolved.
    ///
    /// # Errors
    ///
    /// Returns error on read failures, invalid TOML, import failures,
    /// deserialization failures, or circular imports.
    pub fn load_with_imports(path: &Path) -> Result<Config, Error> {
        let merged = Self::load_toml_with_imports(path)?;
        merged
            .try_into()
            .map_err(|source| Error::ConfigDeserialization { source })
    }

    /// Loads and merges configuration TOML with imports resolved.
    ///
    /// # Errors
    ///
    /// Returns error on read failures, invalid TOML, import failures,
    /// or circular imports.
    pub fn load_toml_with_imports(path: &Path) -> Result<Value, Error> {
        if !path.exists() {
            create_default_config_file(path)?;
        }

        let canonical_path = path.canonicalize().map_err(|source| Error::Io {
            operation: IoOperation::ResolvePath,
            path: path.to_path_buf(),
            source,
        })?;

        let mut detector = CircularDetector::new();
        Self::load_merged_toml(&canonical_path, path, &mut detector)
    }

    fn load_merged_toml(
        path: &Path,
        import_base: &Path,
        detector: &mut CircularDetector,
    ) -> Result<Value, Error> {
        detector.detect_circular_import(path)?;
        detector.push_to_chain(path);

        let main_config_content = fs::read_to_string(path)?;
        let import_paths = Self::extract_import_paths(&main_config_content)?;
        let imported_configs = Self::load_all_imports(import_base, &import_paths, detector)?;

        let main_config: Value =
            toml::from_str(&main_config_content).map_err(|source| Error::TomlParse {
                path: path.to_path_buf(),
                source,
            })?;

        detector.pop_from_chain();
        Ok(merge_toml_configs(imported_configs, main_config))
    }

    fn load_all_imports(
        base_path: &Path,
        import_paths: &[String],
        detector: &mut CircularDetector,
    ) -> Result<Vec<Value>, Error> {
        import_paths
            .iter()
            .map(|import_path| {
                let resolved_path = Self::resolve_import_path(base_path, import_path)?;
                let canonical_import =
                    resolved_path
                        .canonicalize()
                        .map_err(|source| Error::Import {
                            path: resolved_path.clone(),
                            source: Box::new(Error::Io {
                                operation: IoOperation::ResolvePath,
                                path: resolved_path.clone(),
                                source,
                            }),
                        })?;

                Self::load_imported_file_with_tracking(&canonical_import, detector)
            })
            .collect()
    }

    fn load_imported_file_with_tracking(
        path: &Path,
        detector: &mut CircularDetector,
    ) -> Result<Value, Error> {
        detector.detect_circular_import(path)?;
        detector.push_to_chain(path);

        let result = Self::load_toml_file_with_imports(path, detector);
        detector.pop_from_chain();
        result
    }

    fn load_toml_file_with_imports(
        path: &Path,
        detector: &mut CircularDetector,
    ) -> Result<Value, Error> {
        let content = fs::read_to_string(path).map_err(|source| Error::Import {
            path: path.to_path_buf(),
            source: Box::new(Error::Io {
                operation: IoOperation::ReadFile,
                path: path.to_path_buf(),
                source,
            }),
        })?;
        let import_paths = Self::extract_import_paths(&content)?;
        let imported_configs = Self::load_all_imports(path, &import_paths, detector)?;

        let main_value: Value = toml::from_str(&content).map_err(|source| Error::TomlParse {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(merge_toml_configs(imported_configs, main_value))
    }

    fn extract_import_paths(config_content: &str) -> Result<Vec<String>, Error> {
        let value =
            toml::from_str(config_content).map_err(|source| Error::TomlParseInline { source })?;

        let import_paths = if let Value::Table(table) = value {
            if let Some(Value::Array(imports)) = table.get("imports") {
                imports
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Ok(import_paths)
    }

    fn resolve_import_path(base_path: &Path, import_path: &str) -> Result<PathBuf, Error> {
        let parent_dir = base_path.parent().ok_or_else(|| Error::ImportNoParent {
            path: base_path.to_path_buf(),
        })?;

        let mut import_path_buf = PathBuf::from(import_path);
        if import_path_buf.extension().is_none() {
            import_path_buf.set_extension("toml");
        }

        let resolved_path = parent_dir.join(import_path_buf);
        Ok(resolved_path)
    }
}
