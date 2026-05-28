//! Hot-reload watcher for user style overrides.
//!
//! [`spawn`] watches `~/.config/wayle/styles/` recursively and rebuilds the
//! full CSS bundle (static + theme + user) whenever a `.scss` or `.css` file
//! in the tree changes. The rebuilt bundle is dispatched to the consumer's
//! relm4 component via a caller-supplied message constructor.
//!
//! A failed user-SCSS compile leaves the previous bundle in place rather
//! than dropping the user's overrides.

use std::{path::Path, sync::Arc, time::Duration};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use relm4::{Component, ComponentSender, Sender};
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until},
};
use tracing::{debug, error, info};
use wayle_config::{Config, ConfigService, infrastructure::paths::ConfigPaths};

use crate::{
    Error, STATIC_CSS, ensure_user_styles_scaffold, theme_css, try_user_css, user_styles_dir,
};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Spawns a user-style file watcher tied to a relm4 component's lifecycle.
///
/// On each `.scss` or `.css` change (debounced 100ms), the full CSS bundle
/// is recompiled off the main thread and `into_cmd(css)` is dispatched to
/// the component's command channel. The watcher shuts down with the
/// component.
///
/// Idempotently scaffolds `~/.config/wayle/styles/index.scss` if missing.
pub fn spawn<C>(
    sender: &ComponentSender<C>,
    config_service: Arc<ConfigService>,
    into_cmd: impl Fn(String) -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let config_dir = match ConfigPaths::config_dir() {
        Ok(dir) => dir,
        Err(err) => {
            error!(error = %err, "cannot resolve config dir; user style watcher disabled");
            return;
        }
    };

    ensure_user_styles_scaffold(&config_dir);

    let Some(styles_dir) = user_styles_dir(&config_dir) else {
        info!("no ~/.config/wayle/styles/ directory; user style watcher idle");
        return;
    };

    let Some((watcher, rx)) = make_watcher(&styles_dir) else {
        return;
    };

    info!(path = %styles_dir.display(), "user style watcher started");

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        tokio::select! {
            _ = &mut shutdown_fut => {}
            () = run_event_loop(watcher, rx, out, config_service, into_cmd) => {}
        }
    });
}

fn make_watcher(
    styles_dir: &Path,
) -> Option<(Arc<RecommendedWatcher>, mpsc::UnboundedReceiver<Event>)> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut watcher = match notify::recommended_watcher(move |result: Result<Event, _>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    }) {
        Ok(watcher) => watcher,
        Err(err) => {
            error!(error = %err, "cannot create user style watcher");
            return None;
        }
    };

    if let Err(err) = watcher.watch(styles_dir, RecursiveMode::Recursive) {
        error!(
            error = %err,
            path = %styles_dir.display(),
            "cannot watch user styles directory",
        );
        return None;
    }

    Some((Arc::new(watcher), rx))
}

fn touches_stylesheet(event: &Event) -> bool {
    let hit = event.paths.iter().any(|path| {
        path.extension()
            .is_some_and(|ext| ext == "scss" || ext == "css")
    });

    hit && matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

async fn run_event_loop<M>(
    _watcher: Arc<RecommendedWatcher>,
    mut rx: mpsc::UnboundedReceiver<Event>,
    out: Sender<M>,
    config_service: Arc<ConfigService>,
    into_cmd: impl Fn(String) -> M + Send + Sync + 'static,
) where
    M: Send + 'static,
{
    let mut deadline: Option<Instant> = None;

    loop {
        let maybe_event = match deadline {
            Some(until) => tokio::select! {
                biased;
                event = rx.recv() => event,
                () = sleep_until(until) => None,
            },
            None => rx.recv().await,
        };

        match maybe_event {
            Some(event) if touches_stylesheet(&event) => {
                deadline = Some(Instant::now() + DEBOUNCE_DURATION);
            }

            Some(_) => {}

            None if deadline.is_some() => {
                recompile(&out, &config_service, &into_cmd).await;
                deadline = None;
            }

            None => break,
        }
    }
}

async fn recompile<M>(
    out: &Sender<M>,
    config_service: &ConfigService,
    into_cmd: &(impl Fn(String) -> M + Send + Sync + 'static),
) where
    M: Send + 'static,
{
    let config = config_service.config().clone();

    let result = tokio::task::spawn_blocking(move || compile_bundle(&config)).await;

    match result {
        Ok(Ok(css)) => {
            debug!("user style change reloaded");
            let _ = out.send(into_cmd(css));
        }
        Ok(Err(err)) => {
            error!(error = %err, "user styles compile failed; keeping previous bundle");
        }
        Err(err) => {
            error!(error = %err, "user style rebuild task panicked");
        }
    }
}

fn compile_bundle(config: &Config) -> Result<String, Error> {
    let palette = config.styling.palette();
    let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);

    let user = match ConfigPaths::config_dir() {
        Ok(dir) => try_user_css(&dir)?,
        Err(_) => String::new(),
    };

    Ok(format!("{STATIC_CSS}\n{theme}\n{user}"))
}
