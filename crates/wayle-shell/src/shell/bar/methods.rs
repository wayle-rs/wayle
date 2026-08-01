//! Bar component methods: layer-shell positioning, layout diffing,
//! orientation, and section rebuilding.

use std::{collections::HashMap, rc::Rc};

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, LayerShell};
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk};
use tracing::debug;
use wayle_config::{
    ClickAction, Config,
    schemas::bar::{BarItem, BarLayout, Location},
};

use crate::services::shell_ipc::{DropdownAction, ShellIpcState};
use wayle_widgets::prelude::BarSettings;

use super::{
    Bar,
    dropdowns::{DropdownOpener, DropdownRegistry},
    factory::{BarItemFactory, BarItemFactoryInit},
};
use crate::shell::services::ShellServices;

impl Bar {
    /// Whether this bar should be visible at startup: its layout says `show` and its
    /// connector isn't in the CLI-hidden set.
    pub(super) fn visible_on_startup(
        config: &Config,
        ipc_state: &ShellIpcState,
        connector: &str,
    ) -> bool {
        wayle_config::schemas::bar::find_layout(&config.bar.layout.get(), connector)
            .is_some_and(|layout| layout.show)
            && !ipc_state.hidden_bars.get().contains(connector)
    }

    /// Install the bar's keyboard + click dismissal controllers (both capture-phase).
    ///
    /// Keyboard: when the bar (or one of its popovers) holds focus, Escape closes the
    /// open surface and nav keys drive the systray menu's cascade — the scrim covers
    /// the same while the pointer is over the empty desktop; together they span every
    /// pointer position. Click: any click on the bar — left, middle, OR right —
    /// dismisses the open dropdown/menu, EXCEPT a click on an opener widget (left to
    /// that opener's own toggle/swap). A right-click additionally defers to a
    /// secondary-opener (the tray button, whose right-click opens its menu). This is
    /// THE automatic dismiss — modules never call `dismiss_current` themselves.
    pub(super) fn install_dismiss_controllers(
        window: &gtk::Window,
        dropdowns: &Rc<DropdownRegistry>,
    ) {
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed({
            let coordinator = dropdowns.coordinator();
            move |_, keyval, _, state| coordinator.handle_key_event(keyval, state)
        });
        window.add_controller(keys);

        let click_dismiss = gtk::GestureClick::builder().button(0).build();
        click_dismiss.set_propagation_phase(gtk::PropagationPhase::Capture);
        click_dismiss.connect_pressed({
            let coordinator = dropdowns.coordinator();
            let window = window.downgrade();
            move |gesture, _, x, y| {
                let secondary = gesture.current_button() == gdk::BUTTON_SECONDARY;
                let Some(window) = window.upgrade() else {
                    return;
                };
                let target = window.pick(x, y, gtk::PickFlags::DEFAULT);
                coordinator.handle_bar_click(target.as_ref(), secondary);
            }
        });
        window.add_controller(click_dismiss);
    }

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

    pub(super) fn apply_layout(&mut self, new_layout: BarLayout, root: &gtk::Window) {
        if self.layout == new_layout {
            return;
        }

        if self.layout.show != new_layout.show {
            root.set_visible(new_layout.show);
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

    /// Rebuild the `identifier -> (opener, dropdown)` map from the live modules'
    /// dropdown openers (the source of truth — no config walk), and publish the
    /// identifiers for `wayle dropdown list`. Walks the sections in layout order;
    /// a module type that appears more than once gets a positional `#n` suffix.
    pub(super) fn rebuild_dropdown_targets(&self) {
        // Ordered (module token, opener, live names) for every live module that opens a
        // dropdown. Names are read once here (they may change with the config).
        let mut entries: Vec<(String, DropdownOpener, Vec<String>)> = Vec::new();
        for factory in [&self.left, &self.center, &self.right] {
            for index in 0..factory.len() {
                let Some(item) = factory.get(index) else {
                    continue;
                };
                for (module, opener) in item.dropdown_targets() {
                    if let Some(opener) = opener {
                        let names = opener.names();
                        if !names.is_empty() {
                            entries.push((module.to_string(), opener, names));
                        }
                    }
                }
            }
        }

        // `#n` suffix only for a module type that appears more than once.
        let mut totals: HashMap<String, usize> = HashMap::new();
        for (token, _, _) in &entries {
            *totals.entry(token.clone()).or_default() += 1;
        }

        let mut ordinals: HashMap<String, usize> = HashMap::new();
        let mut targets = HashMap::new();
        let mut ids = Vec::new();
        for (token, opener, names) in entries {
            let ordinal = {
                let counter = ordinals.entry(token.clone()).or_default();
                *counter += 1;
                *counter
            };
            let suffix = if totals[&token] > 1 {
                format!("#{ordinal}")
            } else {
                String::new()
            };
            for name in names {
                let identifier = format!("{name}@{token}{suffix}");
                ids.push(identifier.clone());
                targets.insert(identifier, (opener.clone(), name.clone()));
            }
        }
        *self.dropdown_targets.borrow_mut() = targets;
        self.publish_dropdown_ids(ids);
    }

    /// Publish this bar's live dropdown identifiers to the shell IPC state, keyed by
    /// its connector, so `wayle dropdown list` reflects the actual openers.
    fn publish_dropdown_ids(&self, ids: Vec<String>) {
        let Some(connector) = self.settings.monitor_name.clone() else {
            return;
        };
        let ipc = self.services.shell_ipc.state();
        let mut map = ipc.dropdown_ids.get();
        map.insert(connector, ids);
        ipc.dropdown_ids.set(map);
    }

    /// Handle a CLI dropdown request on this bar.
    ///
    /// [`DropdownAction::Close`] dismisses whatever surface is open (no-op if none).
    /// [`Toggle`](DropdownAction::Toggle)/[`Open`](DropdownAction::Open) resolve the
    /// identifier to the owning module's [`DropdownOpener`] and dispatch through it —
    /// the exact same anchor/freeze/coordinator ceremony as a mouse click — toggling
    /// or opening-only respectively. Unknown identifiers (not on this bar) are ignored.
    pub(super) fn handle_dropdown_request(
        &self,
        action: DropdownAction,
        identifier: &str,
        root: &gtk::Window,
    ) {
        if action == DropdownAction::Close {
            self.dropdowns.coordinator().dismiss_current();
            return;
        }

        let target = self.dropdown_targets.borrow().get(identifier).cloned();
        let Some((opener, dropdown)) = target else {
            debug!(identifier, "dropdown request: identifier not present on this bar");
            return;
        };
        // If this bar is hidden (`wayle panel hide`), a CLI-opened dropdown would
        // show the scrim with no bar above it — reveal the bar first.
        self.ensure_unhidden(root);
        let click = ClickAction::Dropdown(dropdown);
        match action {
            DropdownAction::Open => opener.dispatch_open(&click),
            DropdownAction::Toggle => opener.dispatch(&click),
            // Handled by the early return above (it needs no identifier/target); listed
            // explicitly so a new `DropdownAction` variant is a compile error, not a
            // silent Toggle.
            DropdownAction::Close => unreachable!("Close is handled before target resolution"),
        }
    }

    /// Remove this bar's connector from the hidden set and show it now, so a
    /// CLI-opened dropdown has its bar above the scrim. No-op when already visible.
    fn ensure_unhidden(&self, root: &gtk::Window) {
        let Some(connector) = self.settings.monitor_name.as_deref() else {
            return;
        };
        let ipc = self.services.shell_ipc.state();
        let mut hidden = ipc.hidden_bars.get();
        if hidden.remove(connector) {
            ipc.hidden_bars.set(hidden);
            root.set_visible(true);
        }
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
