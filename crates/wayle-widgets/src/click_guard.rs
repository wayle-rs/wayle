//! Protection against clicking a control that just moved under the pointer.
//!
//! When a list reorders (or gains a row) while it is on screen, the row under
//! the pointer can change between the user aiming and the click landing. A
//! control that has just moved is guarded by arming its [`ClickGuard`]: for
//! [`CLICK_GUARD_INTERVAL`] it neither receives clicks nor looks clickable.

use std::{cell::RefCell, rc::Rc, time::Duration};

use gtk4::{glib, prelude::*};

/// How long a control ignores clicks after moving under the pointer.
///
/// Longer than visual reaction time (~200–250 ms), so a click already in
/// motion when the control moved is not delivered to whatever took its place;
/// in line with the 300–500 ms windows browsers use for similar protection.
pub const CLICK_GUARD_INTERVAL: Duration = Duration::from_millis(400);

/// Makes a widget unclickable for [`CLICK_GUARD_INTERVAL`] after being
/// [armed](Self::arm).
///
/// While armed, the widget (and everything inside it) is excluded from pointer
/// picking via `can-target`, so it gets no clicks, no `:hover` state and no
/// pointer cursor. This changes no styles, so arming causes no style work or
/// redraws. GTK only re-picks the pointer target on the next pointer event,
/// so a widget already hovered when armed keeps `:hover` until the pointer
/// moves.
///
/// Arming only needs a shared reference, so a factory row reached via `get`
/// can be guarded without marking it changed and re-rendering it. Clones share
/// the same guard.
#[derive(Debug, Clone)]
pub struct ClickGuard(Rc<Inner>);

#[derive(Debug)]
struct Inner {
    widget: gtk4::Widget,
    /// The pending expiry, while armed.
    timer: RefCell<Option<glib::SourceId>>,
}

impl ClickGuard {
    /// A disarmed guard for `widget`.
    pub fn new(widget: &impl IsA<gtk4::Widget>) -> Self {
        Self(Rc::new(Inner {
            widget: widget.clone().upcast(),
            timer: RefCell::new(None),
        }))
    }

    /// Starts (or restarts) the guard interval.
    pub fn arm(&self) {
        if let Some(timer) = self.0.timer.take() {
            timer.remove();
        }

        self.0.widget.set_can_target(false);

        let inner = Rc::downgrade(&self.0);
        let timer = glib::timeout_add_local_once(CLICK_GUARD_INTERVAL, move || {
            if let Some(inner) = inner.upgrade() {
                // The source is finishing; forget it rather than removing it.
                inner.timer.take();
                inner.release();
            }
        });
        self.0.timer.replace(Some(timer));
    }

    /// Ends the guard interval early.
    pub fn disarm(&self) {
        if let Some(timer) = self.0.timer.take() {
            timer.remove();
            self.0.release();
        }
    }

    /// Whether the widget is currently guarded.
    pub fn is_armed(&self) -> bool {
        self.0.timer.borrow().is_some()
    }
}

impl Inner {
    fn release(&self) {
        self.widget.set_can_target(true);
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(timer) = self.timer.take() {
            timer.remove();
        }
    }
}
