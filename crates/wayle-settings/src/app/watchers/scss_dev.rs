//! Development SCSS hot-reload watcher.
//!
//! Only active when `WAYLE_DEV=1` is set. Watches the SCSS source
//! directory and pushes recompiled CSS to the settings window.

use std::{env, sync::Arc, time::Duration};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use relm4::{ComponentSender, Sender};
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until},
};
use tracing::{debug, error, info};
use wayle_config::ConfigService;
use wayle_styling::{compile_dev, scss_dir, theme_css};

use crate::app::{SettingsApp, SettingsAppMsg};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

pub fn spawn(sender: &ComponentSender<SettingsApp>, config_service: &Arc<ConfigService>) {
    if env::var("WAYLE_DEV").is_err() {
        return;
    }

    let scss_path = scss_dir();

    let (tx, rx) = mpsc::unbounded_channel();

    let mut watcher = match notify::recommended_watcher(move |result: Result<Event, _>| {
        if let Ok(event) = result {
            let _ = tx.send(event);
        }
    }) {
        Ok(watcher) => watcher,
        Err(err) => {
            error!(error = %err, "cannot create SCSS watcher");
            return;
        }
    };

    if let Err(err) = watcher.watch(&scss_path, RecursiveMode::Recursive) {
        error!(error = %err, path = %scss_path.display(), "cannot watch SCSS directory");
        return;
    }

    info!(path = %scss_path.display(), "SCSS dev watcher started");

    let watcher = Arc::new(watcher);
    let input_sender = sender.input_sender().clone();
    let config_service = Arc::clone(config_service);

    tokio::spawn(run_event_loop(watcher, rx, input_sender, config_service));
}

fn should_reload(event: &Event) -> bool {
    let has_scss = event.paths.iter().any(|path| {
        path.extension()
            .is_some_and(|ext| ext == "scss" || ext == "css")
    });

    has_scss
        && matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        )
}

async fn run_event_loop(
    _watcher: Arc<RecommendedWatcher>,
    mut rx: mpsc::UnboundedReceiver<Event>,
    sender: Sender<SettingsAppMsg>,
    config_service: Arc<ConfigService>,
) {
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
                recompile(&sender, &config_service);
                deadline = None;
            }

            None => break,
        }
    }
}

fn recompile(sender: &relm4::Sender<SettingsAppMsg>, config_service: &ConfigService) {
    let config = config_service.config();
    let palette = config.styling.palette();

    match compile_dev() {
        Ok(static_css) => {
            let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);
            let css = format!("{static_css}\n{theme}");
            debug!("SCSS recompiled for settings");
            let _ = sender.send(SettingsAppMsg::DevCssRecompiled(css));
        }

        Err(err) => {
            error!(error = %err, "SCSS compilation failed");
        }
    }
}
