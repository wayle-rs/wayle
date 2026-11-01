use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use relm4::ComponentSender;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;
use tracing::warn;
use wayle_config::schemas::modules::WorldClockConfig;
use wayle_widgets::watch;

use super::{WorldClockModule, helpers::format_world_clock, messages::WorldClockCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<WorldClockModule>,
    config: &WorldClockConfig,
) {
    let interval_stream = IntervalStream::new(interval(Duration::from_secs(1)));
    let initial_format = config.format.get();
    let initial_label = render_label(&initial_format);
    let prev_label = Arc::new(Mutex::new(initial_label));

    let format = config.format.clone();
    let prev = Arc::clone(&prev_label);
    watch!(sender, [interval_stream], |out| {
        let label = format_world_clock(&format.get()).unwrap_or_default();
        let mut prev = prev.lock().unwrap_or_else(|poison| poison.into_inner());
        if *prev != label {
            *prev = label.clone();
            let _ = out.send(WorldClockCmd::UpdateLabel(label));
        }
    });

    let format = config.format.clone();
    let prev = Arc::clone(&prev_label);
    watch!(sender, [format.watch()], |out| {
        let label = render_label(&format.get());
        let mut prev = prev.lock().unwrap_or_else(|poison| poison.into_inner());
        if *prev != label {
            *prev = label.clone();
            let _ = out.send(WorldClockCmd::UpdateLabel(label));
        }
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(WorldClockCmd::UpdateIcon(icon_name.get().clone()));
    });
}

pub(super) fn render_label(format: &str) -> String {
    match format_world_clock(format) {
        Ok(label) => label,
        Err(err) => {
            warn!(
                error = %err,
                format = %format,
                "world-clock format failed; expected tz() syntax \
                 e.g. {{{{ tz('UTC', '%H:%M %Z') }}}}"
            );
            String::new()
        }
    }
}
