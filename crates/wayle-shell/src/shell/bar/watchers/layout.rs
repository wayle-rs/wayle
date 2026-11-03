use std::collections::HashSet;

use futures::{StreamExt, stream};
use relm4::{
    ComponentSender,
    gtk::{gdk, prelude::*},
};
use tracing::{debug, warn};
use wayle_config::{Config, schemas::bar::BarLayout};
use wayle_core::monitor::{MonitorInfo, best_match};

use crate::{
    services::shell_ipc::ShellIpcState,
    shell::{
        bar::{Bar, BarCmd},
        helpers::monitors::monitor_info,
        services::ShellServices,
    },
};

/// Spawns a task for the given `monitor`'s bar that rebuilds its
/// [`BarLayout`] whenever the layout config changes, bar visibility
/// is toggled via IPC, or the monitor's identity changes.
pub(crate) fn spawn(
    sender: &ComponentSender<Bar>,
    monitor: &gdk::Monitor,
    services: &ShellServices,
    ipc_state: &ShellIpcState,
) {
    let config = services.config.config().clone();
    let ipc = ipc_state.clone();
    let connector = monitor
        .connector()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    // gdk::Monitor is not Send, so resolve the fallback identity here
    let fallback = monitor_info(&connector, monitor, None);
    let outputs = services.outputs.clone();

    let mut layout_stream = config.bar.layout.watch();
    let mut hidden_stream = ipc.hidden_bars.watch();
    let mut outputs_stream = match &outputs {
        Some(outputs) => outputs.outputs.watch().boxed(),
        None => stream::pending().boxed(),
    };

    let identity = move || {
        outputs
            .as_ref()
            .and_then(|outputs| outputs.find(&connector))
            .unwrap_or_else(|| fallback.clone())
    };

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        let mut current = identity();

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,

                Some(_) = layout_stream.next() => {}
                Some(_) = hidden_stream.next() => {}
                Some(_) = outputs_stream.next() => {
                    let next = identity();
                    if next == current {
                        continue;
                    }
                    current = next;
                }
            }

            let layout = build_layout(&config, &ipc, &current).unwrap_or_else(|| {
                debug!(connector = %current.connector, "no layout matched, hiding bar");
                hidden_layout()
            });
            let _ = out.send(BarCmd::LayoutLoaded(layout));
        }
    });
}

/// Layout used when no entry matches: no modules, bar hidden.
fn hidden_layout() -> BarLayout {
    BarLayout {
        show: false,
        left: Vec::new(),
        center: Vec::new(),
        right: Vec::new(),
        ..BarLayout::default()
    }
}

fn build_layout(config: &Config, ipc: &ShellIpcState, monitor: &MonitorInfo) -> Option<BarLayout> {
    let layouts = config.bar.layout.get();
    debug!(connector = %monitor.connector, layout_count = layouts.len(), "Loaded bar layouts");

    let mut layout = find_layout(&layouts, monitor)?;

    if ipc.hidden_bars.get().contains(&monitor.connector) {
        layout.show = false;
    }

    Some(layout)
}

/// Finds the layout whose `monitor` selector best matches `monitor` and
/// resolves any `extends` chain into a single flattened layout.
pub(crate) fn find_layout(layouts: &[BarLayout], monitor: &MonitorInfo) -> Option<BarLayout> {
    let layout = best_match(layouts, |candidate| candidate.monitor.as_str(), monitor)?;
    Some(merge_parent(layout, layouts, &mut HashSet::new()))
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

#[cfg(test)]
mod tests {
    use wayle_config::schemas::bar::{BarItem, BarModule, ModuleRef};

    use super::*;

    fn layout(monitor: &str, extends: Option<&str>, left: &[BarModule]) -> BarLayout {
        BarLayout {
            monitor: monitor.to_owned(),
            extends: extends.map(str::to_owned),
            show: true,
            left: left
                .iter()
                .map(|module| BarItem::Module(ModuleRef::Plain(module.clone())))
                .collect(),
            center: Vec::new(),
            right: Vec::new(),
        }
    }

    fn ancor(connector: &str) -> MonitorInfo {
        MonitorInfo::new(
            connector,
            "Ancor Communications Inc",
            "BE24A",
            Some("XYZ789"),
        )
    }

    #[test]
    fn serial_layout_follows_connector_change() {
        let layouts = [
            layout("*", None, &[BarModule::Clock]),
            layout("sn:XYZ789", None, &[BarModule::Battery]),
        ];

        for connector in ["DP-6", "DP-7", "DP-8"] {
            let found = find_layout(&layouts, &ancor(connector)).map(|l| l.monitor);
            assert_eq!(found.as_deref(), Some("sn:XYZ789"));
        }
    }

    #[test]
    fn selector_layout_inherits_from_wildcard() {
        let layouts = [
            layout("*", None, &[BarModule::Clock]),
            layout("desc:Ancor Communications Inc BE24A", Some("*"), &[]),
        ];

        let found = find_layout(&layouts, &ancor("DP-8"));
        assert_eq!(found.map(|l| l.left), Some(layouts[0].left.clone()));
    }

    #[test]
    fn no_match_without_wildcard() {
        let layouts = [layout("DP-6", None, &[BarModule::Clock])];
        assert!(find_layout(&layouts, &ancor("DP-8")).is_none());
    }
}
