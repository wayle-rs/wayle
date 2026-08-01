use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    time::Duration,
};

use gdk4::{
    gio::prelude::ListModelExt,
    glib::{self, object::Cast},
    prelude::{DisplayExt, MonitorExt},
};
use relm4::{Controller, gtk::gdk, prelude::*};
use tracing::{debug, info, warn};

use crate::shell::{
    ShellCmd,
    bar::{Bar, BarInit},
    services::ShellServices,
};

pub(crate) type Connector = String;
pub(crate) type BarMap = HashMap<Connector, Controller<Bar>>;

const MAX_SYNC_RETRIES: u32 = 5;
const BASE_RETRY_DELAY_MS: u64 = 50;
const RETRY_BACKOFF_FACTOR: u64 = 2;

#[allow(clippy::expect_used)]
pub(crate) fn current_monitors() -> Vec<(Connector, gdk::Monitor)> {
    let display = gdk::Display::default().expect("No GDK display found...");
    let monitor_list = display.monitors();

    (0..monitor_list.n_items())
        .filter_map(|i| monitor_list.item(i))
        .filter_map(|obj| obj.downcast::<gdk::Monitor>().ok())
        .filter_map(|monitor| match monitor.connector() {
            Some(connector) => Some((connector.to_string(), monitor)),
            None => {
                warn!(
                    model = monitor.model().map(|m| m.to_string()),
                    "GDK monitor has no connector, skipping"
                );
                None
            }
        })
        .collect()
}

pub(crate) fn create_bars(services: &ShellServices) -> BarMap {
    let mut bars = HashMap::new();

    for (connector, monitor) in current_monitors() {
        debug!(connector = %connector, "Creating bar");
        let bar = Bar::builder()
            .launch(BarInit {
                monitor,
                services: services.clone(),
            })
            .detach();
        bars.insert(connector, bar);
    }

    info!(count = bars.len(), "Bars created");
    sync_ipc_state(services, &bars);

    bars
}

/// Checks if any GDK monitors were missed during initial bar creation
/// (race: monitor in list but connector not yet populated) and schedules
/// a deferred sync to pick them up.
#[allow(clippy::expect_used)]
pub(crate) fn schedule_deferred_sync_if_needed<C: Component<CommandOutput = ShellCmd>>(
    bar_count: usize,
    sender: &ComponentSender<C>,
) {
    let display = gdk::Display::default().expect("No GDK display found...");
    let gdk_monitor_count = display.monitors().n_items();

    if (bar_count as u32) >= gdk_monitor_count {
        return;
    }

    warn!(
        bar_count,
        gdk_monitor_count, "Fewer bars than monitors, scheduling deferred sync"
    );

    let cmd_sender = sender.command_sender().clone();
    glib::timeout_add_local_once(Duration::from_millis(BASE_RETRY_DELAY_MS), move || {
        let _ = cmd_sender.send(ShellCmd::SyncMonitors {
            expected_count: gdk_monitor_count,
            attempt: 0,
        });
    });
}

pub(crate) fn sync(
    bars: &mut BarMap,
    services: &ShellServices,
    expected_count: u32,
    attempt: u32,
    retry: impl FnOnce(u32, u32),
) {
    let monitors = current_monitors();
    let found_count = monitors.len() as u32;

    debug!(expected_count, found_count, attempt, "Syncing monitors");

    if found_count < expected_count && attempt < MAX_SYNC_RETRIES {
        retry(expected_count, attempt);
        return;
    }

    if found_count < expected_count {
        warn!(
            found_count,
            expected_count, "Monitor sync incomplete after max retries"
        );
    }

    reconcile_bars(bars, services, monitors);
}

pub(crate) fn schedule_retry<C: Component<CommandOutput = ShellCmd>>(
    expected_count: u32,
    attempt: u32,
    sender: &ComponentSender<C>,
) {
    let delay_ms = BASE_RETRY_DELAY_MS * RETRY_BACKOFF_FACTOR.pow(attempt);
    let next_attempt = attempt + 1;

    debug!(delay_ms, next_attempt, "Scheduling monitor sync retry");

    let cmd_sender = sender.command_sender().clone();
    glib::timeout_add_local_once(Duration::from_millis(delay_ms), move || {
        let _ = cmd_sender.send(ShellCmd::SyncMonitors {
            expected_count,
            attempt: next_attempt,
        });
    });
}

#[allow(clippy::cognitive_complexity)]
fn reconcile_bars(
    bars: &mut BarMap,
    services: &ShellServices,
    monitors: Vec<(Connector, gdk::Monitor)>,
) {
    let active: HashSet<&str> = monitors
        .iter()
        .map(|(connector, _)| connector.as_str())
        .collect();
    debug!(?active, "Reconciling bars");

    let stale: Vec<String> = bars
        .keys()
        .filter(|connector| !active.contains(connector.as_str()))
        .cloned()
        .collect();

    for connector in stale {
        bars.remove(&connector);
        info!(connector = %connector, "Removed bar for disconnected monitor");
    }

    for (connector, monitor) in monitors {
        let Entry::Vacant(entry) = bars.entry(connector) else {
            continue;
        };

        info!(connector = %entry.key(), "Creating bar for new monitor");
        let bar = Bar::builder()
            .launch(BarInit {
                monitor,
                services: services.clone(),
            })
            .detach();
        entry.insert(bar);
    }

    sync_ipc_state(services, bars);

    debug!(bar_count = bars.len(), "Bar reconciliation complete");
}

fn sync_ipc_state(services: &ShellServices, bars: &BarMap) {
    let connectors: Vec<String> = bars.keys().cloned().collect();
    let ipc = services.shell_ipc.state();

    let mut hidden = ipc.hidden_bars.get();
    let before = hidden.len();
    hidden.retain(|connector| connectors.contains(connector));

    if hidden.len() < before {
        ipc.hidden_bars.set(hidden);
    }

    // Drop published dropdown ids for connectors whose bar is gone, so a disconnected
    // monitor doesn't leave a stale entry accumulating in the map across churn.
    let mut ids = ipc.dropdown_ids.get();
    let ids_before = ids.len();
    ids.retain(|connector, _| connectors.contains(connector));
    if ids.len() < ids_before {
        ipc.dropdown_ids.set(ids);
    }

    ipc.connectors.set(connectors);
}
