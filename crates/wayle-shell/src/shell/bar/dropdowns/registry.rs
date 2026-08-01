use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use tracing::{debug, warn};
use wayle_config::{ClickAction, DropdownSources, schemas::bar::Location};
use wayle_widgets::prelude::{BarButton, BarButtonInput};

use super::coordinator::{DismissFn, OpenSurfaceCoordinator};
use super::scrim::Scrim;
use crate::{process, shell::services::ShellServices};

/// Per-bar wiring attached to each dropdown after creation, so its open/close
/// hooks can reach the shared dismissal coordinator (which owns the scrim).
#[derive(Clone)]
struct DropdownWiring {
    coordinator: Rc<OpenSurfaceCoordinator>,
}

/// Where a module's dropdown hangs off: the anchor widget the coordinator keys
/// toggles on, plus how to freeze/thaw a label-bearing button so a live-updating
/// label can't shift the open popover. `freeze` is `Some` only for BarButton-
/// backed modules; container/workspace anchors leave it `None`.
#[derive(Clone)]
pub(crate) struct DropdownAnchor {
    widget: gtk::Widget,
    freeze: Option<relm4::Sender<BarButtonInput>>,
}

impl DropdownAnchor {
    fn button(button: &Controller<BarButton>) -> Self {
        Self {
            widget: button.widget().clone().upcast(),
            freeze: Some(button.sender().clone()),
        }
    }

    fn widget(widget: &impl IsA<gtk::Widget>) -> Self {
        Self {
            widget: widget.clone().upcast(),
            freeze: None,
        }
    }
}

/// A module's canonical dropdown opener: its registry, its fixed anchor, and the
/// dropdown names it can open (declared by the module from its own click config —
/// the single source of truth for `wayle dropdown list`, so there is no central
/// module→dropdown table). Both a mouse click (the module's `update`) and a CLI
/// request (`Bar::handle_dropdown_request`) invoke the SAME opener, so they run the
/// identical open/close/swap ceremony against the identical anchor. Constructing one via
/// [`for_button`](Self::for_button) / [`for_widget`](Self::for_widget) registers it
/// with the bar automatically, so a module cannot forget to. Cheap to clone.
#[derive(Clone)]
pub(crate) struct DropdownOpener {
    registry: Rc<DropdownRegistry>,
    anchor: DropdownAnchor,
    /// Live source of the dropdowns this module opens. Read on demand (not snapshotted)
    /// so `wayle dropdown list` and CLI addressing track runtime config changes to the
    /// click bindings. Usually the module's own config (a `#[wayle_config]` struct
    /// implements [`DropdownSources`] from its `ClickAction` fields).
    sources: Rc<dyn DropdownSources>,
}

impl DropdownOpener {
    /// For a `BarButton`-backed module — anchors to the button, freezes its label.
    /// `sources` is the module's config; its `ClickAction` fields determine which
    /// dropdowns the opener exposes, read live (see [`DropdownSources`]).
    /// (The opener *anchor* is the button; the bar-click "opener" marker that stops
    /// pre-dismiss is applied to the module's outer widget by the factory, since
    /// `BarButton` rewrites its own CSS classes and would wipe a mark on it.)
    pub(crate) fn for_button(
        registry: &Rc<DropdownRegistry>,
        button: &Controller<BarButton>,
        sources: impl DropdownSources + 'static,
    ) -> Self {
        Self::register(registry, DropdownAnchor::button(button), Rc::new(sources))
    }

    /// For a container module — anchors to an arbitrary widget, no freeze.
    /// The bar-click gesture skips the entire opener widget, so a container module
    /// with non-dropdown click actions (e.g. workspace focus) must call
    /// [`dismiss`](Self::dismiss) on those actions to close an open surface.
    pub(crate) fn for_widget(
        registry: &Rc<DropdownRegistry>,
        widget: &impl IsA<gtk::Widget>,
        sources: impl DropdownSources + 'static,
    ) -> Self {
        Self::register(registry, DropdownAnchor::widget(widget), Rc::new(sources))
    }

    /// For a container module that opens no CLI-addressable dropdown (the workspace
    /// switchers). Anchors like [`for_widget`] so mouse-driven dropdowns still work,
    /// but exposes no names, so it never appears in `wayle dropdown list`.
    pub(crate) fn for_widget_unlisted(
        registry: &Rc<DropdownRegistry>,
        widget: &impl IsA<gtk::Widget>,
    ) -> Self {
        Self::register(registry, DropdownAnchor::widget(widget), Rc::new(NoDropdownSources))
    }

    /// Build the opener and publish it to the registry so the CLI list/toggle path
    /// can reach it (drained by `create_module`). Registration is part of
    /// construction so it can't be omitted.
    fn register(
        registry: &Rc<DropdownRegistry>,
        anchor: DropdownAnchor,
        sources: Rc<dyn DropdownSources>,
    ) -> Self {
        let opener = Self {
            registry: registry.clone(),
            anchor,
            sources,
        };
        registry.publish_opener(opener.clone());
        opener
    }

    /// The distinct dropdown names this module currently opens — the live source of
    /// truth for `wayle dropdown list`, re-derived from the config on each call.
    pub(crate) fn names(&self) -> Vec<String> {
        self.sources.dropdown_names()
    }

    /// The single dispatch both the mouse handler and the CLI call. Toggles the
    /// named dropdown against this opener's anchor (or dismisses on Shell/None).
    pub(crate) fn dispatch(&self, action: &ClickAction) {
        dispatch_action(action, &self.registry, |dropdown, style| {
            dropdown.toggle_on(&self.anchor, style);
        });
    }

    /// Like [`dispatch`](Self::dispatch), but *open-only*: opens the named dropdown
    /// against this opener's anchor if it isn't already open, and no-ops if it is.
    /// Backs `wayle dropdown open`.
    pub(crate) fn dispatch_open(&self, action: &ClickAction) {
        dispatch_action(action, &self.registry, |dropdown, style| {
            dropdown.open_on_only(&self.anchor, style);
        });
    }

    /// Dismiss whatever surface is open. For a non-dropdown action on an opener
    /// widget (e.g. a workspace focus/scroll click): the bar-click gesture skips
    /// opener widgets, so such actions dismiss explicitly here — the same effect
    /// `dispatch` gives a `Shell`/`None` action on an ordinary button.
    pub(crate) fn dismiss(&self) {
        self.registry.coordinator().dismiss_current();
    }

    /// Ask the bar to re-derive and republish its dropdown identifiers (via the
    /// registry's republish hook). For a module whose [`DropdownSources`] names change
    /// at runtime outside the config-reload path — the `custom` module updating its
    /// per-instance click bindings — so `wayle dropdown list` refreshes deterministically.
    pub(crate) fn request_republish(&self) {
        self.registry.request_republish();
    }
}

/// A [`DropdownSources`] that opens nothing — backs
/// [`DropdownOpener::for_widget_unlisted`] for the workspace switchers.
struct NoDropdownSources;

impl DropdownSources for NoDropdownSources {
    fn dropdown_names(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Returns `value` unchanged, logging at debug if it is `None`.
///
/// Use inside dropdown factories that gate on a service dependency: instead of
/// returning `None` silently, the helper records which dropdown failed and the
/// service it was waiting on, so the dispatch-site catch-all has the cause
/// already in the log before it runs.
pub(crate) fn require_service<T>(
    dropdown: &'static str,
    service: &'static str,
    value: Option<T>,
) -> Option<T> {
    if value.is_none() {
        debug!(
            dropdown,
            service, "service unavailable, dropdown disabled on this system"
        );
    }
    value
}

/// Shared dropdown instance for a dropdown name.
///
/// Reuse keeps dropdown state consistent across modules that reference the same
/// dropdown and avoids rebuilding the same component repeatedly.
pub(crate) struct DropdownInstance {
    popover: gtk::Popover,
    _controller: Box<dyn Any>,
    thaw_target: Rc<Cell<Option<relm4::Sender<BarButtonInput>>>>,
    /// Filled by `DropdownRegistry::get_or_create` after construction (the
    /// factories that call `new` don't have the coordinator/scrim).
    wiring: Rc<RefCell<Option<DropdownWiring>>>,
    /// This dropdown's dismiss closure while it's the open surface, so
    /// `connect_closed` can tell the coordinator it closed.
    current_dismiss: Rc<RefCell<Option<DismissFn>>>,
}

impl DropdownInstance {
    pub(crate) fn new(popover: gtk::Popover, controller: Box<dyn Any>) -> Self {
        let thaw_target: Rc<Cell<Option<relm4::Sender<BarButtonInput>>>> = Rc::default();
        let wiring: Rc<RefCell<Option<DropdownWiring>>> = Rc::default();
        let current_dismiss: Rc<RefCell<Option<DismissFn>>> = Rc::default();

        popover.connect_map(|popover| {
            debug!(
                width = popover.width(),
                height = popover.height(),
                classes = ?popover.css_classes(),
                "popover mapped"
            );
        });


        let thaw = thaw_target.clone();
        let wiring_closed = wiring.clone();
        let dismiss_closed = current_dismiss.clone();
        popover.connect_closed(move |popover| {
            debug!(
                width = popover.width(),
                height = popover.height(),
                classes = ?popover.css_classes(),
                "popover closed"
            );
            let frozen_sender = thaw.take();

            if let Some(sender) = &frozen_sender {
                sender.emit(BarButtonInput::ThawSize);
            }

            if frozen_sender.is_some()
                && let Some(parent) = popover.parent()
            {
                parent.set_size_request(-1, -1);
            }

            // Clone the wiring out before calling into it so no borrow is held
            // across GTK/coordinator calls that could re-enter. `notify_closed`
            // also hides the scrim; Escape/keyboard are handled at the bar/scrim
            // level, not per-popover.
            let wiring = wiring_closed.borrow().clone();
            if let (Some(wiring), Some(dismiss)) = (wiring, dismiss_closed.borrow_mut().take()) {
                wiring.coordinator.notify_closed(&dismiss);
            }
        });

        Self {
            popover,
            _controller: controller,
            thaw_target,
            wiring,
            current_dismiss,
        }
    }

    fn attach(&self, wiring: DropdownWiring) {
        *self.wiring.borrow_mut() = Some(wiring);
    }

    fn coordinator(&self) -> Option<Rc<OpenSurfaceCoordinator>> {
        self.wiring.borrow().as_ref().map(|w| w.coordinator.clone())
    }

    /// Register this dropdown as the open surface (closing any other) and show the
    /// scrim. Called right before `popup()`.
    ///
    /// If this dropdown is ALREADY the open surface (a reparent to a different
    /// opener — e.g. a shared dropdown moving from one bar button to another), this
    /// just re-anchors it in place via [`OpenSurfaceCoordinator::reanchor`], keeping
    /// the same dismiss token and leaving the scrim shown — no close+reopen dip.
    fn register_open(&self) {
        let Some(wiring) = self.wiring.borrow().clone() else {
            return;
        };
        let anchor = self.popover.parent().map(|widget| widget.downgrade());

        // Re-anchor if we're still the registered surface (our dismiss token
        // survived the reparent — `unparent` doesn't emit `closed`).
        if let Some(existing) = self.current_dismiss.borrow().clone()
            && wiring.coordinator.reanchor(&existing, anchor.clone())
        {
            return;
        }

        let popover = self.popover.downgrade();
        let dismiss: DismissFn = Rc::new(move || {
            if let Some(popover) = popover.upgrade() {
                popover.popdown();
            }
        });
        *self.current_dismiss.borrow_mut() = Some(dismiss.clone());
        // Dropdowns have no custom key handler — their own focused widgets handle
        // keys; Escape is handled by the bar/scrim via `dismiss_current`. The
        // anchor is the button the popover hangs off, so clicking it toggles/swaps
        // rather than plain-dismisses. `open` shows the scrim.
        wiring.coordinator.open(dismiss, None, anchor);
    }

    /// Whether THIS dropdown is the coordinator's currently-open surface AND anchored
    /// at `anchor`. `current_dismiss` is `Some` only while this instance is the open
    /// surface (its `connect_closed` clears it when swapped out), so combining it with
    /// the anchor check distinguishes "this dropdown open here" from a *different*
    /// dropdown open on the same button (a module's left- vs right-click dropdowns
    /// share an anchor) and from this dropdown open on a *different* button.
    fn is_open_at(&self, coordinator: &OpenSurfaceCoordinator, anchor: &DropdownAnchor) -> bool {
        self.current_dismiss.borrow().is_some() && coordinator.on_same_anchor(&anchor.widget)
    }

    /// Toggle this dropdown for `anchor` (a module's canonical opener anchor). Closes
    /// it only when THIS dropdown is already open here; otherwise opens it, which
    /// swaps out whatever else is showing (so clicking a module's other-button
    /// dropdown switches to it in one click) or re-anchors this one to a new button.
    /// Mouse clicks and CLI toggles both route here via one [`DropdownOpener`].
    fn toggle_on(&self, anchor: &DropdownAnchor, style: DropdownStyle) {
        let Some(coordinator) = self.coordinator() else {
            return;
        };
        if self.is_open_at(&coordinator, anchor) {
            coordinator.dismiss_current();
        } else {
            self.open_on(anchor, style);
        }
    }

    /// Open-only variant of [`toggle_on`](Self::toggle_on): opens (or reparents)
    /// this dropdown at `anchor` if it isn't already the open surface there, and
    /// no-ops if it is. Backs `wayle dropdown open`.
    fn open_on_only(&self, anchor: &DropdownAnchor, style: DropdownStyle) {
        let Some(coordinator) = self.coordinator() else {
            return;
        };
        if !self.is_open_at(&coordinator, anchor) {
            self.open_on(anchor, style);
        }
    }

    /// Open (or reparent-and-open) this dropdown anchored to `anchor`, regardless
    /// of current visibility — the coordinator's `toggle` already decided we're
    /// opening. Thaws the old parent first when moving from a different one, and
    /// freezes the button label only when the anchor carries a freeze sender (a
    /// `BarButton`) and the style enables it. Margins/position are applied here so
    /// individual dropdowns never handle placement.
    fn open_on(&self, anchor: &DropdownAnchor, style: DropdownStyle) {
        if self.popover.is_visible()
            && self.popover.parent().as_ref() != Some(&anchor.widget)
            && let Some(sender) = self.thaw_target.take()
        {
            sender.emit(BarButtonInput::ThawSize);
        }
        self.ensure_parent(&anchor.widget);

        if style.freeze_label
            && let Some(sender) = &anchor.freeze
        {
            self.thaw_target.set(Some(sender.clone()));
            sender.emit(BarButtonInput::FreezeSize);
            self.lock_parent_size();
        }

        self.apply_position();
        self.apply_margins(style.margins);
        self.apply_style(&style);
        self.register_open();
        self.popover.popup();
    }

    fn ensure_parent(&self, target: &gtk::Widget) {
        if self.popover.parent().as_ref() == Some(target) {
            return;
        }
        if self.popover.parent().is_some() {
            // Reparenting to a different anchor. `unparent` doesn't emit `closed`,
            // so `current_dismiss` (and the coordinator's registration) survive the
            // move — `register_open` then re-anchors in place, keeping the scrim
            // shown instead of a close→reopen dip.
            self.popover.unparent();
        }
        self.popover.set_parent(target);

        let popover = self.popover.downgrade();
        target.connect_destroy(move |destroyed| {
            let Some(popover) = popover.upgrade() else {
                return;
            };
            if popover.parent().as_ref() == Some(destroyed) {
                popover.unparent();
            }
        });
    }

    fn apply_style(&self, style: &DropdownStyle) {
        self.popover.set_opacity(style.opacity);
        // Never autohide: the Wayland popup grab freezes input and swallows the
        // dismiss-click on Hyprland. Dismissal is handled by the coordinator +
        // scrim instead (see `register_open`).
        self.popover.set_autohide(false);
        if style.shadow_enabled {
            self.popover.add_css_class("shadow");
        } else {
            self.popover.remove_css_class("shadow");
        }
    }

    fn apply_position(&self) {
        let Some(parent) = self.popover.parent() else {
            return;
        };
        let position = Self::detect_popover_position(&parent);
        self.popover.set_position(position);

        for class in &[
            "position-top",
            "position-bottom",
            "position-left",
            "position-right",
        ] {
            self.popover.remove_css_class(class);
        }
        let class = match position {
            gtk::PositionType::Top => "position-top",
            gtk::PositionType::Bottom => "position-bottom",
            gtk::PositionType::Left => "position-left",
            gtk::PositionType::Right => "position-right",
            _ => "position-bottom",
        };
        self.popover.add_css_class(class);
    }

    fn apply_margins(&self, margins: DropdownMargins) {
        let Some(child) = self.popover.child() else {
            return;
        };
        child.set_margin_top(margins.top);
        child.set_margin_bottom(margins.bottom);
        child.set_margin_start(margins.start);
        child.set_margin_end(margins.end);
    }

    fn lock_parent_size(&self) {
        let Some(parent) = self.popover.parent() else {
            return;
        };
        parent.set_size_request(parent.width(), parent.height());
    }

    fn detect_popover_position(widget: &gtk::Widget) -> gtk::PositionType {
        let Some(window) = widget.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else {
            return gtk::PositionType::Bottom;
        };

        if window.has_css_class("bottom") {
            gtk::PositionType::Top
        } else if window.has_css_class("left") {
            gtk::PositionType::Right
        } else if window.has_css_class("right") {
            gtk::PositionType::Left
        } else {
            gtk::PositionType::Bottom
        }
    }
}

impl Drop for DropdownInstance {
    fn drop(&mut self) {
        if self.popover.parent().is_some() {
            self.popover.unparent();
        }
    }
}

struct DropdownStyle {
    margins: DropdownMargins,
    opacity: f64,
    shadow_enabled: bool,
    freeze_label: bool,
}

const REM_PX: f32 = 16.0;

/// Pixel margins applied to dropdown containers.
///
/// Values are rounded to whole pixels so popover content stays visually crisp.
/// The bar-facing edge gets a smaller gap; the opposite edge and sides get
/// standard content padding.
#[derive(Debug, Clone, Copy)]
struct DropdownMargins {
    top: i32,
    bottom: i32,
    start: i32,
    end: i32,
}

impl DropdownMargins {
    const GAP_REM: f32 = 0.275;
    const CONTENT_REM: f32 = 1.0;

    fn new(scale: f32, location: Location) -> Self {
        let gap = Self::round(Self::GAP_REM, scale);
        let content = Self::round(Self::CONTENT_REM, scale);

        match location {
            Location::Top => Self {
                top: gap,
                bottom: content,
                start: content,
                end: content,
            },
            Location::Bottom => Self {
                top: content,
                bottom: gap,
                start: content,
                end: content,
            },
            Location::Left => Self {
                top: content,
                bottom: content,
                start: gap,
                end: content,
            },
            Location::Right => Self {
                top: content,
                bottom: content,
                start: content,
                end: gap,
            },
        }
    }

    fn round(rem: f32, scale: f32) -> i32 {
        (rem * REM_PX * scale).round() as i32
    }
}

/// Factory trait for creating dropdown component instances.
pub(crate) trait DropdownFactory {
    /// Creates a dropdown component, returning `None` if required services are unavailable.
    fn create(services: &ShellServices) -> Option<DropdownInstance>;
}

/// The per-bar dropdown hub — the one object threaded to both every module's `init()`
/// and to `create_module`. Beyond its namesake cache it wears three hats:
///
/// - **cache**: dropdown instances keyed by name, created lazily on first use and
///   reused so repeated interactions resolve to the same logical instance;
/// - **coordinator owner**: it holds the [`OpenSurfaceCoordinator`] (and thus the
///   scrim), which it attaches to each instance on `get_or_create` — genuinely coupled
///   to the cache, so they live together;
/// - **wiring courier**: the `pending_opener` slot couriers a module's opener to the
///   bar, and the `republish` hook lets a module ask the bar to re-run
///   `rebuild_dropdown_targets` — both exist only because this is the already-shared
///   object, not because they belong to a cache.
pub(crate) struct DropdownRegistry {
    services: ShellServices,
    cache: RefCell<HashMap<String, Rc<DropdownInstance>>>,
    coordinator: Rc<OpenSurfaceCoordinator>,
    /// Courier slot: a module publishes its [`DropdownOpener`] here during its
    /// `init()`, and `create_module` drains it immediately after the module is
    /// launched. relm4 runs `init` synchronously and modules are built one at a
    /// time on the GTK main thread, so at most one opener is ever un-drained.
    pending_opener: RefCell<Option<DropdownOpener>>,
    /// Hook the bar installs so a module can ask it to re-run `rebuild_dropdown_targets`
    /// after mutating its dropdown names at runtime (see `request_republish`).
    republish: RefCell<Option<Rc<dyn Fn()>>>,
}

impl DropdownRegistry {
    pub(crate) fn new(
        services: &ShellServices,
        monitor: &gtk::gdk::Monitor,
        bar_window: &gtk::Window,
    ) -> Self {
        let coordinator = Rc::new(OpenSurfaceCoordinator::default());
        // The scrim needs the coordinator (for its dismiss handlers), and the
        // coordinator owns the scrim thereafter (show/hide on open/close).
        coordinator.set_scrim(Scrim::new(services, monitor, bar_window, &coordinator));
        Self {
            services: services.clone(),
            cache: RefCell::default(),
            coordinator,
            pending_opener: RefCell::default(),
            republish: RefCell::default(),
        }
    }

    pub(crate) fn coordinator(&self) -> Rc<OpenSurfaceCoordinator> {
        self.coordinator.clone()
    }

    /// Install the bar's republish hook (called once at bar init). Invoking it re-runs
    /// the bar's `rebuild_dropdown_targets`, re-deriving and republishing the dropdown
    /// identifiers from the live openers.
    pub(crate) fn set_republish(&self, hook: Rc<dyn Fn()>) {
        *self.republish.borrow_mut() = Some(hook);
    }

    /// Ask the bar to re-run `rebuild_dropdown_targets`. Used by a module whose dropdown
    /// names changed at runtime in a way the config-reload republish can't observe in
    /// order (the `custom` module updating its per-instance click bindings), so the
    /// rebuild runs *after* the new names are in place rather than racing them.
    pub(crate) fn request_republish(&self) {
        if let Some(hook) = self.republish.borrow().as_ref() {
            hook();
        }
    }

    /// A module hands its canonical opener to the bar (see `pending_opener`).
    pub(crate) fn publish_opener(&self, opener: DropdownOpener) {
        debug_assert!(
            self.pending_opener.borrow().is_none(),
            "pending opener slot must be empty at publish (synchronous-init invariant)"
        );
        *self.pending_opener.borrow_mut() = Some(opener);
    }

    /// Drain the opener a module published during its `init()`, if any.
    pub(crate) fn take_opener(&self) -> Option<DropdownOpener> {
        self.pending_opener.borrow_mut().take()
    }

    pub(crate) fn warm_all(&self) {
        for name in super::DROPDOWN_NAMES {
            let _ = self.get_or_create(name);
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn get_or_create(&self, name: &str) -> Option<Rc<DropdownInstance>> {
        let mut cache = self.cache.borrow_mut();
        if let Some(instance) = cache.get(name) {
            debug!(dropdown = name, "cache hit");
            return Some(instance.clone());
        }

        debug!(dropdown = name, "creating dropdown");
        let Some(raw) = super::create(name, &self.services) else {
            debug!(
                dropdown = name,
                "no instance created (factory declined, usually due to a missing service or dependency \
                 -- see preceding debug log from the factory for the specific cause)"
            );
            return None;
        };
        let instance = Rc::new(raw);
        instance.attach(DropdownWiring {
            coordinator: self.coordinator.clone(),
        });
        cache.insert(name.to_owned(), instance.clone());
        debug!(dropdown = name, "dropdown cached");
        Some(instance)
    }
}

#[allow(clippy::cognitive_complexity)]
fn dispatch_action(
    action: &ClickAction,
    registry: &DropdownRegistry,
    toggle: impl FnOnce(&DropdownInstance, DropdownStyle),
) {
    match action {
        ClickAction::Dropdown(name) => {
            debug!(dropdown = %name, "click: dropdown");
            if let Some(dropdown) = registry.get_or_create(name) {
                let style = dropdown_style(registry);
                toggle(&dropdown, style);
            } else {
                warn!(
                    dropdown = %name,
                    "click dropped: no dropdown available (dropdown is either unregistered or its \
                     backing service is unavailable on this system)"
                );
            }
        }
        // A dropdown-capable module's whole outer widget is marked `dropdown-opener`
        // by the factory, so the bar-click gesture always skips it (for every mouse
        // button). A Shell/None click therefore has to dismiss the open surface
        // itself here — the same automatic dismiss the gesture gives ordinary widgets.
        ClickAction::Shell(cmd) => {
            debug!(command = %cmd, "click: shell");
            registry.coordinator.dismiss_current();
            process::run_if_set(cmd);
        }
        ClickAction::None => {
            debug!("click: none");
            registry.coordinator.dismiss_current();
        }
    }
}

fn dropdown_style(registry: &DropdownRegistry) -> DropdownStyle {
    let config = registry.services.config.config();
    let bar = &config.bar;
    let scale = bar.scale.get().value();
    DropdownStyle {
        margins: DropdownMargins::new(scale, bar.location.get()),
        opacity: f64::from(bar.dropdown_opacity.get().value()) / 100.0,
        shadow_enabled: bar.dropdown_shadow.get(),
        freeze_label: bar.dropdown_freeze_label.get(),
    }
}
