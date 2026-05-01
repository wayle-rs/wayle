//! Icon installation and removal operations.
//!
//! The manager handles fetching icons from CDN sources and storing them
//! in the icon directory for GTK to discover.

use std::{collections::BTreeSet, fs, path::Path};

use futures::future::join_all;
use tokio::fs as async_fs;
use tracing::{debug, info, warn};
use usvg::{Options, Tree};

use crate::{
    error::{Error, Result, SvgValidationError},
    registry::IconRegistry,
    sources::{self, CUSTOM_PREFIX, IconSource},
    transform,
};

/// Result of a batch icon installation operation.
#[derive(Debug, Clone, Default)]
pub struct InstallResult {
    /// Icons that were successfully installed.
    pub installed: Vec<String>,
    /// Icons that failed to install, with error messages.
    pub failed: Vec<InstallFailure>,
}

/// A failed icon installation.
#[derive(Debug, Clone)]
pub struct InstallFailure {
    /// The icon slug that failed.
    pub slug: String,
    /// The error message.
    pub error: String,
}

impl InstallResult {
    /// Returns true if all icons were installed successfully.
    pub fn all_succeeded(&self) -> bool {
        self.failed.is_empty()
    }

    /// Returns true if no icons were installed.
    pub fn all_failed(&self) -> bool {
        self.installed.is_empty()
    }
}

/// Manages icon installation and removal.
///
/// Uses [`IconRegistry`] to determine where icons are stored and provides
/// async methods to fetch icons from CDN sources.
#[derive(Debug, Clone)]
pub struct IconManager {
    registry: IconRegistry,
    client: reqwest::Client,
}

impl IconManager {
    /// Creates a new manager with the default icon directory.
    ///
    /// # Errors
    ///
    /// Returns error if `$HOME` is not set and `$XDG_DATA_HOME` is also unset.
    pub fn new() -> Result<Self> {
        Ok(Self {
            registry: IconRegistry::new()?,
            client: reqwest::Client::new(),
        })
    }

    /// Creates a manager with a custom registry.
    ///
    /// Useful for testing or custom configurations.
    pub fn with_registry(registry: IconRegistry) -> Self {
        Self {
            registry,
            client: reqwest::Client::new(),
        }
    }

    /// Returns the registry used by this manager.
    pub fn registry(&self) -> &IconRegistry {
        &self.registry
    }

    /// Installs icons from a source by fetching from CDN.
    ///
    /// Returns an [`InstallResult`] containing both successful and failed installations.
    ///
    /// # Arguments
    ///
    /// * `source` - The icon source (Tabler, SimpleIcons, etc.)
    /// * `slugs` - Icon identifiers to install (e.g., "home", "settings")
    ///
    /// # Errors
    ///
    /// Returns error only if the icon directory cannot be created. Individual
    /// icon failures are captured in [`InstallResult::failed`].
    pub async fn install(&self, source: &dyn IconSource, slugs: &[&str]) -> Result<InstallResult> {
        let icons_dir = self.registry.icons_dir();
        async_fs::create_dir_all(&icons_dir)
            .await
            .map_err(|err| Error::DirectoryError {
                path: icons_dir.clone(),
                source: err,
            })?;

        let source_name = source.cli_name();
        let fetch_data: Vec<_> = slugs
            .iter()
            .map(|slug| {
                let url = source.cdn_url(slug);
                let icon_name = source.icon_name(slug);
                (*slug, url, icon_name)
            })
            .collect();

        let futures: Vec<_> = fetch_data
            .iter()
            .map(|(slug, url, icon_name)| self.fetch_and_save(slug, url, icon_name, &icons_dir))
            .collect();

        let results = join_all(futures).await;

        let mut install_result = InstallResult::default();
        for (slug, result) in fetch_data.iter().map(|(s, _, _)| *s).zip(results) {
            match result {
                Ok(name) => {
                    info!(icon = %name, source = source_name, "Installed icon");
                    install_result.installed.push(name);
                }
                Err(err) => {
                    warn!(slug = %slug, source = source_name, error = %err, "cannot install icon");
                    install_result.failed.push(InstallFailure {
                        slug: slug.to_string(),
                        error: err.to_string(),
                    });
                }
            }
        }

        Ok(install_result)
    }

    async fn fetch_and_save(
        &self,
        slug: &str,
        url: &str,
        icon_name: &str,
        icons_dir: &Path,
    ) -> Result<String> {
        debug!(url = %url, "Fetching icon");

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::FetchError {
                slug: slug.to_string(),
                icon_source: "cdn".to_string(),
                status: response.status(),
            });
        }

        let svg_content = response.text().await?;
        Self::validate_svg(&svg_content, slug)?;

        let transformed = transform::to_symbolic(&svg_content);
        let file_path = icons_dir.join(format!("{icon_name}-symbolic.svg"));
        async_fs::write(&file_path, &transformed)
            .await
            .map_err(|source| Error::WriteError {
                path: file_path,
                source,
            })?;

        Ok(format!("{icon_name}-symbolic"))
    }

    /// Removes an installed icon by name.
    ///
    /// # Arguments
    ///
    /// * `icon_name` - Full icon name including prefix (e.g., "tb-home")
    ///
    /// # Errors
    ///
    /// Returns error if icon doesn't exist or deletion fails.
    pub fn remove(&self, icon_name: &str) -> Result<()> {
        let file_path = self.registry.icons_dir().join(format!("{icon_name}.svg"));

        if !file_path.exists() {
            return Err(Error::NotFound {
                name: icon_name.to_string(),
            });
        }

        fs::remove_file(&file_path).map_err(|source| Error::DeleteError {
            name: icon_name.to_string(),
            source,
        })?;

        info!(icon = %icon_name, "Removed icon");
        Ok(())
    }

    /// Lists all installed icons.
    ///
    /// Returns icon names without the `.svg` extension.
    pub fn list(&self) -> Vec<String> {
        let mut dirs = vec![self.registry.icons_dir()];

        for system_path in IconRegistry::system_icon_paths() {
            let actions_dir = system_path.join("hicolor").join("scalable").join("actions");

            if actions_dir.exists() {
                dirs.push(actions_dir);
            }
        }

        let mut icons: BTreeSet<String> = BTreeSet::new();

        for dir in dirs {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if path.extension().is_some_and(|ext| ext == "svg")
                    && let Some(name) = path.file_stem().and_then(|stem| stem.to_str())
                {
                    icons.insert(name.to_owned());
                }
            }
        }

        icons.into_iter().collect()
    }

    /// Checks if an icon is installed.
    ///
    /// # Arguments
    ///
    /// * `icon_name` - Full icon name including prefix (e.g., "tb-home")
    pub fn is_installed(&self, icon_name: &str) -> bool {
        self.registry
            .icons_dir()
            .join(format!("{icon_name}.svg"))
            .exists()
    }

    /// Imports a local SVG file as a custom icon.
    ///
    /// Validates the SVG using usvg, transforms it for GTK compatibility,
    /// and installs it with the `cm-` prefix.
    ///
    /// # Errors
    ///
    /// Returns error if the file doesn't exist, isn't a valid SVG, or cannot be written.
    pub fn import_local(&self, path: &Path, name: &str) -> Result<String> {
        if !path.exists() {
            return Err(Error::NotFound {
                name: path.display().to_string(),
            });
        }

        let content = fs::read_to_string(path).map_err(|source| Error::ReadError {
            path: path.to_path_buf(),
            source,
        })?;

        Tree::from_str(&content, &Options::default()).map_err(|err| Error::InvalidSvg {
            slug: name.to_string(),
            reason: SvgValidationError::ParseError(err.to_string()),
        })?;

        let icons_dir = self.registry.icons_dir();
        fs::create_dir_all(&icons_dir).map_err(|source| Error::DirectoryError {
            path: icons_dir.clone(),
            source,
        })?;

        let transformed = transform::to_symbolic(&content);
        let icon_name = format!("{CUSTOM_PREFIX}-{name}-symbolic");
        let dest_path = icons_dir.join(format!("{icon_name}.svg"));

        fs::write(&dest_path, &transformed).map_err(|source| Error::WriteError {
            path: dest_path,
            source,
        })?;

        info!(icon = %icon_name, path = %path.display(), "Imported custom icon");
        Ok(icon_name)
    }

    /// Imports all SVG files from a directory.
    ///
    /// Files are transformed for GTK symbolic icon compatibility. Names are
    /// preserved, with `cm-` prefix added if no known source prefix exists.
    ///
    /// # Errors
    ///
    /// Returns error if directory cannot be read or icons directory cannot be created.
    pub fn import_dir(&self, dir: &Path) -> Result<InstallResult> {
        if !dir.is_dir() {
            return Err(Error::NotFound {
                name: dir.display().to_string(),
            });
        }

        let icons_dir = self.registry.icons_dir();
        fs::create_dir_all(&icons_dir).map_err(|source| Error::DirectoryError {
            path: icons_dir.clone(),
            source,
        })?;

        let entries = fs::read_dir(dir).map_err(|source| Error::ReadError {
            path: dir.to_path_buf(),
            source,
        })?;

        let mut result = InstallResult::default();
        let prefixes = sources::all_prefixes();

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "svg") {
                match self.import_single_file(&path, &icons_dir, &prefixes) {
                    Ok(name) => {
                        info!(icon = %name, "Imported icon");
                        result.installed.push(name);
                    }
                    Err(err) => {
                        let filename = path.file_name().unwrap_or_default().to_string_lossy();
                        warn!(file = %filename, error = %err, "cannot import icon");
                        result.failed.push(InstallFailure {
                            slug: filename.to_string(),
                            error: err.to_string(),
                        });
                    }
                }
            }
        }

        Ok(result)
    }

    fn import_single_file(
        &self,
        path: &Path,
        icons_dir: &Path,
        prefixes: &[&str],
    ) -> Result<String> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::NotFound {
                name: path.display().to_string(),
            })?;

        let base_name = stem.strip_suffix("-symbolic").unwrap_or(stem);
        let has_prefix = prefixes
            .iter()
            .any(|p| base_name.starts_with(&format!("{p}-")));

        let icon_name = if has_prefix {
            format!("{base_name}-symbolic")
        } else {
            format!("{CUSTOM_PREFIX}-{base_name}-symbolic")
        };

        let content = fs::read_to_string(path).map_err(|source| Error::ReadError {
            path: path.to_path_buf(),
            source,
        })?;

        Tree::from_str(&content, &Options::default()).map_err(|err| Error::InvalidSvg {
            slug: base_name.to_string(),
            reason: SvgValidationError::ParseError(err.to_string()),
        })?;

        let transformed = transform::to_symbolic(&content);
        let dest_path = icons_dir.join(format!("{icon_name}.svg"));

        fs::write(&dest_path, &transformed).map_err(|source| Error::WriteError {
            path: dest_path,
            source,
        })?;

        Ok(icon_name)
    }

    fn validate_svg(content: &str, slug: &str) -> Result<()> {
        Tree::from_str(content, &Options::default()).map_err(|err| Error::InvalidSvg {
            slug: slug.to_string(),
            reason: SvgValidationError::ParseError(err.to_string()),
        })?;
        Ok(())
    }
}
