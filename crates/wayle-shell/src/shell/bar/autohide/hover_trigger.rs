//! Layer-shell surface for detecting pointer hover to trigger bar reveal.

use std::rc::Rc;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::{gtk, gtk::gdk};
use wayle_config::schemas::bar::Location;

/// Thin layer-shell surface mapped at a screen edge to detect pointer entry.
///
/// Calls the `on_hover` callback on every pointer `enter` *and* `motion`
/// over this surface to trigger revealing the hidden bar (see `HoverTrigger::new`'s
/// body for why motion is included too) -- callers must treat `on_hover` as
/// cheap and safe to invoke repeatedly in quick succession, not just once
/// per approach. The surface itself is invisible (0px exclusive zone,
/// keyboard input disabled) and stretches across the edge it anchors to,
/// creating a sensitive strip for hover detection.
pub(crate) struct HoverTrigger {
    window: gtk::Window,
}

impl HoverTrigger {
    /// Create a new hover trigger surface on the given monitor.
    ///
    /// # Arguments
    /// * `monitor` - The monitor to attach this trigger to
    /// * `location` - Which screen edge to anchor to (Top, Bottom, Left, Right)
    /// * `size_px` - Thickness in pixels of the trigger surface
    /// * `on_hover` - Callback invoked on pointer `enter` *and* `motion`
    ///   over the surface (see the struct doc comment) -- must be cheap and
    ///   safe to call repeatedly per approach, not just once
    pub(crate) fn new(
        monitor: &gdk::Monitor,
        location: Location,
        size_px: f32,
        on_hover: impl Fn() + 'static,
    ) -> Self {
        let window = gtk::Window::builder()
            .decorated(false)
            .css_classes(["bar-hover-trigger"])
            .build();

        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(0);
        window.set_monitor(Some(monitor));

        if let Some(connector) = monitor.connector() {
            window.set_namespace(Some(&format!("wayle-bar-trigger-{connector}")));
        }

        Self::apply_trigger_geometry(&window, location, size_px);

        // Live-reproduced: a fast single-stroke swipe to the screen edge
        // sometimes delivered raw `motion` events on this surface with no
        // preceding `enter` (most likely because the compositor doesn't
        // always synthesize one when the pointer is already near the edge
        // as the swipe lands -- not confirmed against the Wayland protocol
        // itself, just observed live). Reacting to `motion` as well as
        // `enter` makes reveal robust to whichever event actually arrives
        // first. This is safe to fire repeatedly, not because
        // `on_trigger_enter` is idempotent (it isn't -- each call bumps the
        // state machine's timer generation and reschedules the fallback
        // hide timer), but because the trigger only stays mapped while the
        // bar is hidden: the first call reveals the bar and unmaps this
        // surface in the same step, so only the handful of events already
        // in flight across that async unmap can still land here.
        let on_hover = Rc::new(on_hover);
        let motion = gtk::EventControllerMotion::new();
        let on_enter = on_hover.clone();
        motion.connect_enter(move |_controller, _x, _y| {
            on_enter();
        });
        motion.connect_motion(move |_controller, _x, _y| {
            on_hover();
        });
        window.add_controller(motion);

        Self { window }
    }

    /// Set the visibility of this trigger surface.
    pub(crate) fn set_visible(&self, visible: bool) {
        self.window.set_visible(visible);
    }

    /// Update the geometry (edge and thickness) of the trigger surface.
    pub(crate) fn update_geometry(&self, location: Location, size_px: f32) {
        Self::apply_trigger_geometry(&self.window, location, size_px);
    }

    fn apply_trigger_geometry(window: &gtk::Window, location: Location, size_px: f32) {
        let size = (size_px.max(1.0)).round() as i32;

        let (anchor_edge, stretch_edges) = match location {
            Location::Top => (Edge::Top, [Edge::Left, Edge::Right]),
            Location::Bottom => (Edge::Bottom, [Edge::Left, Edge::Right]),
            Location::Left => (Edge::Left, [Edge::Top, Edge::Bottom]),
            Location::Right => (Edge::Right, [Edge::Top, Edge::Bottom]),
        };

        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);

        window.set_anchor(anchor_edge, true);
        for edge in stretch_edges {
            window.set_anchor(edge, true);
        }

        if location.is_vertical() {
            window.set_size_request(size, -1);
        } else {
            window.set_size_request(-1, size);
        }
    }
}

impl Drop for HoverTrigger {
    /// Ensures the layer-shell surface is always destroyed when a
    /// `HoverTrigger` is dropped -- including when the owning `Bar`
    /// component itself is dropped on monitor removal (see
    /// `gdk::Monitor::connect_invalidate` in `bar/methods.rs`), which never
    /// explicitly tears down this surface.
    fn drop(&mut self) {
        self.window.destroy();
    }
}
