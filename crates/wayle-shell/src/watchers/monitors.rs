use std::sync::Arc;

use futures::{StreamExt, stream};
use gdk4::{gio::prelude::ListModelExt, prelude::DisplayExt};
use relm4::{ComponentSender, gtk::gdk};
use tracing::{debug, info};
use wayle_output::OutputService;

use crate::shell::{Shell, ShellCmd};

#[allow(clippy::expect_used)]
pub(crate) fn spawn(sender: &ComponentSender<Shell>, outputs: Option<&Arc<OutputService>>) {
    let display = gdk::Display::default().expect("No GDK display found...");
    let monitors = display.monitors();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<u32>();

    let initial_count = monitors.n_items();
    info!(count = initial_count, "Monitor watcher started");

    monitors.connect_items_changed(move |model, pos, removed, added| {
        let expected_count = model.n_items();
        info!(
            pos,
            removed,
            added,
            total = expected_count,
            "Monitors changed"
        );
        let _ = tx.send(expected_count);
    });

    // identities (e.g. a late serial) can change without a GDK monitor change
    let mut outputs_stream = match outputs {
        Some(outputs) => outputs.outputs.watch().boxed(),
        None => stream::pending().boxed(),
    };

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);
        loop {
            tokio::select! {
                _ = &mut shutdown_fut => break,
                Some(expected_count) = rx.recv() => {
                    debug!(expected_count, "Monitors changed, starting sync");
                    let _ = out.send(ShellCmd::SyncMonitors { expected_count, attempt: 0 });
                }
                Some(_) = outputs_stream.next() => {
                    let _ = out.send(ShellCmd::OutputsChanged);
                }
            }
        }
    });
}
