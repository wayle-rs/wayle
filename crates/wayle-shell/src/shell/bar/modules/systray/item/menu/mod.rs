//! Custom, scrollable, keyboard-navigable system-tray dropdown menu.
//!
//! `GtkPopoverMenu` cannot scroll, so a submenu taller than the screen (a VPN
//! applet's country list) overflows and clips. This renders the menu directly
//! from the `wayle_systray` [`MenuItem`] tree as a cascade of **independent,
//! non-grab [`gtk::Popover`]s** — one per column. Each column is its own surface,
//! so it positions and flips independently (the parent stays put when a child
//! opens; a child that would overflow the right edge flips to the left), scrolls
//! on its own, and is naturally its own floating card. No autohide (its Wayland
//! grab freezes and swallows clicks on Hyprland); outside-click/Escape dismissal
//! comes from the shared scrim (see `dropdowns::scrim`).
//!
//! Keyboard: the column popovers never get GTK focus (their buttons are
//! non-focusable), so per-popover key controllers wouldn't fire — keyboard focus
//! sits on whichever surface the pointer is over (the bar or the scrim). Instead
//! the menu registers [`MenuInner::handle_key`] with the open-surface coordinator;
//! the bar window and the scrim forward nav keys to it. There is ONE selected entry
//! ([`MenuInner::selected`]) — the one Enter activates — reached identically by
//! pointer or keyboard (see [`MenuInner::select`]): the mouse selects whatever entry
//! it moves over, and otherwise the keyboard has priority (Up/Down move within the
//! entry's column, Right descends into its submenu, Left ascends to the entry that
//! owns the column; vim `h`/`j`/`k`/`l` alias the arrows, `gg`/`G` jump to the
//! top/bottom, and Ctrl+`d`/`u`/`f`/`b` page the selection — see
//! [`MenuInner::handle_key`]). The selected entry is shown with its submenu; the
//! selection is a `.selected` CSS class.
//!
//! Submodules: [`column`] (one column: popover + rows + cursor + open child) and
//! [`construct`] (build + wire).

mod column;
mod construct;

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::Arc,
};

use chrono::Utc;
use relm4::gtk::{self, prelude::*};
use tracing::{debug, error, warn};
use wayle_systray::{
    core::item::TrayItem,
    types::menu::{MenuEvent, MenuItem},
};

use column::{MenuColumn, RowActivation};

/// A built tray menu: the root column plus the shared controller state.
pub(super) struct TrayMenu {
    inner: Rc<MenuInner>,
    root: Rc<MenuColumn>,
}

/// Shared controller state, referenced (weakly) from every row handler.
struct MenuInner {
    item: Arc<TrayItem>,
    /// The root popover, for dismissing the whole cascade.
    root_popover: gtk::Popover,
    /// Suppresses the synthetic hover-enter GTK fires when a submenu pops up
    /// under a stationary pointer, so it can't fight a structural change.
    navigating: Cell<bool>,
    /// Set while the user is navigating by keyboard, so a synthetic hover-enter GTK
    /// fires when the menu SCROLLS under a stationary pointer (a new row slides under
    /// the cursor) can't snap the selection off the keyboard cursor. Cleared by the
    /// first real pointer motion (see `construct::wire_row`).
    keyboard_nav: Cell<bool>,
    /// The column holding the single selected entry (highlighted, shown with its
    /// submenu, activated by Enter). The row index lives in that column's `cursor`
    /// (the single source of truth, so a reconcile that clamps the cursor can't leave
    /// a stale index here). Exactly one entry is selected at a time, whether reached
    /// by pointer or keyboard; `None` until the first selection. All selection changes
    /// go through [`MenuInner::select`].
    selected: RefCell<Option<Weak<MenuColumn>>>,
    /// The pointer's last surface position, to tell a REAL pointer move from the
    /// synthetic motion GTK fires when a popdown / reconcile / scroll re-targets a
    /// stationary pointer. The surface position is stable unless the pointer
    /// physically moves (unlike a row's local x/y, which shift as the row
    /// scrolls/relayouts). Only a real move clears `keyboard_nav`.
    pointer_pos: Cell<Option<(f64, f64)>>,
    /// Set after a lone `g`, so the next key completes a vim `gg` (go-to-top) if it is
    /// a second `g`. Any other key clears it. Reset when the menu opens fresh.
    pending_g: Cell<bool>,
    /// The height caps, so reconcile can build submenu columns that appear at
    /// runtime with the same sizing the initial build used.
    ctx: construct::BuildCtx,
}

impl TrayMenu {
    pub(super) fn is_visible(&self) -> bool {
        self.root.popover.is_visible()
    }

    pub(super) fn popup(&self) {
        // No reset here: the selection is cleared on CLOSE (see the root popover's
        // `connect_closed` in `build`), so the cascade is already collapsed with nothing
        // selected before it reopens — and a freshly built cascade starts that way too.
        self.root.popover.popup();
    }

    /// Patch the whole cascade in place from a fresh layout, reusing the popovers,
    /// buttons, and submenu columns — no teardown, no surface churn. Safe whether
    /// the menu is hidden or visible (a visible reconcile is the point: an
    /// AboutToShow/LayoutUpdated change updates the open menu without a flicker).
    pub(super) fn reconcile(&self, new_root: &MenuItem) {
        construct::reconcile_column(&self.inner, &self.root, &new_root.children);
    }

    /// Unparent every popover, deepest-first.
    pub(super) fn teardown(&self) {
        construct::teardown_column(&self.root);
    }

    /// The root popover, for the caller to hook `connect_closed` on.
    pub(super) fn root_popover(&self) -> &gtk::Popover {
        &self.root.popover
    }

    /// A closure that dismisses the whole menu (for the open-surface coordinator).
    pub(super) fn dismiss_handle(&self) -> Rc<dyn Fn()> {
        let popover = self.root.popover.downgrade();
        Rc::new(move || {
            if let Some(popover) = popover.upgrade() {
                popover.popdown();
            }
        })
    }

    /// A key handler for the coordinator: nav keys forwarded from the bar/scrim
    /// (whichever holds keyboard focus) drive the cascade. Returns `true` when the
    /// key was consumed.
    pub(super) fn key_handler(&self) -> Rc<dyn Fn(gtk::gdk::Key, gtk::gdk::ModifierType) -> bool> {
        let inner = self.inner.clone();
        let root = self.root.clone();
        Rc::new(move |key, state| inner.handle_key(&root, key, state))
    }
}

/// Build a menu for `item` rooted at `root_item`, parenting the root popover to
/// `parent` (the tray button). `scale` is the bar scale (root bar-gap offset, to
/// match the dropdown panels) and `styling_scale` the styling scale (submenu flush
/// offset, to match the panel's `space-xs` padding).
pub(super) fn build(
    item: &Arc<TrayItem>,
    root_item: &MenuItem,
    parent: &gtk::Widget,
    scale: f32,
    styling_scale: f32,
) -> TrayMenu {
    let ctx = construct::build_ctx(parent, scale, styling_scale);
    let root = construct::build_root(&ctx, root_item, parent);

    let inner = Rc::new(MenuInner {
        item: item.clone(),
        root_popover: root.popover.clone(),
        navigating: Cell::new(false),
        keyboard_nav: Cell::new(false),
        selected: RefCell::new(None),
        pointer_pos: Cell::new(None),
        pending_g: Cell::new(false),
        ctx,
    });

    construct::wire_columns(&inner, &root);

    // Deselect whenever the menu closes, so no `.selected` button lingers on the cached
    // (reused) surface. Clearing it here — while hidden — means the button's fade-out
    // `transition` runs invisibly; clearing it on the next OPEN instead (as a lone
    // `popup`-time reset does) makes that fade flash for a frame as the popover appears.
    // Weak refs avoid a cycle: the popover lives inside `root`.
    root.popover.connect_closed({
        let inner = Rc::downgrade(&inner);
        let root = Rc::downgrade(&root);
        move |_| {
            if let (Some(inner), Some(root)) = (inner.upgrade(), root.upgrade()) {
                inner.reset(&root);
            }
        }
    });

    TrayMenu { inner, root }
}

/// Follow the open-submenu chain from `root` to the deepest column whose popover is
/// currently open (returns `root` itself when no submenu is open) — the level a fresh
/// keyboard nav starts in (see [`MenuInner::active`]).
fn deepest_open_column(root: &Rc<MenuColumn>) -> Rc<MenuColumn> {
    let mut column = root.clone();
    loop {
        let child = column.open_child.borrow().clone();
        match child {
            Some(child) if child.popover.is_visible() => column = child,
            _ => return column,
        }
    }
}

impl MenuInner {
    /// Move the single selection to row `index` of `column` — the one entry that is
    /// highlighted, shown with its submenu, and activated by Enter. Pointer and
    /// keyboard both route here, so an entry behaves identically however it is
    /// reached: clear any prior selection (in this or another column), highlight this
    /// row, show its submenu one level deep (collapsing anything deeper), and — when
    /// `scroll` — bring it into view. `scroll` is off for pointer selection (the row
    /// is already under the pointer; scrolling would fight the mouse) and on for
    /// keyboard nav.
    fn select(&self, column: &Rc<MenuColumn>, index: usize, scroll: bool) {
        // Single selection: clear the previous row when it lived in a DIFFERENT column
        // (a same-column move is cleared by `set_selected` itself).
        let previous = self.selected.borrow().as_ref().and_then(Weak::upgrade);
        if let Some(prev_column) = previous
            && !Rc::ptr_eq(&prev_column, column)
        {
            prev_column.set_selected(None);
        }
        column.set_selected(Some(index));
        *self.selected.borrow_mut() = Some(Rc::downgrade(column));
        if scroll {
            column.scroll_into_view(index);
        }
        // Show the selected entry's submenu one level deep (as hovering it does),
        // collapsing any deeper level a prior descent opened. Guarded: the popup/
        // popdown can put a row under a stationary pointer and synthesize a crossing.
        self.navigating.set(true);
        match column.submenu_at(index) {
            Some(child) => {
                if column.open_child(&child) {
                    self.notify_about_to_show(column, index);
                    verify_submenu_presented(column, &child, column.row_id(index));
                }
                child.close_open_child();
            }
            None => column.close_open_child(),
        }
        self.navigating.set(false);
    }

    /// Tell the app a submenu is about to be shown (DBusMenu `AboutToShow`), so apps
    /// that build or refresh submenu contents on demand do so, and re-fetch the layout
    /// when the app reports it changed (`needsUpdate`). Fired once per submenu open
    /// (see [`MenuColumn::open_child`]), off the UI thread; a failure (the app doesn't
    /// implement `AboutToShow`) is expected and only logged.
    fn notify_about_to_show(&self, column: &Rc<MenuColumn>, index: usize) {
        let Some(id) = column.row_id(index) else {
            return;
        };
        let item = self.item.clone();
        tokio::spawn(async move {
            match item.menu_about_to_show(id).await {
                // The app repopulated/changed the submenu — pull the fresh layout, which
                // reconciles the open cascade in place.
                Ok(true) => {
                    if let Err(error) = item.refresh_menu().await {
                        error!(error = %error, id, "cannot refresh menu after AboutToShow");
                    }
                }
                Ok(false) => {}
                Err(error) => debug!(error = %error, id, "submenu AboutToShow unsupported"),
            }
        });
    }

    /// The currently-selected `(column, index)`, or `(root, None)` when nothing is
    /// selected yet (so the first Up/Down starts from the root). The index is read
    /// fresh from the column's `cursor` so a reconcile that clamps it is reflected;
    /// falls back to the root if the selected column is no longer visible.
    fn active(&self, root: &Rc<MenuColumn>) -> (Rc<MenuColumn>, Option<usize>) {
        let selected = self.selected.borrow().as_ref().and_then(Weak::upgrade);
        match selected {
            Some(column) if column.popover.is_visible() => {
                let index = column.cursor.get();
                (column, index)
            }
            // Nothing selected: start in the DEEPEST currently-open submenu rather than
            // the root. A submenu can be open with nothing selected once the pointer has
            // left the row that opened it (see `hover_leave`), and the first nav key
            // should drop into the level the user is actually looking at.
            _ => (deepest_open_column(root), None),
        }
    }

    /// Hover moved onto row `index` of `column`: the pointer selects it (moving the
    /// mouse over an entry always selects it). Skipped while a structural change is in
    /// flight, or when the column's popover is closing/closed — a synthetic motion as
    /// a submenu pops down must not re-select a vanishing column.
    fn hover_row(&self, column: &Rc<MenuColumn>, index: usize) {
        if self.navigating.get() || !column.popover.is_visible() {
            return;
        }
        self.select(column, index, false);
    }

    /// The pointer left row `index` of `column`: deselect it, so a button never stays
    /// highlighted once the mouse is off it — INCLUDING a submenu row. For a submenu row
    /// only the `.selected` highlight is cleared; its submenu is left OPEN (the pointer
    /// may be heading into it), because [`MenuColumn::set_selected`] touches only the
    /// cursor/highlight, never `open_child`. A row that is no longer THE selection is
    /// left alone (the pointer already crossed onto and selected another row — leaving
    /// the old one must not clear the new one). Skipped during keyboard nav, where a row
    /// scrolling out from under a stationary pointer fires a synthetic leave that must
    /// not clear the keyboard cursor.
    fn hover_leave(&self, column: &Rc<MenuColumn>, index: usize) {
        // Ignore SYNTHETIC leaves so only a real pointer leave clears the selection:
        // during keyboard nav (a row scrolling out from under a stationary pointer) and
        // while a structural change is in flight or the column is closing (a popping-down
        // submenu sliding rows under the pointer).
        if self.keyboard_nav.get() || self.navigating.get() || !column.popover.is_visible() {
            return;
        }
        let selected_here = self
            .selected
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|selected| Rc::ptr_eq(&selected, column))
            && column.cursor.get() == Some(index);
        if selected_here {
            column.set_selected(None);
            *self.selected.borrow_mut() = None;
        }
    }

    fn activate_leaf(&self, id: i32) {
        let item = self.item.clone();
        tokio::spawn(async move {
            let timestamp = Utc::now().timestamp().max(0) as u32;
            if let Err(error) = item.menu_event(id, MenuEvent::Clicked, timestamp).await {
                error!(error = %error, "cannot send menu event");
            }
        });
        self.root_popover.popdown();
    }

    /// Collapse to the root with nothing selected. Run when the menu closes (see the
    /// root popover's `connect_closed` in `build`), so the cached cascade is already
    /// clean — no lingering `.selected` highlight to fade in as it reopens.
    fn reset(&self, root: &Rc<MenuColumn>) {
        let previous = self.selected.borrow().as_ref().and_then(Weak::upgrade);
        if let Some(prev_column) = previous {
            prev_column.set_selected(None);
        }
        *self.selected.borrow_mut() = None;
        self.pending_g.set(false);
        root.close_open_child();
        root.set_selected(None);
    }

    /// Dispatch a forwarded nav key (with its modifier `state`) against the selected
    /// entry (or the root when nothing is selected yet).
    ///
    /// - Up/Down (vim `k`/`j`) move within the column; Right (`l`) descends into the
    ///   submenu; Left (`h`) ascends to the entry that owns the column.
    /// - `gg` jumps to the top of the column, Shift+`G` to the bottom.
    /// - Ctrl+`d`/`u` move a half page down/up and Ctrl+`f`/`b` a full page, clamped to
    ///   the ends (no wrap); a "page" is how many rows fit in the visible viewport.
    /// - Enter/Space activate; Escape dismisses (normally already intercepted by the
    ///   bar/scrim's `dismiss_current`, but handling it here too is harmless).
    ///
    /// Every motion sets `keyboard_nav`, which stops the synthetic hover-enter GTK
    /// fires when `scroll_into_view` slides a row under a stationary pointer from
    /// snapping the selection off the keyboard cursor (until the next real pointer
    /// move).
    fn handle_key(
        &self,
        root: &Rc<MenuColumn>,
        key: gtk::gdk::Key,
        state: gtk::gdk::ModifierType,
    ) -> bool {
        use gtk::gdk::Key;

        let (column, index) = self.active(root);
        // A leading `g` armed the previous keypress; consume the arming now — only a
        // second `g` (matched below) completes `gg`; every other key just clears it.
        let pending_g = self.pending_g.replace(false);

        // Ctrl page motions: half page (d/u) or full page (f/b), down (+) or up (−),
        // clamped to the ends without wrapping. Any other Ctrl combo falls through to
        // be handled (or ignored) as if unmodified.
        if state.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            let full = i32::try_from(column.page_rows()).unwrap_or(1);
            let half = (full / 2).max(1);
            let steps = match key {
                Key::d => Some(half),
                Key::u => Some(-half),
                Key::f => Some(full),
                Key::b => Some(-full),
                _ => None,
            };
            if let Some(steps) = steps {
                self.keyboard_nav.set(true);
                if let Some(target) = column.step_by(index, steps) {
                    self.select(&column, target, true);
                }
                return true;
            }
        }

        match key {
            // Vim keys (h/j/k/l) are accepted as aliases for the arrows and behave
            // identically.
            Key::Down | Key::j => {
                self.keyboard_nav.set(true);
                if let Some(next) = column.next_from(index) {
                    self.select(&column, next, true);
                }
            }
            Key::Up | Key::k => {
                self.keyboard_nav.set(true);
                if let Some(prev) = column.prev_from(index) {
                    self.select(&column, prev, true);
                }
            }
            // `gg` — go to the top. The first `g` only arms (next arm); this fires on
            // the second consecutive `g`.
            Key::g if pending_g => {
                self.keyboard_nav.set(true);
                if let Some(top) = column.first_selectable() {
                    self.select(&column, top, true);
                }
            }
            // Lone `g`: arm the `gg` sequence and wait for the next key.
            Key::g => self.pending_g.set(true),
            // Shift+`G` — go to the bottom.
            Key::G => {
                self.keyboard_nav.set(true);
                if let Some(bottom) = column.prev_from(None) {
                    self.select(&column, bottom, true);
                }
            }
            Key::Right | Key::l => self.enter(&column, index),
            Key::Left | Key::h => self.leave(&column),
            Key::Return | Key::KP_Enter | Key::space => self.activate(&column, index),
            Key::Escape => self.root_popover.popdown(),
            _ => return false,
        }
        true
    }

    /// Right: descend into the selected entry's submenu (already shown, since
    /// selecting the entry opened it), selecting its first row. No-op if the entry has
    /// no submenu.
    fn enter(&self, column: &Rc<MenuColumn>, index: Option<usize>) {
        self.keyboard_nav.set(true);
        let Some(index) = index else {
            return;
        };
        let Some(child) = column.submenu_at(index) else {
            return;
        };
        // Normally already open (selecting `index` opened it); ensure it, guarding the
        // popup against a synthetic crossing under a stationary pointer.
        if !child.popover.is_visible() {
            self.navigating.set(true);
            column.open_child(&child);
            self.navigating.set(false);
        }
        if let Some(first) = child.first_selectable() {
            self.select(&child, first, true);
        }
    }

    /// Left: ascend to the entry that owns this column, selecting it. That entry's
    /// submenu IS this column, so it stays shown — nothing under the pointer is
    /// unmapped, so the compositor never defers keyboard focus (this is why arrow keys
    /// keep working after Left). No-op at the root, which has no parent.
    fn leave(&self, column: &Rc<MenuColumn>) {
        self.keyboard_nav.set(true);
        if column.depth == 0 {
            return;
        }
        let Some(parent) = column.parent.upgrade() else {
            return;
        };
        let Some(owner) = parent.index_of_submenu(column) else {
            return;
        };
        self.select(&parent, owner, true);
    }

    /// Enter/Space: activate the selected leaf, or descend into its submenu.
    fn activate(&self, column: &Rc<MenuColumn>, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        match column.activation_at(index) {
            Some(RowActivation::Leaf(id)) => self.activate_leaf(id),
            Some(RowActivation::Submenu) => self.enter(column, Some(index)),
            None => {}
        }
    }
}

/// After a submenu is opened, verify a beat later that its popover actually presented.
///
/// A submenu is an independent, non-grab nested `xdg_popup`; deep ones (a grandchild
/// panel — a popover inside a popover inside a popover) intermittently fail to map or
/// map blank, and because the cascade is persistent (built once, only reconciled) the
/// failure sticks for the session until it is rebuilt. This is silent in normal use —
/// it only `warn!`s when a submenu we just popped up is still meant to be showing yet
/// isn't mapped/sized, i.e. the "nested dropdown panel didn't appear" bug — so it is
/// safe to ship and gives a reproducer's logs the fingerprint to diagnose it.
fn verify_submenu_presented(parent: &Rc<MenuColumn>, child: &Rc<MenuColumn>, submenu_id: Option<i32>) {
    let parent = Rc::downgrade(parent);
    let child = Rc::downgrade(child);
    // A generous delay so a merely-slow present is never mistaken for a failure (no
    // false positives in shipped logs); a genuinely-failed popover stays unmapped.
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
        let (Some(parent), Some(child)) = (parent.upgrade(), child.upgrade()) else {
            return;
        };
        // Not an anomaly if the cascade closed or the pointer/keyboard moved to a
        // different entry (so this submenu was legitimately collapsed) meanwhile.
        if !parent.popover.is_visible() {
            return;
        }
        let still_open = parent
            .open_child
            .borrow()
            .as_ref()
            .is_some_and(|open| Rc::ptr_eq(open, &child));
        if !still_open {
            return;
        }

        let popover = &child.popover;
        let mapped = popover.is_mapped();
        let content_sized = popover
            .child()
            .is_some_and(|content| content.width() > 0 && content.height() > 0);
        if !mapped || !content_sized {
            warn!(
                depth = child.depth,
                submenu_id = submenu_id.unwrap_or(-1),
                visible = popover.is_visible(),
                mapped,
                content_sized,
                width = popover.width(),
                height = popover.height(),
                "systray submenu was opened but its popover did not present (missing or \
                 blank nested menu panel) — please report this log and the lines around it",
            );
        }
    });
}
