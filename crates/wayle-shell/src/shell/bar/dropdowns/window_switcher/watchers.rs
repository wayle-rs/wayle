use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges};
use wayle_widgets::watch;
use wayle_wlr_toplevel::WlrToplevelService;

use super::{WindowSwitcherDropdown, messages::WindowSwitcherDropdownCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<WindowSwitcherDropdown>,
    service: &Arc<WlrToplevelService>,
    config: &Arc<ConfigService>,
) {
    let toplevels = service.toplevels.clone();
    watch!(sender, [toplevels.watch()], |out| {
        let _ = out.send(WindowSwitcherDropdownCmd::ToplevelsChanged);
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
