//! Tracks the one open dismissable surface per bar (a dropdown popover or a
//! systray menu) so opening another closes it — the non-grab replacement for
//! GTK autohide's "only one popup at a time" behavior.
//!
//! It also owns the shared [`Scrim`] and drives the whole open/close ceremony, so
//! the two surface types (dropdowns, systray menu) don't each hand-roll it:
//! [`open`](OpenSurfaceCoordinator::open) installs the new surface then closes the
//! previous one (show-before-close keeps the scrim shown across a swap), and
//! [`notify_closed`](OpenSurfaceCoordinator::notify_closed) re-derives scrim visibility.
//!
//! It is likewise the single *decision* authority: [`toggle`](OpenSurfaceCoordinator::toggle)
//! decides open-vs-close-vs-swap for any opener (bar clicks, the tray, and the CLI
//! all route through it), and [`handle_bar_click`](OpenSurfaceCoordinator::handle_bar_click)
//! decides which bar clicks dismiss. Because a swap is thus handled entirely by
//! `open`'s show-before-close, there is no press-time dismissal dip — no debounce.
//!
//! A surface is represented by *how to close it* (a `DismissFn`) plus an optional
//! *key handler* (`KeyHandler`) for keyboard nav and the *anchor* bar widget it
//! hangs off. Both the bar window and the scrim hold keyboard focus in different
//! situations (pointer over the bar/popover vs the empty desktop), and the menu's
//! popovers never get GTK focus themselves (their buttons are non-focusable), so
//! nav keys are forwarded here from whichever surface has focus rather than handled
//! per-popover. Identity is by `Rc` pointer on the `DismissFn`, which lets a
//! surface's `connect_closed` ask "am I still the registered one?" without
//! downcasting.

use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};

use relm4::gtk::{self, prelude::*};

use super::scrim::Scrim;

/// A currently-open dismissable surface, as a closure that closes it.
pub(crate) type DismissFn = Rc<dyn Fn()>;

/// Consumes a forwarded key (with its modifier state) for the open surface; returns
/// `true` when handled.
pub(crate) type KeyHandler = Rc<dyn Fn(gtk::gdk::Key, gtk::gdk::ModifierType) -> bool>;

/// CSS class marking a widget as a dropdown *opener* — clicking it toggles a
/// surface itself, so `handle_bar_click` must not pre-dismiss on press. Applied
/// eagerly at construction by the bar factory (`BarItemFactory::init_widgets`) to a
/// dropdown-capable module's outer widget; `handle_bar_click` only tests the mark.
pub(crate) const OPENER_CSS_CLASS: &str = "dropdown-opener";

/// CSS class marking a widget that opens a surface on the *secondary* (right)
/// click only — the systray tray button, whose right-click toggles its menu but
/// whose left/middle-click activate the app and should still dismiss any open
/// surface. `handle_bar_click` treats it as an opener for secondary clicks only,
/// so a right-click there defers to the item's own dip-free toggle instead of
/// being pre-dismissed.
pub(crate) const SECONDARY_OPENER_CSS_CLASS: &str = "dropdown-opener-secondary";

struct OpenSurface {
    dismiss: DismissFn,
    keys: Option<KeyHandler>,
    /// The bar widget this surface hangs off (a dropdown/tray button/container).
    /// `toggle` compares against it to decide "clicked my own opener → close".
    anchor: Option<gtk::glib::WeakRef<gtk::Widget>>,
}

#[derive(Default)]
pub(crate) struct OpenSurfaceCoordinator {
    current: RefCell<Option<OpenSurface>>,
    /// Set while `open` is closing the previous surface, so that surface's
    /// `connect_closed` → `notify_closed` doesn't clear the newly-registered one.
    dismissing: Cell<bool>,
    /// The shared dismissal scrim. Set once at init (see `set_scrim`); shown on
    /// `open`, hidden on `notify_closed`, so surfaces don't touch it directly.
    scrim: OnceCell<Rc<Scrim>>,
}

impl OpenSurfaceCoordinator {
    /// Attach the scrim. Called once at construction (the scrim needs the
    /// coordinator, so it can't be passed to `new`).
    pub(crate) fn set_scrim(&self, scrim: Rc<Scrim>) {
        let _ = self.scrim.set(scrim);
    }

    // The scrim is attached exactly once, in `DropdownRegistry::new`, immediately
    // after the coordinator is constructed and before any surface can open — so it is
    // always present by the time anything calls this.
    #[allow(clippy::expect_used)]
    fn scrim(&self) -> &Rc<Scrim> {
        self.scrim.get().expect("scrim attached at init")
    }

    /// The scrim is shown iff a surface is open. Called after every mutation of
    /// `current`, so scrim visibility is a pure function of that single source of
    /// truth. Both `show`/`hide` are idempotent, so a missed `connect_closed` can't
    /// strand the scrim "shown" the way the old ref-count could.
    fn sync_scrim(&self) {
        if self.current.borrow().is_some() {
            self.scrim().show();
        } else {
            self.scrim().hide();
        }
    }

    /// Register `dismiss` (an optional key handler, and the anchor bar widget) as
    /// the open surface, closing whatever was open first. The caller pops its own
    /// surface up after.
    pub(crate) fn open(
        &self,
        dismiss: DismissFn,
        keys: Option<KeyHandler>,
        anchor: Option<gtk::glib::WeakRef<gtk::Widget>>,
    ) {
        // Install the new one BEFORE closing the old, so the old surface's
        // `connect_closed` (→ `notify_closed`) sees a different `current` and is a
        // no-op for it. `current` is now Some, so `sync_scrim` shows the scrim
        // before the close — it never dips to hidden across a swap.
        let previous = self.current.borrow_mut().replace(OpenSurface {
            dismiss,
            keys,
            anchor,
        });
        self.sync_scrim();
        if let Some(previous) = previous {
            self.dismissing.set(true);
            (previous.dismiss)();
            self.dismissing.set(false);
        }
    }

    /// If `dismiss` identifies the currently-open surface, move it to a new
    /// `anchor` in place and return `true` — a *re-anchor*: the same surface reused
    /// from a different opener (e.g. a shared dropdown reparented from one bar
    /// button to another that opens the same dropdown). `current` never goes
    /// `None`, so the scrim stays shown with no hide→show dip. Returns `false` when
    /// it isn't the open surface, so the caller falls back to [`open`](Self::open).
    pub(crate) fn reanchor(
        &self,
        dismiss: &DismissFn,
        anchor: Option<gtk::glib::WeakRef<gtk::Widget>>,
    ) -> bool {
        let mut current = self.current.borrow_mut();
        let Some(surface) = current.as_mut() else {
            return false;
        };
        if !Rc::ptr_eq(&surface.dismiss, dismiss) {
            return false;
        }
        surface.anchor = anchor;
        true
    }

    /// A surface reports it closed (from its `connect_closed`). Clears the slot
    /// unless mid-swap or no longer the registered one, then re-derives the scrim.
    pub(crate) fn notify_closed(&self, who: &DismissFn) {
        if !self.dismissing.get() {
            let mut current = self.current.borrow_mut();
            if current
                .as_ref()
                .is_some_and(|open| Rc::ptr_eq(&open.dismiss, who))
            {
                *current = None;
            }
        }
        self.sync_scrim();
    }

    /// Close whatever is open (the scrim/bar Escape handlers, scrim outside-click).
    /// Returns `true` if something was open and dismissed, so callers can decide
    /// whether to consume the key.
    pub(crate) fn dismiss_current(&self) -> bool {
        let current = self.current.borrow_mut().take();
        let dismissed = current.is_some();
        if let Some(current) = current {
            (current.dismiss)();
        }
        // `current` is now None, so the scrim hides — even if the dismiss's
        // `connect_closed` stranded its token and never reached `notify_closed`.
        self.sync_scrim();
        dismissed
    }

    /// Whether a dismissable surface is currently open. The bar consults this before
    /// re-layering itself on a config change: while a surface is open the scrim owns the
    /// bar's layer (raised to Overlay), so the bar must not drop below the still-active
    /// scrim — [`Scrim::hide`] re-applies the configured layer when the surface closes.
    pub(crate) fn has_open_surface(&self) -> bool {
        self.current.borrow().is_some()
    }

    /// The single open/close/toggle decision for an opener — a dropdown button, a
    /// tray button, or `wayle dropdown toggle`. If the open surface
    /// is anchored to this same `anchor` widget, close it; otherwise run `open`,
    /// which builds/reparents/shows and calls [`open`](Self::open), whose
    /// show-before-close swaps out any other open surface with no scrim dip.
    ///
    /// `anchor` is the caller's canonical anchor widget (the same object stored in
    /// `OpenSurface.anchor`), so this is a plain widget-identity comparison.
    pub(crate) fn toggle(&self, anchor: &gtk::Widget, open: impl FnOnce()) {
        if self.on_same_anchor(anchor) {
            self.dismiss_current();
        } else {
            open();
        }
    }

    /// Like [`toggle`](Self::toggle), but *open-only*: if this `anchor`'s surface is
    /// already open, do nothing (rather than closing it); otherwise run `open`.
    /// Backs `wayle dropdown open` (open-if-closed, no-op-if-open).
    pub(crate) fn open_only(&self, anchor: &gtk::Widget, open: impl FnOnce()) {
        if !self.on_same_anchor(anchor) {
            open();
        }
    }

    /// Whether the currently-open surface is anchored to `anchor` (a plain
    /// widget-identity comparison against `OpenSurface.anchor`). Computed in a `let`
    /// so the `current` borrow is dropped before any follow-up open/dismiss. Note
    /// this identifies the *anchor*, not which surface — a module with several
    /// dropdowns on one button shares an anchor, so callers that must distinguish
    /// them combine this with the surface's own open-state (see `DropdownInstance`).
    pub(crate) fn on_same_anchor(&self, anchor: &gtk::Widget) -> bool {
        self.current
            .borrow()
            .as_ref()
            .and_then(|surface| surface.anchor.as_ref())
            .and_then(gtk::glib::WeakRef::upgrade)
            .is_some_and(|current_anchor| current_anchor == *anchor)
    }

    /// Dismiss the open surface for a click on the bar at `target` (the picked
    /// widget), unless the click landed on (or inside) an *opener* widget. Openers
    /// carry the `dropdown-opener` CSS class (applied eagerly at construction by the
    /// factory); they toggle/swap themselves dip-free via [`toggle`](Self::toggle),
    /// so pre-dismissing their press would only cause a flash. A `secondary` (right)
    /// click additionally defers to a `dropdown-opener-secondary` widget (the tray
    /// button, whose right-click opens its menu). Everything else — empty bar, a
    /// workspace button, a left/middle-clicked tray icon — dismisses, so a click of
    /// any button anywhere off an opener closes the open surface. This is the
    /// automatic dismiss; modules never call `dismiss_current` for ordinary clicks.
    pub(crate) fn handle_bar_click(&self, target: Option<&gtk::Widget>, secondary: bool) {
        let on_opener = target.is_some_and(|target| {
            let mut widget = Some(target.clone());
            while let Some(current) = widget {
                if current.has_css_class(OPENER_CSS_CLASS)
                    || (secondary && current.has_css_class(SECONDARY_OPENER_CSS_CLASS))
                {
                    return true;
                }
                widget = current.parent();
            }
            false
        });
        if !on_opener {
            self.dismiss_current();
        }
    }

    /// Forward a key to the open surface's key handler (the systray menu's nav);
    /// returns `true` if consumed. Clones the handler out before calling so no
    /// borrow is held across a re-entrant close (activation/Escape pops the menu
    /// down, which clears `current`).
    pub(crate) fn handle_key(&self, key: gtk::gdk::Key, state: gtk::gdk::ModifierType) -> bool {
        let handler = self
            .current
            .borrow()
            .as_ref()
            .and_then(|open| open.keys.clone());
        handler.is_some_and(|handler| handler(key, state))
    }

    /// The single Escape/nav-key policy for both key controllers (bar window and
    /// scrim): Escape dismisses the open surface, other keys (with their modifier
    /// state, for the menu's Ctrl page motions) drive the systray menu's nav. Returns
    /// the propagation the controller should report.
    pub(crate) fn handle_key_event(
        &self,
        key: gtk::gdk::Key,
        state: gtk::gdk::ModifierType,
    ) -> gtk::glib::Propagation {
        let handled = if key == gtk::gdk::Key::Escape {
            self.dismiss_current()
        } else {
            self.handle_key(key, state)
        };
        if handled {
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    }
}
