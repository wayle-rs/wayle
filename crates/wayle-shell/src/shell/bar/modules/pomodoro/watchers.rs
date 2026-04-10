use std::time::Duration;

use relm4::ComponentSender;
use tokio::time::interval;
use tokio_stream::wrappers::{IntervalStream, WatchStream};
use wayle_config::schemas::modules::PomodoroConfig;
use wayle_widgets::watch;

use super::{PomodoroCmd, PomodoroModule};
use crate::shell::SharedPomodoroState;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<PomodoroModule>,
    config: &PomodoroConfig,
    state: &SharedPomodoroState,
) {
    let state_tick = state.clone();
    let interval_stream = IntervalStream::new(interval(Duration::from_secs(1)));
    watch!(sender, [interval_stream], |out| {
        if let Some(snapshot) = state_tick.tick() {
            let _ = out.send(PomodoroCmd::StateChanged(snapshot));
        }
    });

    let state_stream = WatchStream::new(state.subscribe());
    let state_watch = state.clone();
    watch!(sender, [state_stream], |out| {
        let _ = out.send(PomodoroCmd::StateChanged(state_watch.snapshot()));
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(PomodoroCmd::UpdateIcon(icon_name.get().clone()));
    });

    let work_color = config.work_color.clone();
    let short_break_color = config.short_break_color.clone();
    let long_break_color = config.long_break_color.clone();
    let state_colors = state.clone();
    watch!(
        sender,
        [
            work_color.watch(),
            short_break_color.watch(),
            long_break_color.watch()
        ],
        |out| {
            let _ = out.send(PomodoroCmd::StateChanged(state_colors.snapshot()));
        }
    );

    let work = config.work_duration.clone();
    let short_break = config.short_break_duration.clone();
    let long_break = config.long_break_duration.clone();
    let cycles = config.cycles_before_long_break.clone();

    watch!(
        sender,
        [
            work.watch(),
            short_break.watch(),
            long_break.watch(),
            cycles.watch()
        ],
        |out| {
            let _ = out.send(PomodoroCmd::UpdateDurations {
                work: work.get(),
                short_break: short_break.get(),
                long_break: long_break.get(),
                cycles: cycles.get(),
            });
        }
    );
}
