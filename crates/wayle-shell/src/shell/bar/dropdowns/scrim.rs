//! A transparent, fullscreen layer-shell surface that catches outside-clicks to
//! dismiss an open dropdown or systray menu — the non-grab replacement for GTK
//! autohide's outside-click dismissal.
//!
//! Why not autohide: on Hyprland the autohide xdg_popup grab freezes input until
//! the next pointer motion after an outside-click and swallows the dismiss-click,
//! so clicking another bar button doesn't open it. Non-grab popovers avoid both,
//! but then get no notification of outside interaction — that's what the scrim
//! provides: a click anywhere on it (empty desktop / other windows) closes the
//! open surface, and Escape over it does too.
//!
//! Mapped only while a surface is open. The scrim is presented (mapped) on `show`
//! and hidden (unmapped) on `hide`, so it exists exactly while a dropdown/menu is
//! open. The [`OpenSurfaceCoordinator`] drives this off its single `current` slot
//! (via `sync_scrim`), and its show-before-close swap keeps `current` populated
//! across a surface swap — so the scrim maps on the first open, stays mapped across
//! swaps (each `show` is a no-op while already visible), and unmaps only on the
//! final close. No ref-count or debounce is needed; the coordinator's swap handling
//! subsumes them.
//!
//! Tradeoff (map/unmap vs. a persistent scrim). Mapping or unmapping a fullscreen
//! layer surface under a *stationary* pointer makes the compositor defer
//! re-evaluating pointer focus until the next motion. A persistent scrim (mapped
//! once, only its input region toggled) was tried to avoid this, but emptying the
//! input region on close hit the very same deferral from the other side: after a
//! dismiss the scrim kept pointer/keyboard focus until the mouse moved — an
//! everyday annoyance. Map/unmap instead pays the cost rarely and only on *open*:
//! unmapping on close forces an immediate focus re-evaluation (the surface is gone),
//! so dismissal is clean, while the residual "open a dropdown on an idle secondary
//! output, its second same-spot click doesn't close until you move the mouse" race
//! is uncommon and strictly less disruptive than a stuck scrim after every dismiss.
//!
//! Stacking. The dropdown/menu popovers are xdg_popups glued to the *bar* surface.
//! For the scrim to catch outside clicks it must sit above application windows; for
//! the popovers and bar buttons to stay clickable they must sit above the scrim.
//! The scrim maps at `overlay` (above fullscreen — dismissal + popup shortcuts keep
//! working while an app is fullscreen); `show` then raises the bar to `overlay` too,
//! *after* the scrim maps, so it (and its popovers) stack above the scrim. On close
//! the bar drops back to its configured layer.
//!
//! Transparent-but-clickable. Input is independent of opacity, but
//! `set_input_region` only marks the region dirty; it reaches the compositor solely
//! on a render frame, so `apply_input_region` calls `queue_render()` after. On
//! wlroots the surface must also commit *painted content* to be hit-tested, so the
//! scrim keeps a ~1/255 alpha background (`.dropdown-scrim` in the base stylesheet)
//! — a full-monitor buffer that is visually imperceptible. The input region is
//! re-applied on every map (`connect_map`), since a fresh surface starts with none.

use std::rc::Rc;

use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk::{self, cairo, glib, prelude::*};

use super::coordinator::OpenSurfaceCoordinator;
use crate::shell::{helpers::layer_shell::apply_layer, services::ShellServices};

pub(crate) struct Scrim {
    window: gtk::Window,
    bar_window: glib::WeakRef<gtk::Window>,
    services: ShellServices,
}

/// Set the scrim's input region to the full monitor and commit it. Called on every
/// map: the scrim only exists while a surface is open, so it always wants to catch
/// outside-clicks across the whole output. `set_input_region` only marks the region
/// dirty; `queue_render` flushes it to the compositor (and paints the ~1/255 alpha
/// buffer wlroots needs to hit-test it).
fn apply_input_region(window: &gtk::Window, monitor: &gtk::gdk::Monitor) {
    let Some(surface) = window.surface() else {
        return;
    };
    let geometry = monitor.geometry();
    let region = cairo::Region::create_rectangle(&cairo::RectangleInt::new(
        0,
        0,
        geometry.width(),
        geometry.height(),
    ));
    surface.set_input_region(Some(&region));
    surface.queue_render();
}

impl Scrim {
    pub(crate) fn new(
        services: &ShellServices,
        monitor: &gtk::gdk::Monitor,
        bar_window: &gtk::Window,
        coordinator: &Rc<OpenSurfaceCoordinator>,
    ) -> Rc<Self> {
        let window = gtk::Window::new();
        window.set_decorated(false);
        // Attach to the running application so the window gets a frame clock and
        // renders/allocates like the other shell surfaces.
        window.set_application(Some(&relm4::main_application()));
        // `.dropdown-scrim` gives it a ~1/255 alpha background (visually
        // transparent, but a committed buffer so wlroots hit-tests it).
        window.add_css_class("dropdown-scrim");

        // A layer-shell window anchored to all edges stretches the *surface*, but
        // GTK allocates the *content* from the child's measured size — an empty
        // catcher measures 0×0. Pin the window to the monitor size so it fills.
        let geometry = monitor.geometry();
        window.set_default_size(geometry.width(), geometry.height());

        window.init_layer_shell();
        window.set_namespace(Some("wayle-dropdown-scrim"));
        // `overlay` so the scrim stays above fullscreen windows (dismissal keeps
        // working while fullscreen); the bar is raised above it in `show`.
        window.set_layer(Layer::Overlay);
        window.set_monitor(Some(monitor));
        // Keyboard follows focus; on-demand while shown (set in `show`).
        window.set_keyboard_mode(KeyboardMode::None);
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            window.set_anchor(edge, true);
        }
        window.set_exclusive_zone(-1);

        let catcher = gtk::Box::new(gtk::Orientation::Vertical, 0);
        catcher.set_hexpand(true);
        catcher.set_vexpand(true);
        // `button(0)` = listen to every button (GtkGestureSingle defaults to primary
        // only): a right/middle-click on the empty desktop dismisses too, matching the
        // bar's any-button dismiss.
        let click = gtk::GestureClick::builder().button(0).build();
        click.connect_pressed({
            let coordinator = Rc::downgrade(coordinator);
            move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                if let Some(coordinator) = coordinator.upgrade() {
                    coordinator.dismiss_current();
                }
            }
        });
        catcher.add_controller(click);
        window.set_child(Some(&catcher));

        // When the pointer is over the scrim (empty desktop) the scrim holds
        // keyboard focus: Escape dismisses the open surface and nav keys drive the
        // systray menu's cascade (mirrors the bar's handler for the popover-focused
        // case; the shared policy lives on the coordinator).
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let coordinator = Rc::downgrade(coordinator);
            move |_, keyval, _, state| {
                coordinator
                    .upgrade()
                    .map_or(gtk::glib::Propagation::Proceed, |coordinator| {
                        coordinator.handle_key_event(keyval, state)
                    })
            }
        });
        window.add_controller(keys);

        // Apply (and commit, via queue_render) the input region each time the
        // surface maps.
        window.connect_map({
            let monitor = monitor.clone();
            move |window| apply_input_region(window, &monitor)
        });

        Rc::new(Self {
            window,
            bar_window: bar_window.downgrade(),
            services: services.clone(),
        })
    }

    /// Show the scrim (idempotent). Maps it, then raises the bar above it so the bar
    /// (and its popovers) stay clickable; both switch to on-demand keyboard. A no-op
    /// while already visible, so a surface swap doesn't re-map or re-layer.
    pub(crate) fn show(&self) {
        if self.window.is_visible() {
            return;
        }
        self.window.set_keyboard_mode(KeyboardMode::OnDemand);
        self.window.present();
        if let Some(bar) = self.bar_window.upgrade() {
            // Re-raise into overlay *after* the scrim maps so the bar (and its
            // popovers) stack above it; give it on-demand keyboard so hovering a
            // popover routes keys there.
            bar.set_layer(Layer::Overlay);
            bar.set_keyboard_mode(KeyboardMode::OnDemand);
        }
    }

    /// Hide the scrim (idempotent) and restore the bar's layer/keyboard. Unmapping
    /// (rather than just clearing the input region) forces the compositor to
    /// re-evaluate pointer focus immediately, so a dismiss doesn't leave the scrim
    /// holding focus until the pointer moves.
    pub(crate) fn hide(&self) {
        if !self.window.is_visible() {
            return;
        }
        self.window.set_visible(false);
        self.window.set_keyboard_mode(KeyboardMode::None);
        if let Some(bar) = self.bar_window.upgrade() {
            bar.set_keyboard_mode(KeyboardMode::None);
            apply_layer(
                &bar,
                self.services.config.config().bar.layer.get(),
                &self.services.config,
            );
        }
    }
}

impl Drop for Scrim {
    fn drop(&mut self) {
        // GTK keeps a ref on mapped toplevels; destroy explicitly so the
        // layer-shell surface goes away with the bar's registry.
        self.window.destroy();
    }
}
