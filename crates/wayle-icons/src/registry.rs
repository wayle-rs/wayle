//! Registers the Wayle icon directory with GTK's IconTheme so icons are discoverable at runtime.
//!
//! Also watches the icon directory for changes and automatically refreshes
//! GTK's icon cache when icons are added or removed.

use std::{fs, path::PathBuf, thread};

use gtk4::{gdk, glib};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{debug, info, warn};

use crate::error::{Error, Result};

const SYSTEM_ICONS_PATH: &str = "/usr/share/wayle/icons";

const INDEX_THEME_CONTENT: &str = r#"[Icon Theme]
Name=Wayle Icons
Comment=Icons installed by Wayle
Directories=hicolor/scalable/actions

[hicolor/scalable/actions]
Size=48
MinSize=16
MaxSize=512
Type=Scalable
"#;

/// Manages GTK IconTheme registration for Wayle icons.
///
/// [`IconRegistry::init`] should be invoked at application startup to ensure
/// GTK can discover icons installed via `wayle icons install`.
#[derive(Debug, Clone)]
pub struct IconRegistry {
    base_path: PathBuf,
}

impl IconRegistry {
    /// Creates a new registry with the default icon directory.
    ///
    /// The default path is `~/.local/share/wayle/icons/`.
    ///
    /// # Errors
    ///
    /// Returns error if `$HOME` is not set and `$XDG_DATA_HOME` is also unset.
    pub fn new() -> Result<Self> {
        Ok(Self {
            base_path: Self::default_path()?,
        })
    }

    /// Creates a registry with a custom icon directory.
    ///
    /// Useful for testing or custom configurations.
    pub fn with_path(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Returns the default icon directory path.
    ///
    /// Uses `$XDG_DATA_HOME/wayle/icons` or `~/.local/share/wayle/icons`.
    ///
    /// # Errors
    ///
    /// Returns error if `$HOME` is not set and `$XDG_DATA_HOME` is also unset.
    pub fn default_path() -> Result<PathBuf> {
        let data_home = match std::env::var("XDG_DATA_HOME") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                let home = std::env::var("HOME").map_err(|_| Error::HomeNotSet)?;
                PathBuf::from(home).join(".local").join("share")
            }
        };

        Ok(data_home.join("wayle").join("icons"))
    }

    /// Returns the base path for this registry.
    pub fn base_path(&self) -> &PathBuf {
        &self.base_path
    }

    /// Returns the directory where SVG icons are stored.
    ///
    /// This is `<base_path>/hicolor/scalable/actions/`.
    pub fn icons_dir(&self) -> PathBuf {
        self.base_path
            .join("hicolor")
            .join("scalable")
            .join("actions")
    }

    /// Ensures the icon directory structure and index.theme exist.
    ///
    /// Suitable for CLI contexts where GTK is not available.
    /// GUI applications should use [`Self::init`] instead, which also
    /// registers with GTK and starts file watching.
    ///
    /// # Errors
    ///
    /// Returns error if directory creation or file writing fails.
    pub fn ensure_setup(&self) -> Result<()> {
        self.ensure_directory_structure()?;
        self.ensure_index_theme()?;
        Ok(())
    }

    /// Initializes the icon registry with GTK and starts watching for changes.
    ///
    /// 1. Creates the icon directory structure if it doesn't exist
    /// 2. Creates the `index.theme` file if missing
    /// 3. Registers the directory with GTK's IconTheme
    /// 4. Starts a background watcher that refreshes icons when files change
    ///
    /// Should be invoked once at application startup before displaying any
    /// widgets that use Wayle icons.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - File writing fails
    /// - No display is available (GTK not initialized)
    pub fn init(&self) -> Result<()> {
        self.ensure_setup()?;
        self.register_with_gtk()?;
        self.start_watcher();

        info!(path = %self.base_path.display(), "Icon registry initialized with file watching");
        Ok(())
    }

    fn start_watcher(&self) {
        let icons_dir = self.icons_dir();
        let base_path = self.base_path.clone();

        thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher: RecommendedWatcher =
                match notify::recommended_watcher(move |res: std::result::Result<Event, _>| {
                    if let Ok(event) = res
                        && (event.kind.is_create() || event.kind.is_remove())
                    {
                        let _ = tx.send(());
                    }
                }) {
                    Ok(watcher) => watcher,
                    Err(err) => {
                        warn!(error = %err, "cannot create icon watcher");
                        return;
                    }
                };

            if let Err(err) = watcher.watch(&icons_dir, RecursiveMode::NonRecursive) {
                warn!(error = %err, path = %icons_dir.display(), "cannot watch icons directory");
                return;
            }

            debug!(path = %icons_dir.display(), "Watching icons directory for changes");

            let mut last_refresh = std::time::Instant::now();
            loop {
                if rx.recv().is_err() {
                    break;
                }

                if last_refresh.elapsed() < std::time::Duration::from_millis(100) {
                    continue;
                }
                last_refresh = std::time::Instant::now();

                let base_path = base_path.clone();
                glib::idle_add_once(move || {
                    if let Err(err) = Self::refresh_gtk_theme(&base_path) {
                        warn!(error = %err, "cannot refresh icon theme");
                    } else {
                        debug!("Icon theme refreshed");
                    }
                });
            }
        });
    }

    fn ensure_directory_structure(&self) -> Result<()> {
        let icons_dir = self.icons_dir();

        if !icons_dir.exists() {
            fs::create_dir_all(&icons_dir).map_err(|source| Error::DirectoryError {
                path: icons_dir,
                source,
            })?;
        }

        Ok(())
    }

    fn ensure_index_theme(&self) -> Result<()> {
        let index_path = self.base_path.join("index.theme");

        if !index_path.exists() {
            fs::write(&index_path, INDEX_THEME_CONTENT).map_err(|source| Error::WriteError {
                path: index_path,
                source,
            })?;
        }

        Ok(())
    }

    fn register_with_gtk(&self) -> Result<()> {
        Self::refresh_gtk_theme(&self.base_path)
    }

    /// Forces GTK to rescan icon directories and pick up newly installed icons.
    ///
    /// GTK's IconTheme caches directory contents at startup and doesn't
    /// automatically detect new files. This is needed after installing icons
    /// via CLI while a GUI application is running.
    ///
    /// # Errors
    ///
    /// Returns error if no display is available.
    pub fn refresh() -> Result<()> {
        let path = Self::default_path()?;
        Self::refresh_gtk_theme(&path)
    }

    fn refresh_gtk_theme(user_path: &PathBuf) -> Result<()> {
        let display = gdk::Display::default().ok_or_else(|| Error::RegistryError {
            reason: "no display available",
        })?;

        let icon_theme = gtk4::IconTheme::for_display(&display);

        let existing: Vec<PathBuf> = icon_theme
            .search_path()
            .into_iter()
            .filter(|p| p != user_path && !Self::system_icon_paths().contains(p))
            .collect();

        let mut paths = vec![user_path.clone()];
        for system_path in Self::system_icon_paths() {
            if !paths.contains(&system_path) {
                paths.push(system_path);
            }
        }
        paths.extend(existing);

        let path_refs: Vec<&std::path::Path> = paths.iter().map(|p| p.as_path()).collect();
        icon_theme.set_search_path(&path_refs);

        Ok(())
    }

    pub(crate) fn system_icon_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        let xdg_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
        for dir in xdg_dirs.split(':').filter(|d| !d.is_empty()) {
            let path = PathBuf::from(dir).join("wayle/icons");
            if path.exists() {
                paths.push(path);
            }
        }

        let system_path = PathBuf::from(SYSTEM_ICONS_PATH);
        if system_path.exists() && !paths.contains(&system_path) {
            paths.push(system_path);
        }

        paths
    }

    /// Checks if the icon directory and theme are properly set up.
    ///
    /// Returns `true` if:
    /// - The icons directory exists
    /// - The index.theme file exists
    pub fn is_valid(&self) -> bool {
        self.icons_dir().exists() && self.base_path.join("index.theme").exists()
    }
}
