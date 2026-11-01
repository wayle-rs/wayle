//! Development SCSS hot-reload watcher.
//!
//! Watches the SCSS source directory for changes and triggers CSS recompilation.
//! Only active when `WAYLE_DEV=1` environment variable is set.

use std::{sync::Arc, time::Duration};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use relm4::ComponentSender;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use wayle_config::{ConfigService, infrastructure::paths::ConfigPaths};
use wayle_styling::{compile_dev, scss_dir, theme_css, user_css};

use crate::shell::{Shell, ShellCmd, ShellServices};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Spawns the SCSS directory watcher for development hot-reload.
pub fn spawn(sender: &ComponentSender<Shell>, services: &ShellServices) {
    let scss_path = scss_dir();

    let (tx, rx) = mpsc::unbounded_channel();

    let mut watcher = match notify::recommended_watcher(move |result: Result<Event, _>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    }) {
        Ok(w) => w,
        Err(e) => {
            error!(error = %e, "cannot create SCSS watcher");
            return;
        }
    };

    if let Err(e) = watcher.watch(&scss_path, RecursiveMode::Recursive) {
        error!(error = %e, path = %scss_path.display(), "cannot watch SCSS directory");
        return;
    }

    info!(path = %scss_path.display(), "SCSS dev watcher started");

    let watcher = Arc::new(watcher);
    let cmd_sender = sender.command_sender().clone();
    let config_service = services.config.clone();

    tokio::spawn(run_debounced_event_loop(
        watcher,
        rx,
        cmd_sender,
        config_service,
    ));
}

fn should_reload(event: &Event) -> bool {
    let dominated_by_scss = event.paths.iter().any(|path| {
        path.extension()
            .is_some_and(|ext| ext == "scss" || ext == "css")
    });

    dominated_by_scss
        && matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        )
}

async fn run_debounced_event_loop(
    _watcher: Arc<RecommendedWatcher>,
    mut rx: mpsc::UnboundedReceiver<Event>,
    cmd_sender: relm4::Sender<ShellCmd>,
    config_service: Arc<ConfigService>,
) {
    use tokio::time::{Instant, sleep_until};

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
            Some(event) if should_reload(&event) => {
                deadline = Some(Instant::now() + DEBOUNCE_DURATION);
            }
            Some(_) => {}
            None if deadline.is_some() => {
                recompile_css(&cmd_sender, &config_service);
                deadline = None;
            }
            None => break,
        }
    }
}

fn recompile_css(cmd_sender: &relm4::Sender<ShellCmd>, config_service: &ConfigService) {
    let config = config_service.config();
    let palette = config.styling.palette();

    match compile_dev() {
        Ok(static_css) => {
            let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);

            let user = match ConfigPaths::config_dir() {
                Ok(dir) => user_css(&dir),
                Err(err) => {
                    error!(error = %err, "cannot resolve config dir; user styles disabled");
                    String::new()
                }
            };

            let css = format!("{static_css}\n{theme}\n{user}");
            debug!("SCSS recompiled");
            let _ = cmd_sender.send(ShellCmd::CssRecompiled(css));
        }
        Err(e) => {
            error!(error = %e, "SCSS compilation failed");
        }
    }
}
