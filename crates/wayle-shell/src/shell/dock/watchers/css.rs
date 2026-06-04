//! Dock CSS/style watcher.

use std::time::Duration;

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

    dock.visibility.subscribe_changes(tx.clone());
    dock.autohide_delay.subscribe_changes(tx.clone());
    dock.size.subscribe_changes(tx.clone());
    dock.item_padding.subscribe_changes(tx.clone());
    dock.item_rounding.subscribe_changes(tx.clone());
    dock.background_opacity.subscribe_changes(tx.clone());
    dock.bg.subscribe_changes(tx.clone());
    dock.show_running.subscribe_changes(tx);

    sender.command(move |out, shutdown| async move {
        const DEBOUNCE: Duration = Duration::from_millis(50);

        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(()) = rx.recv() => {
                    loop {
                        tokio::select! {
                            () = &mut shutdown_fut => return,
                            Some(()) = rx.recv() => continue,
                            () = tokio::time::sleep(DEBOUNCE) => {
                                let _ = out.send(DockCmd::StyleChanged);
                                break;
                            }
                        }
                    }
                }
            }
        }
    });
}
