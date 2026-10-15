use std::sync::Arc;

use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::gtk;
use tracing::warn;
use wayle_config::{ConfigService, schemas::general::Layer as ConfigLayer};
use wayle_core::monitor::MonitorSelector;
use wayle_output::OutputService;

use super::monitors::{current_monitors, monitor_info};

/// Clears all layer-shell edge anchors and zeroes margins.
pub(crate) fn reset_anchors(root: &gtk::Window) {
    root.set_anchor(Edge::Top, false);
    root.set_anchor(Edge::Bottom, false);
    root.set_anchor(Edge::Left, false);
    root.set_anchor(Edge::Right, false);

    root.set_margin(Edge::Top, 0);
    root.set_margin(Edge::Bottom, 0);
    root.set_margin(Edge::Left, 0);
    root.set_margin(Edge::Right, 0);
}

/// Maps a config [`ConfigLayer`] to a layer-shell [`Layer`].
pub(crate) fn to_gtk_layer(layer: ConfigLayer) -> Layer {
    match layer {
        ConfigLayer::Background => Layer::Background,
        ConfigLayer::Bottom => Layer::Bottom,
        ConfigLayer::Top => Layer::Top,
        ConfigLayer::Overlay => Layer::Overlay,
    }
}

/// Returns the layer to actually use after honoring `general.tearing-mode`.
///
/// Tearing-mode demotes `Overlay` to `Top` so fullscreen tearing works; other
/// layers pass through unchanged.
pub(crate) fn effective_layer(configured: ConfigLayer, tearing: bool) -> ConfigLayer {
    if tearing && configured == ConfigLayer::Overlay {
        ConfigLayer::Top
    } else {
        configured
    }
}

/// Applies the configured layer, honoring `general.tearing-mode`.
pub(crate) fn apply_layer(
    root: &gtk::Window,
    configured: ConfigLayer,
    config: &Arc<ConfigService>,
) {
    let tearing = config.config().general.tearing_mode.get();
    let layer = effective_layer(configured, tearing);
    root.set_layer(to_gtk_layer(layer));
}

/// Applies the first monitor matching `target` (connector name, `desc:` or
/// `sn:` selector), falling back to primary.
pub(crate) fn apply_monitor_by_selector(
    root: &gtk::Window,
    target: &str,
    outputs: Option<&OutputService>,
) {
    let monitors = current_monitors();
    let primary = monitors.first().map(|(_, monitor)| monitor.clone());

    let selector = match target.parse::<MonitorSelector>() {
        Ok(selector) => selector,
        Err(err) => {
            warn!(target, error = %err, "invalid monitor, falling back to primary");
            root.set_monitor(primary.as_ref());
            return;
        }
    };

    let mut matched = monitors
        .into_iter()
        .filter(|(connector, monitor)| selector.matches(&monitor_info(connector, monitor, outputs)))
        .map(|(_, monitor)| monitor);

    let first = matched.next();
    if first.is_none() {
        warn!(
            target,
            "configured monitor not found, falling back to primary"
        );
    } else if matched.next().is_some() {
        warn!(target, "several monitors match, using the first");
    }

    root.set_monitor(first.or(primary).as_ref());
}

/// Assigns the first available monitor to the layer-shell surface.
pub(crate) fn apply_primary_monitor(root: &gtk::Window) {
    let monitors = current_monitors();

    let primary = monitors.into_iter().next().map(|(_, monitor)| monitor);

    root.set_monitor(primary.as_ref());
}
