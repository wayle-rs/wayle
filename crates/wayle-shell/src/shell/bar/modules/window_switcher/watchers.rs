use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::schemas::modules::WindowSwitcherConfig;
use wayle_widgets::watch;
use wayle_wlr_toplevel::WlrToplevelService;

use super::{WindowSwitcherModule, helpers};
use crate::{
    services::ShellIpcState, shell::bar::modules::window_switcher::messages::WindowSwitcherCmd,
};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<WindowSwitcherModule>,
    config: &WindowSwitcherConfig,
    service: &Arc<WlrToplevelService>,
    ipc_state: &ShellIpcState,
) {
    // See the matching comment in the dropdown's watchers.rs: `Property::watch()`
    // always replays the current value first, which would otherwise pop the
    // dropdown open the moment the bar starts.
    let cycle_step = ipc_state.window_cycle_step.clone();
    watch!(sender, [cycle_step.watch().skip(1)], |out| {
        let _ = out.send(WindowSwitcherCmd::EnsureDropdownOpen);
    });

    let toplevels = service.toplevels.clone();
    let ignore_app_id = config.ignore_app_id.clone();
    let hide_when_empty = config.hide_when_empty.clone();
    let icon = config.icon.clone();

    let toplevels_stream = toplevels.watch();
    let ignore_stream = ignore_app_id.watch();
    let hide_stream = hide_when_empty.watch();

    watch!(
        sender,
        [toplevels_stream, ignore_stream, hide_stream],
        |out| {
            let count = helpers::count_visible(&toplevels.get(), &ignore_app_id.get());
            let label = helpers::count_label(count, hide_when_empty.get());
            let _ = out.send(WindowSwitcherCmd::UpdateLabel(label));
        }
    );

    let icon_stream = icon.watch();
    watch!(sender, [icon_stream], |out| {
        let _ = out.send(WindowSwitcherCmd::UpdateIcon(icon.get()));
    });
}
