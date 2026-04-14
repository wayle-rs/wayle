use relm4::ComponentSender;
use tokio_stream::wrappers::WatchStream;
use wayle_config::{ConfigService, schemas::modules::PomodoroConfig};
use wayle_widgets::watch;

use super::{PomodoroDropdown, PomodoroDropdownCmd};
use crate::shell::SharedPomodoroState;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<PomodoroDropdown>,
    shell_config: &std::sync::Arc<ConfigService>,
    config: &PomodoroConfig,
    state: &SharedPomodoroState,
) {
    let scale = shell_config.config().styling.scale.clone();
    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(PomodoroDropdownCmd::ScaleChanged(scale.get().value()));
    });

    let state_stream = WatchStream::new(state.subscribe());
    let state_watch = state.clone();
    watch!(sender, [state_stream], |out| {
        let _ = out.send(PomodoroDropdownCmd::StateChanged(state_watch.snapshot()));
    });

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
            let _ = out.send(PomodoroDropdownCmd::UpdateDurations {
                work: work.get(),
                short_break: short_break.get(),
                long_break: long_break.get(),
                cycles: cycles.get(),
            });
        }
    );
}
