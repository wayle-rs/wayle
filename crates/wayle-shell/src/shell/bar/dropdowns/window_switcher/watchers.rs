use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges};
use wayle_widgets::watch;
use wayle_wlr_toplevel::WlrToplevelService;

use super::{WindowSwitcherDropdown, messages::WindowSwitcherDropdownCmd};
use crate::services::ShellIpcState;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<WindowSwitcherDropdown>,
    service: &Arc<WlrToplevelService>,
    config: &Arc<ConfigService>,
    ipc_state: &ShellIpcState,
) {
    let toplevels = service.toplevels.clone();
    watch!(sender, [toplevels.watch()], |out| {
        let _ = out.send(WindowSwitcherDropdownCmd::ToplevelsChanged);
    });

    // `Property::watch()` always replays the current value as the first
    // stream item (a fresh `watch::Receiver` starts "unseen"). Harmless for
    // the other watchers above (re-running the same rebuild is a no-op),
    // but here it would spuriously pop the dropdown open at startup -
    // `skip(1)` drops that replay so only real CLI-triggered increments
    // reach the handler.
    let cycle_step = ipc_state.window_cycle_step.clone();
    watch!(sender, [cycle_step.watch().skip(1)], |out| {
        let _ = out.send(WindowSwitcherDropdownCmd::CycleStep);
    });

    let cycle_commit = ipc_state.window_cycle_commit.clone();
    watch!(sender, [cycle_commit.watch().skip(1)], |out| {
        let _ = out.send(WindowSwitcherDropdownCmd::CycleCommit);
    });

    let cycle_cancel = ipc_state.window_cycle_cancel.clone();
    watch!(sender, [cycle_cancel.watch().skip(1)], |out| {
        let _ = out.send(WindowSwitcherDropdownCmd::CycleCancel);
    });

    let (tx, rx) = mpsc::unbounded_channel();
    config
        .config()
        .modules
        .window_switcher
        .subscribe_changes(tx);
    sender.command(move |out, shutdown| watch_config_changes(rx, out, shutdown));
}

async fn watch_config_changes(
    mut rx: mpsc::UnboundedReceiver<()>,
    out: relm4::Sender<WindowSwitcherDropdownCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            received = rx.recv() => {
                if received.is_none() {
                    return;
                }
                while rx.try_recv().is_ok() {}
                let _ = out.send(WindowSwitcherDropdownCmd::ConfigChanged);
            }
        }
    }
}
