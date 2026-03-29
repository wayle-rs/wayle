use std::{
    env, fs,
    io::{Error, ErrorKind},
    path::PathBuf,
};

/// Configuration path resolver following XDG Base Directory specification.
pub struct ConfigPaths;

impl ConfigPaths {
    /// Configuration directory path (`$XDG_CONFIG_HOME/wayle` or `~/.config/wayle`).
    ///
    /// # Errors
    ///
    /// Returns error if neither `XDG_CONFIG_HOME` nor `HOME` is set.
    pub fn config_dir() -> Result<PathBuf, Error> {
        let config_home = env::var("XDG_CONFIG_HOME")
            .or_else(|_| env::var("HOME").map(|home| format!("{home}/.config")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Neither XDG_CONFIG_HOME nor HOME environment variable found: {e}"),
                )
            })?;

        Ok(PathBuf::from(config_home).join("wayle"))
    }

    /// Data directory (`$XDG_DATA_HOME/wayle` or `~/.local/share/wayle`).
    /// Creates directory if absent.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set or directory creation fails.
    pub fn data_dir() -> Result<PathBuf, Error> {
        let data_home = env::var("XDG_DATA_HOME")
            .or_else(|_| env::var("HOME").map(|home| format!("{home}/.local/share")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Neither XDG_DATA_HOME nor HOME environment variable found: {e}"),
                )
            })?;

        let data_dir = PathBuf::from(data_home).join("wayle");

        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)?;
        }

        Ok(data_dir)
    }

    /// State directory (`$XDG_STATE_HOME/wayle` or `~/.local/state/wayle`).
    /// Creates directory if absent.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set or directory creation fails.
    pub fn state_dir() -> Result<PathBuf, Error> {
        let state_home = env::var("XDG_STATE_HOME")
            .or_else(|_| env::var("HOME").map(|home| format!("{home}/.local/state")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Neither XDG_STATE_HOME nor HOME environment variable found: {e}"),
                )
            })?;

        let state_dir = PathBuf::from(state_home).join("wayle");

        if !state_dir.exists() {
            fs::create_dir_all(&state_dir)?;
        }

        Ok(state_dir)
    }

    /// Log directory (`$XDG_STATE_HOME/wayle` or `~/.local/state/wayle`).
    /// Creates directory if absent.
    ///
    /// # Errors
    ///
    /// Returns error if directory creation fails.
    pub fn log_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let log_dir = Self::state_dir()?;

        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)?;
        }

        Ok(log_dir)
    }

    /// Path to `config.toml`.
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn main_config() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("config.toml"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Path to `runtime.toml` (GUI-modified settings).
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn runtime_config() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("runtime.toml"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Path to `themes/` directory.
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn themes_dir() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("themes"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Path to `schema.json` for editor autocomplete and validation.
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn schema_json() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("schema.json"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Path to `config.toml.example` with default configuration values.
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn example_config() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("config.toml.example"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Path to `tombi.toml` for Tombi TOML editor support.
    ///
    /// # Panics
    ///
    /// Panics if config directory cannot be determined.
    #[allow(clippy::panic)]
    pub fn tombi_config() -> PathBuf {
        match Self::config_dir() {
            Ok(dir) => dir.join("tombi.toml"),
            Err(_) => {
                panic!("Failed to determine config directory - is $HOME or $XDG_CONFIG_HOME set?")
            }
        }
    }

    /// Cache directory (`$XDG_CACHE_HOME/wayle` or `~/.cache/wayle`).
    /// Creates directory if absent.
    ///
    /// # Errors
    ///
    /// Returns error if environment variables are not set or directory creation fails.
    pub fn cache_dir() -> Result<PathBuf, Error> {
        let cache_home = env::var("XDG_CACHE_HOME")
            .or_else(|_| env::var("HOME").map(|home| format!("{home}/.cache")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Neither XDG_CACHE_HOME nor HOME environment variable found: {e}"),
                )
            })?;

        let cache_dir = PathBuf::from(cache_home).join("wayle");

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(cache_dir)
    }

    /// Path to cached matugen colors JSON.
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be determined or created.
    pub fn matugen_colors() -> Result<PathBuf, Error> {
        Ok(Self::cache_dir()?.join("matugen-colors.json"))
    }

    /// Path to pywal colors JSON (`~/.cache/wal/colors.json`).
    ///
    /// # Errors
    ///
    /// Returns error if `HOME` or `XDG_CACHE_HOME` is not set.
    pub fn pywal_colors() -> Result<PathBuf, Error> {
        let cache_home = env::var("XDG_CACHE_HOME")
            .or_else(|_| env::var("HOME").map(|home| format!("{home}/.cache")))
            .map_err(|e| {
                Error::new(
                    ErrorKind::NotFound,
                    format!("Neither XDG_CACHE_HOME nor HOME environment variable found: {e}"),
                )
            })?;

        Ok(PathBuf::from(cache_home).join("wal/colors.json"))
    }

    /// Path to cached wallust colors JSON.
    ///
    /// # Errors
    ///
    /// Returns error if cache directory cannot be determined or created.
    pub fn wallust_colors() -> Result<PathBuf, Error> {
        Ok(Self::cache_dir()?.join("wallust-colors.json"))
    }
}
