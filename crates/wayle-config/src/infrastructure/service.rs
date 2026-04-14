use std::{
    io,
    sync::{Arc, RwLock},
};

use tokio::fs;
use tracing::{info, instrument, warn};

use super::{
    error::{Error, InvalidFieldReason, IoOperation},
    paths::ConfigPaths,
    secrets, toml_path,
    watcher::FileWatcher,
};
use crate::{
    ApplyConfigLayer, ApplyRuntimeLayer, ClearAllRuntime, ClearRuntimeByPath, CommitConfigReload,
    Config, ExtractRuntimeValues, infrastructure::themes::utils::load_themes,
};

/// Reactive configuration service.
///
/// Each config field can be watched independently for changes. Runtime
/// overrides are extracted directly from `ConfigProperty` fields.
#[derive(Clone)]
pub struct ConfigService {
    config: Arc<Config>,
    watcher: Arc<RwLock<Option<FileWatcher>>>,
}

impl ConfigService {
    /// Loads configuration from TOML files and starts file watcher.
    ///
    /// Applies `config.toml` to the config layer and `runtime.toml` to
    /// the runtime layer, then starts hot-reload file watching.
    ///
    /// # Errors
    ///
    /// Returns error if config files cannot be loaded or parsed.
    #[instrument]
    pub async fn load() -> Result<Arc<Self>, Error> {
        info!("Loading configuration");

        if let Ok(config_dir) = ConfigPaths::config_dir() {
            secrets::load_env_files(&config_dir);
        }

        let config = Config::default();
        let config_path = ConfigPaths::main_config();

        let config_result =
            tokio::task::spawn_blocking(move || Config::load_toml_with_imports(&config_path))
                .await
                .map_err(|source| Error::TaskJoin { source })?;

        match config_result {
            Ok(config_toml) => config.apply_config_layer(&config_toml, ""),
            Err(e) => warn!("using defaults, config.toml failed:\n{e}"),
        }

        let runtime_path = ConfigPaths::runtime_config();
        let runtime_result =
            tokio::task::spawn_blocking(move || Self::load_toml_file(&runtime_path))
                .await
                .map_err(|source| Error::TaskJoin { source })?;

        match runtime_result {
            Ok(runtime_toml) => {
                if let Err(e) = config.apply_runtime_layer(&runtime_toml, "") {
                    warn!("invalid runtime.toml value:\n{e}");
                }
            }
            Err(e) => warn!("runtime.toml failed:\n{e}"),
        }

        config.commit_config_reload();

        let service = Arc::new(Self {
            config: Arc::new(config),
            watcher: Arc::new(RwLock::new(None)),
        });

        let themes_dir = ConfigPaths::themes_dir();
        load_themes(&service.config, &themes_dir);

        let file_watcher = FileWatcher::start(Arc::clone(&service))?;
        *service
            .watcher
            .write()
            .map_err(|_| Error::WatcherPoisoned)? = Some(file_watcher);

        info!("Configuration loaded successfully");

        Ok(service)
    }

    /// Reference to the config root.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Drops every runtime override and deletes `runtime.toml` from disk.
    ///
    /// The in-memory reset happens first so subscribers see fresh values
    /// immediately. The file removal is best-effort: if it fails (permission,
    /// concurrent removal), a warning is logged and the error is returned so
    /// callers can decide whether to surface it to the user.
    ///
    /// # Errors
    ///
    /// Returns an error if `runtime.toml` exists but cannot be removed.
    pub fn reset_all_runtime(&self) -> Result<(), io::Error> {
        self.config.clear_all_runtime();

        let runtime_path = ConfigPaths::runtime_config();

        match std::fs::remove_file(&runtime_path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    path = %runtime_path.display(),
                    "failed to remove runtime.toml"
                );
                Err(err)
            }
        }
    }

    /// Subscribes to secrets reload events.
    ///
    /// Returns a receiver that fires whenever `.env` files are reloaded.
    /// Returns `None` if the watcher is not initialized.
    pub fn subscribe_secrets_reload(&self) -> Option<tokio::sync::watch::Receiver<()>> {
        self.watcher.read().ok().and_then(|guard| {
            guard
                .as_ref()
                .map(|watcher| watcher.subscribe_secrets_reload())
        })
    }

    /// Persists runtime layer values to `runtime.toml`.
    ///
    /// Only values with runtime overrides are written.
    ///
    /// # Errors
    ///
    /// Returns error if config cannot be serialized or written to disk.
    #[instrument(skip(self))]
    pub async fn save(&self) -> Result<(), Error> {
        let runtime_value = self
            .config
            .extract_runtime_values()
            .unwrap_or_else(|| toml::Value::Table(toml::Table::new()));

        let runtime_path = ConfigPaths::runtime_config();
        let temp_path = runtime_path.with_extension("tmp");

        let toml_str =
            toml::to_string_pretty(&runtime_value).map_err(|source| Error::Serialization {
                content_type: "runtime config",
                source,
            })?;

        fs::write(&temp_path, toml_str)
            .await
            .map_err(|source| Error::Persistence {
                path: temp_path.clone(),
                source,
            })?;

        fs::rename(&temp_path, &runtime_path)
            .await
            .map_err(|source| Error::Persistence {
                path: runtime_path.clone(),
                source,
            })?;

        info!("Configuration saved to runtime.toml");

        Ok(())
    }

    pub(crate) fn load_toml_file(path: &std::path::Path) -> Result<toml::Value, Error> {
        let content = std::fs::read_to_string(path).map_err(|source| Error::Io {
            operation: IoOperation::ReadFile,
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&content).map_err(|source| Error::TomlParse {
            path: path.to_path_buf(),
            source,
        })
    }
}

/// CLI extension for string-based config access.
///
/// Application code should prefer strongly-typed access via `config()`.
pub trait ConfigServiceCli {
    /// Retrieves value at a dot-separated path (e.g., `battery.enabled`).
    ///
    /// # Errors
    ///
    /// Returns error if path is invalid.
    fn get_by_path(&self, path: &str) -> Result<toml::Value, Error>;

    /// Sets a runtime override at a dot-separated path.
    ///
    /// # Errors
    ///
    /// Returns error if path is invalid.
    fn set_by_path(&self, path: &str, value: toml::Value) -> Result<(), Error>;

    /// Clears the runtime override at a dot-separated path.
    ///
    /// Returns `true` if a value was cleared, `false` if no override existed.
    ///
    /// # Errors
    ///
    /// Returns error if path is invalid.
    fn reset_by_path(&self, path: &str) -> Result<bool, Error>;
}

impl ConfigServiceCli for ConfigService {
    fn get_by_path(&self, path: &str) -> Result<toml::Value, Error> {
        let config_value =
            toml::Value::try_from(self.config.as_ref()).map_err(|source| Error::Serialization {
                content_type: "config",
                source,
            })?;

        let mut value = config_value;
        for segment in path.split('.') {
            value = value
                .get(segment)
                .ok_or_else(|| Error::InvalidConfigField {
                    field: segment.to_string(),
                    component: path.to_string(),
                    reason: InvalidFieldReason::NotFound,
                })?
                .clone();
        }

        Ok(value)
    }

    fn set_by_path(&self, path: &str, value: toml::Value) -> Result<(), Error> {
        let mut root = toml::Value::Table(toml::Table::new());
        toml_path::insert(&mut root, path, value)?;
        self.config
            .apply_runtime_layer(&root, "")
            .map_err(Error::InvalidValue)
    }

    fn reset_by_path(&self, path: &str) -> Result<bool, Error> {
        self.config
            .clear_runtime_by_path(path)
            .map_err(Error::InvalidValue)
    }
}
