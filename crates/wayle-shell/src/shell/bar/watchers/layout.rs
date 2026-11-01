use std::{collections::HashSet, sync::Arc};

use futures::StreamExt;
use relm4::{
    ComponentSender,
    gtk::{gdk, prelude::*},
};
use tracing::{debug, warn};
use wayle_config::{Config, ConfigService, schemas::bar::BarLayout};

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

/// Finds the layout matching `connector` (exact match first, then `"*"` wildcard)
/// and resolves any `extends` chain into a single flattened layout.
pub(crate) fn find_layout(layouts: &[BarLayout], connector: &str) -> Option<BarLayout> {
    let mut visited = HashSet::new();

    if let Some(layout) = layouts
        .iter()
        .find(|candidate| candidate.monitor == connector)
    {
        return Some(merge_parent(layout, layouts, &mut visited));
    }

    if let Some(layout) = layouts.iter().find(|candidate| candidate.monitor == "*") {
        return Some(merge_parent(layout, layouts, &mut visited));
    }

    None
}

fn merge_parent(
    layout: &BarLayout,
    all_layouts: &[BarLayout],
    visited: &mut HashSet<String>,
) -> BarLayout {
    let mut resolved = layout.clone();

    let Some(ref extends_name) = layout.extends else {
        return resolved;
    };

    if !visited.insert(extends_name.clone()) {
        warn!(
            layout = %layout.monitor,
            extends = %extends_name,
            "circular extends detected, skipping parent"
        );
        return resolved;
    }

    let Some(parent) = all_layouts
        .iter()
        .find(|candidate| candidate.monitor == *extends_name)
    else {
        return resolved;
    };

    let parent_resolved = merge_parent(parent, all_layouts, visited);

    if resolved.left.is_empty() {
        resolved.left = parent_resolved.left;
    }
    if resolved.center.is_empty() {
        resolved.center = parent_resolved.center;
    }
    if resolved.right.is_empty() {
        resolved.right = parent_resolved.right;
    }

    resolved
}
