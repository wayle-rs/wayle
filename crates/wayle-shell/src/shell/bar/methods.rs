//! Bar component methods: layer-shell positioning, layout diffing,
//! orientation, and section rebuilding.

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, LayerShell};
use relm4::{ComponentSender, factory::FactoryVecDeque, gtk, gtk::gdk};
use tracing::debug;
use wayle_config::{
    Config, ConfigService,
    schemas::{
        bar::{BarItem, BarLayout, Location},
        styling::Spacing,
    },
};
use wayle_widgets::prelude::BarSettings;

use super::{
    Bar, BarCmd, BarInput,
    autohide::{AutohideAction, AutohideState, HoverTrigger},
    dropdowns::DropdownRegistry,
    factory::{BarItemFactory, BarItemFactoryInit},
    watchers,
};
use crate::{
    services::shell_ipc::ShellIpcState,
    shell::{helpers::layer_shell::apply_layer, services::ShellServices},
};

impl Bar {
    pub(super) fn apply_anchors(window: &gtk::Window, location: Location) {
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
    }

    pub(super) fn apply_exclusive_zone(window: &gtk::Window, exclusive: bool) {
        if exclusive {
            window.auto_exclusive_zone_enable();
        } else {
            window.set_exclusive_zone(0);
        }
    }

    pub(super) fn apply_css_classes(
        window: &gtk::Window,
        monitor: &gdk::Monitor,
        location: Location,
        is_floating: bool,
    ) {
        if let Some(connector) = monitor.connector() {
            window.add_css_class(&connector);
            window.set_namespace(Some(&format!("wayle-bar-{connector}")));
        }

        window.add_css_class(location.css_class());

        if is_floating {
            window.add_css_class("floating");
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_orientations(
        center_box: &gtk::CenterBox,
        left_box: &gtk::Box,
        middle_box: &gtk::Box,
        right_box: &gtk::Box,
        left_factory: &gtk::Box,
        center_factory: &gtk::Box,
        right_factory: &gtk::Box,
        is_vertical: bool,
    ) {
        let orientation = if is_vertical {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        };

        center_box.set_orientation(orientation);
        left_box.set_orientation(orientation);
        middle_box.set_orientation(orientation);
        right_box.set_orientation(orientation);

        left_factory.set_orientation(orientation);
        center_factory.set_orientation(orientation);
        right_factory.set_orientation(orientation);

        left_box.set_vexpand(false);
        middle_box.set_vexpand(false);
        right_box.set_vexpand(false);
        left_box.set_hexpand(false);
        middle_box.set_hexpand(false);
        right_box.set_hexpand(false);
    }

    pub(super) fn suppress_alt_focus(window: &gtk::Window) {
        window.connect_focus_visible_notify(|window| {
            if window.gets_focus_visible() {
                window.set_focus_visible(false);
            }
        });

        window.connect_mnemonics_visible_notify(|window| {
            if window.is_mnemonics_visible() {
                window.set_mnemonics_visible(false);
            }
        });
    }

    /// Applies layer-shell setup, anchors, css classes, focus suppression,
    /// and monitor-invalidation teardown to the bar's root window.
    pub(super) fn configure_root_window(
        root: &gtk::Window,
        monitor: &gdk::Monitor,
        config_service: &Arc<ConfigService>,
        location: Location,
        is_floating: bool,
    ) {
        root.init_layer_shell();
        apply_layer(
            root,
            config_service.config().bar.layer.get(),
            config_service,
        );
        root.set_keyboard_mode(KeyboardMode::None);
        Self::apply_exclusive_zone(root, config_service.config().bar.exclusive.get());
        root.set_monitor(Some(monitor));
        Self::apply_anchors(root, location);
        Self::apply_css_classes(root, monitor, location, is_floating);
        Self::suppress_alt_focus(root);

        let window = root.clone();
        monitor.connect_invalidate(move |_| {
            window.destroy();
        });
    }

    /// Spawns all of this bar's config-watcher background commands.
    pub(super) fn spawn_watchers(
        sender: &ComponentSender<Self>,
        monitor: &gdk::Monitor,
        config_service: &Arc<ConfigService>,
        ipc_state: &ShellIpcState,
    ) {
        watchers::layout::spawn(sender, monitor, config_service, ipc_state);
        watchers::dropdowns::spawn(sender, config_service);
        watchers::exclusive::spawn(sender, config_service);
        watchers::layer::spawn(sender, config_service);
        watchers::autohide::spawn(sender, config_service);
    }

    /// Wires an `EventControllerMotion` on `root` so hovering the bar's own
    /// surface drives the autohide state machine (`BarInput::HoverEnter`,
    /// `HoverMotion`, `HoverLeave`).
    pub(super) fn attach_motion_controller(root: &gtk::Window, sender: &ComponentSender<Self>) {
        let motion = gtk::EventControllerMotion::new();

        let enter_sender = sender.input_sender().clone();
        motion.connect_enter(move |_, _, _| {
            enter_sender.emit(BarInput::HoverEnter);
        });

        let motion_sender = sender.input_sender().clone();
        motion.connect_motion(move |_, _, _| {
            motion_sender.emit(BarInput::HoverMotion);
        });

        let leave_sender = sender.input_sender().clone();
        motion.connect_leave(move |_| {
            leave_sender.emit(BarInput::HoverLeave);
        });

        root.add_controller(motion);
    }

    /// Builds the initial `AutohideState` and, if autohide is enabled, the
    /// `HoverTrigger` surface for `monitor`.
    ///
    /// If autohide starts enabled, this also arms the initial hide timer --
    /// mirroring the runtime-enable path (`AutohideChanged(true)` ->
    /// `AutohideState::set_enabled`) -- so a bar that starts with autohide
    /// already on begins its inactivity countdown immediately instead of
    /// staying permanently revealed until the first hover-enter-then-leave
    /// cycle. The trigger itself starts unmapped: a fresh `AutohideState`
    /// always starts revealed, and the trigger is only ever mapped while the
    /// bar is actually hidden (see `apply_effective_visibility`).
    pub(super) fn init_autohide(
        config: &Config,
        monitor: &gdk::Monitor,
        location: Location,
        sender: &ComponentSender<Self>,
    ) -> (AutohideState, Option<HoverTrigger>) {
        let enabled = config.bar.autohide.get();
        let timeout_ms = config.bar.autohide_timeout.get();
        let mut state = AutohideState::new(enabled, timeout_ms);

        if enabled
            && let AutohideAction::ScheduleTimer { duration_ms, token } = state.set_enabled(true)
        {
            Self::schedule_autohide_timer(sender, duration_ms, token);
        }

        let trigger = enabled.then(|| {
            let size = config.bar.autohide_trigger_size.get();
            let size_px = Self::trigger_size_px(config, size);
            let trigger = Self::create_hover_trigger(monitor, location, size_px, sender);
            trigger.set_visible(false);
            trigger
        });

        (state, trigger)
    }

    /// Builds the `DropdownRegistry`, wiring its open/close notifications to
    /// `BarInput::DropdownOpened`/`DropdownClosed`, and warms all dropdowns.
    pub(super) fn init_dropdowns(
        services: &ShellServices,
        sender: &ComponentSender<Self>,
    ) -> Rc<DropdownRegistry> {
        let toggled_sender = sender.input_sender().clone();
        let registry = Rc::new(DropdownRegistry::new(services, move |opened| {
            let input = if opened {
                BarInput::DropdownOpened
            } else {
                BarInput::DropdownClosed
            };
            toggled_sender.emit(input);
        }));
        registry.warm_all();
        registry
    }

    pub(super) fn apply_layout(&mut self, new_layout: BarLayout, root: &gtk::Window) {
        if self.layout == new_layout {
            return;
        }

        if self.layout.show != new_layout.show {
            self.layout.show = new_layout.show;
            self.apply_effective_visibility(root);
        }

        let settings = &self.settings;
        let services = &self.services;
        let dropdowns = &self.dropdowns;

        if self.layout.left != new_layout.left {
            rebuild_section(
                &mut self.left,
                &self.layout.left,
                &new_layout.left,
                settings,
                services,
                dropdowns,
            );
        }

        if self.layout.center != new_layout.center {
            rebuild_section(
                &mut self.center,
                &self.layout.center,
                &new_layout.center,
                settings,
                services,
                dropdowns,
            );
        }

        if self.layout.right != new_layout.right {
            rebuild_section(
                &mut self.right,
                &self.layout.right,
                &new_layout.right,
                settings,
                services,
                dropdowns,
            );
        }

        self.layout = new_layout;
    }

    /// Recomputes real window visibility as the AND of layout-driven
    /// visibility and the autohide state's reveal state, and syncs the
    /// hover trigger's own visibility.
    ///
    /// The trigger is only mapped while the bar is actually hidden
    /// (`layout.show && enabled && !is_revealed`). Keeping it mapped
    /// regardless of reveal state would leave a `Layer::Top` strip mapped
    /// over the bar's own edge even while the bar is fully revealed, able to
    /// steal clicks from bar buttons in the overlap depending on compositor
    /// stacking order.
    ///
    /// This is the single place that calls `root.set_visible`, so bar
    /// visibility always goes through a real map/unmap rather than a
    /// resize/opacity trick (required for compositor animation
    /// compatibility).
    pub(super) fn apply_effective_visibility(&self, root: &gtk::Window) {
        let is_revealed = self.autohide_state.is_revealed();
        root.set_visible(self.layout.show && is_revealed);
        if let Some(trigger) = &self.hover_trigger {
            trigger
                .set_visible(self.layout.show && self.autohide_state.is_enabled() && !is_revealed);
        }
    }

    /// Applies the side effect(s) of an [`AutohideAction`] returned by an
    /// `AutohideState` transition.
    ///
    /// Visibility is resynced unconditionally rather than only for
    /// `Reveal`/`Hide`: `root.set_visible`/`trigger.set_visible` are no-ops
    /// when the value didn't change, and some transitions (e.g.
    /// `AutohideState::on_trigger_enter`) reveal the bar as a side effect of
    /// an action that isn't itself `Reveal` (it returns `ScheduleTimer`,
    /// since it also arms the fallback hide timer in the same step) -- so a
    /// match on just `Reveal | Hide` would silently skip the map/unmap call.
    pub(super) fn handle_autohide_action(
        &self,
        action: AutohideAction,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        debug!(
            ?action,
            revealed = self.autohide_state.is_revealed(),
            enabled = self.autohide_state.is_enabled(),
            cursor_inside = self.autohide_state.is_cursor_inside(),
            "autohide action"
        );

        self.apply_effective_visibility(root);

        if let AutohideAction::ScheduleTimer { duration_ms, token } = action {
            Self::schedule_autohide_timer(sender, duration_ms, token);
        }
    }

    /// Spawns a cancelable (via token staleness, checked in
    /// `AutohideState::on_timeout`) sleep that reports back once `duration_ms`
    /// elapses, unless the component shuts down first.
    ///
    /// `duration_ms` is clamped to a sane minimum here, at the point of use,
    /// rather than only at config/UI-input time: a near-zero timeout with the
    /// cursor parked on the trigger strip produces a reveal/hide/remap loop
    /// that hammers the compositor with surface churn, regardless of how the
    /// value got set (config file, settings UI, or future IPC).
    fn schedule_autohide_timer(sender: &ComponentSender<Self>, duration_ms: u32, token: u64) {
        const MIN_TIMEOUT_MS: u32 = 200;
        let duration_ms = duration_ms.max(MIN_TIMEOUT_MS);
        sender.command(move |out, shutdown| async move {
            let sleep_fut =
                tokio::time::sleep(tokio::time::Duration::from_millis(u64::from(duration_ms)));
            tokio::pin!(sleep_fut);
            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            tokio::select! {
                () = &mut shutdown_fut => {},
                () = &mut sleep_fut => {
                    let _ = out.send(BarCmd::AutohideTimeout(token));
                }
            }
        });
    }

    /// Creates a `HoverTrigger` surface whose `on_hover` callback reveals the
    /// bar by emitting `BarInput::TriggerHover` -- deliberately distinct from
    /// `BarInput::HoverEnter`, since the trigger is a separate surface from
    /// the bar's own window (see `AutohideState::on_trigger_enter`).
    pub(super) fn create_hover_trigger(
        monitor: &gdk::Monitor,
        location: Location,
        size_px: f32,
        sender: &ComponentSender<Self>,
    ) -> HoverTrigger {
        let input_sender = sender.input_sender().clone();
        HoverTrigger::new(monitor, location, size_px, move || {
            input_sender.emit(BarInput::TriggerHover);
        })
    }

    /// Creates or destroys the `HoverTrigger` window to match `enabled`,
    /// then resyncs effective visibility. Keeping the trigger's lifecycle
    /// tied to the `autohide` setting means a disabled bar never carries a
    /// hover-trigger layer-shell surface at all.
    pub(super) fn sync_hover_trigger(
        &mut self,
        enabled: bool,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        if enabled {
            if self.hover_trigger.is_none() {
                let config = self.services.config.config();
                let location = config.bar.location.get();
                let size = config.bar.autohide_trigger_size.get();
                let size_px = Self::trigger_size_px(config, size);
                self.hover_trigger = Some(Self::create_hover_trigger(
                    &self.monitor,
                    location,
                    size_px,
                    sender,
                ));
            }
        } else {
            // `HoverTrigger::drop` destroys the layer-shell surface, so
            // dropping `existing` here is sufficient -- no separate explicit
            // `destroy()` call needed.
            self.hover_trigger.take();
        }

        self.apply_effective_visibility(root);
    }

    /// Re-applies edge and thickness to the hover trigger for the newly
    /// changed `size`. No-op if autohide is disabled (no trigger exists).
    pub(super) fn refresh_trigger_geometry(&self, size: Spacing) {
        let Some(trigger) = &self.hover_trigger else {
            return;
        };
        let config = self.services.config.config();
        let location = config.bar.location.get();
        trigger.update_geometry(location, Self::trigger_size_px(config, size));
    }

    /// Resolves the trigger size directly as literal pixels.
    ///
    /// Unlike other bar spacing values (which are rem-based and scale with
    /// `bar.scale`), `autohide-trigger-size` is documented and intended as a
    /// raw pixel thickness for a deliberately thin, near-invisible hover
    /// strip. Do NOT route this through `styling::rem_to_px_rounded` -- doing
    /// so previously multiplied the default `2.0` by `REM_BASE` (16), turning
    /// a ~2px strip into a 32px one and risking a reveal/hide oscillation
    /// where the cursor sits inside the trigger's footprint without ever
    /// reaching the bar itself.
    pub(super) fn trigger_size_px(_config: &Config, size: Spacing) -> f32 {
        size.value()
    }
}

/// Updates a bar section to match a new layout, only touching modules
/// that actually changed. Modules that stay in the config are left alone
/// (not destroyed and recreated), so they keep their widgets and state.
///
/// Two passes:
///
/// 1. **Remove** - walk the old list, drop anything not in the new list.
///    Uses a shrinking copy of the new list to handle duplicates correctly.
///
/// 2. **Place** - walk the new list by position. Skip if the right module
///    is already there, move it if it exists at a wrong position, or
///    create it if it's new.
fn rebuild_section(
    factory: &mut FactoryVecDeque<BarItemFactory>,
    old_layout: &[BarItem],
    new_layout: &[BarItem],
    settings: &BarSettings,
    services: &ShellServices,
    dropdowns: &Rc<DropdownRegistry>,
) {
    let mut guard = factory.guard();

    let mut remaining: Vec<&BarItem> = new_layout.iter().collect();
    let mut removal_cursor = 0;

    for old_item in old_layout {
        if let Some(matched) = remaining.iter().position(|item| *item == old_item) {
            remaining.remove(matched);
            removal_cursor += 1;
        } else {
            guard.remove(removal_cursor);
        }
    }

    for (target_position, target_item) in new_layout.iter().enumerate() {
        let already_correct = guard
            .get(target_position)
            .is_some_and(|module| module.matches(target_item));

        if already_correct {
            continue;
        }

        let current_position = (target_position..guard.len()).find(|&position| {
            guard
                .get(position)
                .is_some_and(|module| module.matches(target_item))
        });

        match current_position {
            Some(position) => guard.move_to(position, target_position),

            None => {
                guard.insert(
                    target_position,
                    BarItemFactoryInit {
                        item: target_item.clone(),
                        settings: settings.clone(),
                        services: services.clone(),
                        dropdowns: dropdowns.clone(),
                    },
                );
            }
        }
    }
}
