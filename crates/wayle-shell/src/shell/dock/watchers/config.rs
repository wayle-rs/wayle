//! Dock config change watcher (position, visibility).

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::SubscribeChanges;

use super::super::{Dock, DockCmd};

pub(crate) fn spawn(
    sender: &ComponentSender<Dock>,
    services: &crate::shell::services::ShellServices,
) {
    let config = services.config.config().clone();
    let dock = &config.dock;

    let (tx, mut rx) = mpsc::unbounded_channel();

    dock.position.subscribe_changes(tx);

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(()) = rx.recv() => {
                    let _ = out.send(DockCmd::PositionChanged);
                }
            }
        }
    });
}
