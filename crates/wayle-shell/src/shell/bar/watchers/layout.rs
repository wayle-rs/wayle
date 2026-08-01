use std::sync::Arc;

use futures::StreamExt;
use relm4::{
    ComponentSender,
    gtk::{gdk, prelude::*},
};
use tracing::{debug, warn};
use wayle_config::{
    Config, ConfigService,
    schemas::bar::{BarLayout, find_layout},
};

use crate::{
    services::shell_ipc::ShellIpcState,
    shell::bar::{Bar, BarCmd},
};

/// Spawns a task for the given `monitor`'s bar that rebuilds its
/// [`BarLayout`] whenever the layout config changes or bar visibility
/// is toggled via IPC.
pub(crate) fn spawn(
    sender: &ComponentSender<Bar>,
    monitor: &gdk::Monitor,
    config_service: &Arc<ConfigService>,
    ipc_state: &ShellIpcState,
) {
    let config = config_service.config().clone();
    let ipc = ipc_state.clone();
    let connector = monitor
        .connector()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut layout_stream = config.bar.layout.watch();
    let mut hidden_stream = ipc.hidden_bars.watch();

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,

                Some(_) = layout_stream.next() => {
                    let layout = build_layout(&config, &ipc, &connector).unwrap_or_else(|| {
                        warn!(connector = %connector, "no layout matched, sending empty");
                        BarLayout::default()
                    });
                    let _ = out.send(BarCmd::LayoutLoaded(layout));
                }

                Some(_) = hidden_stream.next() => {
                    let layout = build_layout(&config, &ipc, &connector).unwrap_or_else(|| {
                        warn!(connector = %connector, "no layout matched, sending empty");
                        BarLayout::default()
                    });
                    let _ = out.send(BarCmd::LayoutLoaded(layout));
                }
            }
        }
    });
}

fn build_layout(config: &Config, ipc: &ShellIpcState, connector: &str) -> Option<BarLayout> {
    let layouts = config.bar.layout.get();
    debug!(connector = %connector, layout_count = layouts.len(), "Loaded bar layouts");

    let mut layout = find_layout(&layouts, connector)?;

    if ipc.hidden_bars.get().contains(connector) {
        layout.show = false;
    }

    Some(layout)
}
