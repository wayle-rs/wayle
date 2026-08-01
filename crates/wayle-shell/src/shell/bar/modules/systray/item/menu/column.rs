//! One column of the cascading tray menu: its own [`gtk::Popover`] holding a
//! vertically-scrolling list of rows, plus the hover-selection cursor and the
//! submenu currently open beneath it.
//!
//! Each column is an independent non-grab popover, so the cascade positions,
//! flips, and scrolls per surface, and each is naturally its own floating card.
//! The menu is mouse-driven; the `.selected` CSS class is the hover highlight.
//!
//! The column is built once and then updated in place by
//! [`construct::reconcile_column`](super::construct): `rows`, each row's `kind`,
//! and the `list`'s children are interior-mutable so a menu-layout change patches
//! the existing widgets instead of recreating the popover — the cascade surface is
//! never rebuilt while it lives.

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use relm4::gtk::{self, prelude::*};

/// A single menu column.
pub(super) struct MenuColumn {
    /// This column's popover surface (parented to the owning row button, or the
    /// tray button for the root).
    pub(super) popover: gtk::Popover,
    /// The vertical list box inside the scroller, holding the row buttons and the
    /// separators between them. Reconcile appends/inserts/removes children here.
    pub(super) list: gtk::Box,
    /// Rows in visual order (separators are in `list` but are tracked separately in
    /// `separators`, not here). Interior-mutable so reconcile can update/insert/
    /// remove rows without recreating the column.
    pub(super) rows: RefCell<Vec<MenuRow>>,
    /// The separator widgets currently in `list`. Separators carry no menu-item
    /// identity, so reconcile drops and re-derives them wholesale rather than
    /// diffing.
    pub(super) separators: RefCell<Vec<gtk::Separator>>,
    /// Index of the hovered/selected row, or `None`.
    pub(super) cursor: Cell<Option<usize>>,
    /// Depth in the cascade; `0` is the root column.
    pub(super) depth: usize,
    /// The submenu column currently shown beneath one of this column's rows.
    pub(super) open_child: RefCell<Option<Rc<MenuColumn>>>,
    /// Back-link to the parent column (empty on the root); used to clear the
    /// parent's `open_child` when this column's popover closes for any reason.
    pub(super) parent: Weak<MenuColumn>,
}

/// A menu row: a button, its content box (patched in place), and what activating
/// it does.
pub(super) struct MenuRow {
    /// The DBusMenu item id this row renders. Used for leaf activation; reconcile
    /// matches rows by position (not id), so a non-unique id is harmless here.
    pub(super) id: i32,
    pub(super) button: gtk::Button,
    /// The horizontal content box (icon/indicator + label + submenu-arrow/accel),
    /// updated in place by reconcile.
    pub(super) content: gtk::Box,
    /// Interior-mutable so a row can transition Leaf<->Submenu on reconcile.
    pub(super) kind: RefCell<RowKind>,
}

pub(super) enum RowKind {
    Leaf,
    Submenu { column: Rc<MenuColumn> },
}

/// What activating a row does, resolved at click/keyboard time.
pub(super) enum RowActivation {
    Leaf(i32),
    Submenu,
}

impl MenuColumn {
    /// Move the `.selected` (hover) marker to `next` (or clear it when `None`).
    /// Early-returns when the index is unchanged, so it is cheap on repeated hover.
    pub(super) fn set_selected(&self, next: Option<usize>) {
        let previous = self.cursor.replace(next);
        if previous == next {
            return;
        }
        let rows = self.rows.borrow();
        if let Some(row) = previous.and_then(|index| rows.get(index)) {
            row.button.remove_css_class("selected");
        }
        if let Some(row) = next.and_then(|index| rows.get(index)) {
            row.button.add_css_class("selected");
        }
    }

    /// Force `.selected` onto exactly the row at `index` (or clear it), bypassing
    /// [`set_selected`](Self::set_selected)'s unchanged-index early-return. Used
    /// after a reconcile, where the row at a given index may have changed identity
    /// (so the class must be re-applied even when the numeric index is unchanged).
    pub(super) fn reselect(&self, index: Option<usize>) {
        let rows = self.rows.borrow();
        for row in rows.iter() {
            row.button.remove_css_class("selected");
        }
        self.cursor.set(index.filter(|&i| i < rows.len()));
        if let Some(row) = self.cursor.get().and_then(|index| rows.get(index)) {
            row.button.add_css_class("selected");
        }
    }

    /// Show `child`'s popover beneath one of this column's rows, closing any
    /// other child first. No-op if `child` is already the open one. Returns `true`
    /// when it actually popped `child` up (i.e. it was not already open), so the
    /// caller can fire a one-shot side effect — a DBusMenu `AboutToShow` — exactly
    /// once per open rather than on every hover tick.
    pub(super) fn open_child(&self, child: &Rc<MenuColumn>) -> bool {
        let already_open = self
            .open_child
            .borrow()
            .as_ref()
            .is_some_and(|open| Rc::ptr_eq(open, child));
        if already_open {
            return false;
        }
        self.close_open_child();
        child.set_selected(None);
        child.popover.popup();
        *self.open_child.borrow_mut() = Some(child.clone());
        true
    }

    /// Close the open submenu (and, via `cascade_popdown`, its descendants).
    /// Takes the child out before popping down so no borrow is held across the
    /// GTK call.
    pub(super) fn close_open_child(&self) {
        let child = self.open_child.borrow_mut().take();
        if let Some(child) = child {
            child.popover.popdown();
        }
    }

    /// The index of the row whose button is `button`, by widget identity. Row
    /// handlers look their index up at fire time (rather than capturing it) so
    /// reconcile's insert/remove/reorder can't leave a handler pointing at the
    /// wrong row.
    pub(super) fn index_of_button(&self, button: &gtk::Button) -> Option<usize> {
        self.rows
            .borrow()
            .iter()
            .position(|row| &row.button == button)
    }

    /// The activation for row `index`, read at fire time so a Leaf<->Submenu
    /// transition on reconcile is reflected without re-wiring the button.
    pub(super) fn activation_at(&self, index: usize) -> Option<RowActivation> {
        let rows = self.rows.borrow();
        let row = rows.get(index)?;
        Some(match &*row.kind.borrow() {
            RowKind::Leaf => RowActivation::Leaf(row.id),
            RowKind::Submenu { .. } => RowActivation::Submenu,
        })
    }

    /// The DBusMenu item id of row `index` (the id whose submenu is being shown, for
    /// an `AboutToShow` notification).
    pub(super) fn row_id(&self, index: usize) -> Option<i32> {
        self.rows.borrow().get(index).map(|row| row.id)
    }

    /// The submenu column owned by row `index`, if that row is a submenu.
    pub(super) fn submenu_at(&self, index: usize) -> Option<Rc<MenuColumn>> {
        match &*self.rows.borrow().get(index)?.kind.borrow() {
            RowKind::Submenu { column } => Some(column.clone()),
            RowKind::Leaf => None,
        }
    }

    /// The index of the row whose submenu is `child` (by column identity), so Left can
    /// select the entry that owns a submenu column when ascending to it.
    pub(super) fn index_of_submenu(&self, child: &Rc<MenuColumn>) -> Option<usize> {
        self.rows.borrow().iter().position(|row| {
            matches!(&*row.kind.borrow(), RowKind::Submenu { column } if Rc::ptr_eq(column, child))
        })
    }

    /// First sensitive row (keyboard nav entry point), or `None` if the column
    /// has no selectable rows.
    pub(super) fn first_selectable(&self) -> Option<usize> {
        self.next_from(None)
    }

    /// Next sensitive row after `current` (wrapping); starts at row 0 when
    /// `current` is `None`. Skips insensitive (disabled) rows.
    pub(super) fn next_from(&self, current: Option<usize>) -> Option<usize> {
        let rows = self.rows.borrow();
        let count = rows.len();
        if count == 0 {
            return None;
        }
        let start = current.map_or(0, |index| (index + 1) % count);
        (0..count).find_map(|offset| {
            let index = (start + offset) % count;
            rows[index].button.is_sensitive().then_some(index)
        })
    }

    /// Previous sensitive row before `current` (wrapping); starts at the last row
    /// when `current` is `None`. Skips insensitive (disabled) rows.
    pub(super) fn prev_from(&self, current: Option<usize>) -> Option<usize> {
        let rows = self.rows.borrow();
        let count = rows.len();
        if count == 0 {
            return None;
        }
        let start = current.map_or(count - 1, |index| (index + count - 1) % count);
        (0..count).find_map(|offset| {
            let index = (start + count - offset) % count;
            rows[index].button.is_sensitive().then_some(index)
        })
    }

    /// Move `steps` sensitive rows down (positive) or up (negative) from `current`,
    /// clamping at the ends WITHOUT wrapping and skipping insensitive rows — backs the
    /// half/full-page motions (Ctrl-D/U/F/B). `None` starts just past the near edge
    /// (top for a downward step, bottom for an upward one), so a page motion from an
    /// unselected menu jumps into the list. Returns `None` if there are no sensitive
    /// rows.
    pub(super) fn step_by(&self, current: Option<usize>, steps: i32) -> Option<usize> {
        let rows = self.rows.borrow();
        let sensitive: Vec<usize> = (0..rows.len())
            .filter(|&index| rows[index].button.is_sensitive())
            .collect();
        let len = sensitive.len();
        if len == 0 {
            return None;
        }
        let pos = current
            .and_then(|cur| sensitive.iter().position(|&index| index == cur))
            .map_or(if steps >= 0 { -1 } else { len as isize }, |p| p as isize);
        let target = (pos + isize::try_from(steps).unwrap_or(0)).clamp(0, len as isize - 1) as usize;
        Some(sensitive[target])
    }

    /// How many rows are visible in the scroll viewport at once — `floor(viewport
    /// height / row height)` — sizing the page-motion jumps. Falls back to 1 before the
    /// column has been allocated (or has no measurable row), so a motion always moves.
    pub(super) fn page_rows(&self) -> usize {
        let Some(scrolled) = self.popover.child().and_downcast::<gtk::ScrolledWindow>() else {
            return 1;
        };
        let viewport = scrolled.vadjustment().page_size();
        let row_height = self
            .rows
            .borrow()
            .iter()
            .map(|row| row.button.height())
            .find(|&height| height > 0)
            .unwrap_or(0);
        if row_height <= 0 || viewport <= 0.0 {
            return 1;
        }
        ((viewport / f64::from(row_height)).floor() as usize).max(1)
    }

    /// Scroll this column so row `index` is visible (keyboard nav can land on a
    /// row currently scrolled out of view in a tall column, e.g. a country list).
    pub(super) fn scroll_into_view(&self, index: usize) {
        let rows = self.rows.borrow();
        let Some(row) = rows.get(index) else {
            return;
        };
        let Some(scrolled) = self.popover.child().and_downcast::<gtk::ScrolledWindow>() else {
            return;
        };
        let Some(bounds) = row.button.compute_bounds(&scrolled) else {
            return;
        };
        let vadjustment = scrolled.vadjustment();
        let top = f64::from(bounds.y());
        let bottom = top + f64::from(bounds.height());
        let page = vadjustment.page_size();
        if top < 0.0 {
            vadjustment.set_value(vadjustment.value() + top);
        } else if bottom > page {
            vadjustment.set_value(vadjustment.value() + (bottom - page));
        }
    }
}
