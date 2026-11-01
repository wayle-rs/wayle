use std::{path::PathBuf, sync::Arc, time::Duration};

use notify::{
    Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher, event::EventKind,
};
use tokio::sync::{mpsc, watch};
use tracing::{debug, error, info, instrument};

use super::{error::Error, paths::ConfigPaths, secrets, service::ConfigService};
use crate::{
    ApplyConfigLayer, ApplyRuntimeLayer, CommitConfigReload, Config, ResetConfigLayer,
    ResetRuntimeLayer, infrastructure::themes::utils::load_themes,
};

/// Hot-reloads configuration files on disk changes.
///
/// When config files are modified, reloads them and updates corresponding
/// `ConfigProperty` values. Uses `send_if_modified` to prevent circular writes.
#[derive(Clone)]
pub struct FileWatcher {
    config_service: Arc<ConfigService>,
    secrets_tx: watch::Sender<()>,
    _watcher: Arc<RecommendedWatcher>,
}

impl FileWatcher {
    /// Subscribes to secrets reload events.
    ///
    /// The receiver fires whenever `.env` files are reloaded.
    pub fn subscribe_secrets_reload(&self) -> watch::Receiver<()> {
        self.secrets_tx.subscribe()
    }
}

impl FileWatcher {
    /// Starts watching config directory for changes.
    ///
    /// # Errors
    ///
    /// Returns error if file watching cannot be initialized.
    #[instrument(skip(config_service))]
    pub fn start(config_service: Arc<ConfigService>) -> Result<Self, Error> {
        let (tx, rx) = mpsc::unbounded_channel();
        let (secrets_tx, _) = watch::channel(());

        let mut watcher = notify::recommended_watcher(move |result: Result<Event, _>| {
            if let Ok(event) = result {
                let _ = tx.send(event);
            }
        })
        .map_err(|source| Error::WatcherInit { source })?;

        let config_dir = ConfigPaths::config_dir()?;

        watcher
            .watch(&config_dir, RecursiveMode::Recursive)
            .map_err(|source| Error::Watch {
                path: config_dir.clone(),
                source,
            })?;

        info!(?config_dir, "Config directory watcher started");

        let file_watcher = Self {
            config_service,
            secrets_tx,
            _watcher: Arc::new(watcher),
        };

        tokio::spawn(run_debounced_event_loop(file_watcher.clone(), rx));

        Ok(file_watcher)
    }

    fn should_reload(event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        )
    }

    #[instrument(skip(self))]
    async fn reload_and_sync(&self, paths: &[PathBuf]) -> Result<(), Error> {
        let themes_dir = ConfigPaths::themes_dir();
        let runtime_path = ConfigPaths::runtime_config();
        let runtime_tmp_path = runtime_path.with_extension("tmp");

        let is_env = |path: &PathBuf| secrets::is_env_file(path);
        let is_theme = |path: &PathBuf| path.starts_with(&themes_dir);
        let is_runtime = |path: &PathBuf| path == &runtime_path || path == &runtime_tmp_path;

        let has_env_changes = paths.iter().any(is_env);
        let has_theme_changes = paths.iter().any(is_theme);
        let has_runtime_changes = paths.iter().any(is_runtime);
        let has_main_config_changes = paths
            .iter()
            .any(|path| !is_env(path) && !is_theme(path) && !is_runtime(path));

        if has_env_changes && let Ok(config_dir) = ConfigPaths::config_dir() {
            secrets::reload_env_files(&config_dir);
            let _ = self.secrets_tx.send(());
        }

        if has_theme_changes {
            load_themes(self.config_service.config(), &themes_dir);
        }

        if has_main_config_changes {
            self.reload_main_config().await?;
        } else if has_runtime_changes {
            self.reload_runtime_only().await?;
        }

        Ok(())
    }

    async fn reload_main_config(&self) -> Result<(), Error> {
        let config = self.config_service.config();

        let config_path = ConfigPaths::main_config();
        let toml_value =
            tokio::task::spawn_blocking(move || Config::load_toml_with_imports(&config_path))
                .await
                .map_err(|source| Error::TaskJoin { source })??;

        config.reset_config_layer();
        config.apply_config_layer(&toml_value, "");

        config.reset_runtime_layer();
        let runtime_path = ConfigPaths::runtime_config();
        let runtime_result =
            tokio::task::spawn_blocking(move || ConfigService::load_toml_file(&runtime_path))
                .await
                .map_err(|source| Error::TaskJoin { source })?;

        if let Ok(runtime_toml) = runtime_result {
            let _ = config.apply_runtime_layer(&runtime_toml, "");
        }

        config.commit_config_reload();

        Ok(())
    }

    async fn reload_runtime_only(&self) -> Result<(), Error> {
        let config = self.config_service.config();
        let runtime_path = ConfigPaths::runtime_config();

        let runtime_result =
            tokio::task::spawn_blocking(move || ConfigService::load_toml_file(&runtime_path))
                .await
                .map_err(|source| Error::TaskJoin { source })?;

        let Ok(runtime_toml) = runtime_result else {
            return Ok(());
        };

        config.reset_runtime_layer();
        let _ = config.apply_runtime_layer(&runtime_toml, "");
        config.commit_config_reload();

        Ok(())
    }
}

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

async fn run_debounced_event_loop(watcher: FileWatcher, mut rx: mpsc::UnboundedReceiver<Event>) {
    use tokio::time::{Instant, sleep_until};

    let mut pending_paths: Vec<PathBuf> = Vec::new();
    let mut deadline: Option<Instant> = None;

    loop {
        let maybe_event = match deadline {
            Some(d) => tokio::select! {
                biased;
                event = rx.recv() => event,
                () = sleep_until(d) => None,
            },
            None => rx.recv().await,
        };

        match maybe_event {
            Some(event) if FileWatcher::should_reload(&event) => {
                accumulate_paths(&mut pending_paths, event.paths);
                deadline = Some(Instant::now() + DEBOUNCE_DURATION);
            }
            Some(_) => {}
            None if deadline.is_some() => {
                flush_pending(&watcher, &mut pending_paths).await;
                deadline = None;
            }
            None => break,
        }
    }
}

fn accumulate_paths(pending: &mut Vec<PathBuf>, new_paths: Vec<PathBuf>) {
    for path in new_paths {
        if !pending.contains(&path) {
            pending.push(path);
        }
    }
}

async fn flush_pending(watcher: &FileWatcher, pending_paths: &mut Vec<PathBuf>) {
    debug!(?pending_paths, "Debounce complete, reloading config");

    if let Err(e) = watcher.reload_and_sync(pending_paths).await {
        error!("config reload failed:\n{e}");
    }

    pending_paths.clear();
}
