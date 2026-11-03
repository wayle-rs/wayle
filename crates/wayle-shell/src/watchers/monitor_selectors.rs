//! Warns about invalid monitor selectors in config.

use std::collections::HashSet;

use futures::{StreamExt, stream};
use tracing::warn;
use wayle_config::{
    Config,
    schemas::{modules::notification::PopupMonitor, osd::OsdMonitor},
};
use wayle_core::monitor::MonitorSelector;

use crate::shell::ShellServices;

pub(crate) fn spawn(services: &ShellServices) {
    let config = services.config.config().clone();

    let mut changes = stream::select_all([
        config.bar.layout.watch().map(drop).boxed(),
        config.wallpaper.monitors.watch().map(drop).boxed(),
        config.osd.monitor.watch().map(drop).boxed(),
        config
            .modules
            .notifications
            .popup_monitor
            .watch()
            .map(drop)
            .boxed(),
        config.styling.theming_monitor.watch().map(drop).boxed(),
    ]);

    tokio::spawn(async move {
        let mut reported = HashSet::new();

        while changes.next().await.is_some() {
            for (option, value) in monitor_values(&config) {
                if let Err(err) = value.parse::<MonitorSelector>()
                    && reported.insert((option, value.clone()))
                {
                    warn!(option, value, error = %err, "invalid monitor selector");
                }
            }
        }
    });
}

fn monitor_values(config: &Config) -> Vec<(&'static str, String)> {
    let mut values: Vec<(&'static str, String)> = config
        .bar
        .layout
        .get()
        .into_iter()
        .map(|layout| ("bar.layout.monitor", layout.monitor))
        .collect();

    values.extend(
        config
            .wallpaper
            .monitors
            .get()
            .into_iter()
            .filter(|entry| !entry.name.is_empty())
            .map(|entry| ("wallpaper.monitors.name", entry.name)),
    );

    if let OsdMonitor::Connector(target) = config.osd.monitor.get() {
        values.push(("osd.monitor", target));
    }
    if let PopupMonitor::Connector(target) = config.modules.notifications.popup_monitor.get() {
        values.push(("modules.notifications.popup-monitor", target));
    }

    let theming = config.styling.theming_monitor.get();
    if !theming.is_empty() {
        values.push(("styling.theming-monitor", theming));
    }

    values
}
