//! Builds each menu column from the `wayle_systray` [`MenuItem`] tree, wires its
//! popover (positioning, submenu alignment) and rows (hover/click), and — the
//! reactive core — [`reconcile_column`] patches a built column in place when the
//! layout changes, so the popover cascade is created once and never rebuilt while
//! it lives.
//!
//! Every column is its own non-grab popover parented to the row that owns it (the
//! root to the tray button); it's popped up/down as the cascade opens and closes.
//! There are no per-column key controllers: keyboard comes from the scrim/bar,
//! which forward keys to the deepest-open column (see `menu`'s
//! `key_handler`/`handle_key`).

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use gtk4::{gdk, glib};
use relm4::gtk::{self, prelude::*};
use wayle_systray::types::menu::{Disposition, MenuItem, MenuItemType, ToggleState, ToggleType};

use super::MenuInner;
use super::column::{MenuColumn, MenuRow, RowActivation, RowKind};

/// Fraction of the monitor height a *submenu* column may occupy before it
/// scrolls. Submenus open sideways from a row whose on-screen position isn't
/// knowable across Wayland surfaces, so they fall back to a monitor fraction.
const MAX_HEIGHT_FRACTION: f64 = 0.5;
/// Used when the monitor geometry can't be read (surface not yet realized).
const FALLBACK_MONITOR_HEIGHT: i32 = 1080;
/// Gap kept between a column and the far monitor edge. `set_max_content_height`
/// caps only the ScrolledWindow *content*, but the popover *surface* also carries
/// frame chrome (CSS padding and border) that scales with the theme — up to
/// ~20px at 2x. Subtracting this margin from the computed available space keeps
/// content, chrome, and rounding together within what the compositor will grant;
/// otherwise wlroots RESIZE/SLIDE-clamps the xdg_popup, dropping its top edge to
/// the panel and clipping the last row. Erring large costs at most ~1 row of
/// unused space, in exchange for a menu that fits never scrolling or clipping.
const MENU_EDGE_MARGIN: i32 = 24;
/// Floor so a menu opened from an oddly-placed anchor never collapses to nothing.
const MIN_MENU_HEIGHT: i32 = 120;
/// Gap between the bar's outer edge and the root menu card, matched to the
/// dropdown panels' `DropdownMargins::GAP_REM` (dropdowns/registry.rs) so the
/// systray menu sits the same distance from the bar. Applied as a popover offset
/// (see [`realign_root`]).
const ROOT_GAP_REM: f32 = 0.275;
/// Logical pixels per rem, matching the dropdowns' `REM_PX`.
const REM_PX: f32 = 16.0;
/// A submenu is shifted out so its card meets the parent panel's edge rather than
/// tucking under the parent row button. The shift clears BOTH the parent panel's
/// right contents padding AND the child submenu's own left contents padding (the
/// child card extends left of its anchor by that padding), i.e. `2 × space-xs`
/// (`$base-space-xs`, scaled by `styling.scale` / `--global-scale`). A token, not a
/// measurement — the panel's min-width leaves centred empty space around the rows,
/// so every widget's position in the panel is offset by it.
const SUBMENU_FLUSH_REM: f32 = 0.5;

/// The per-column height caps, computed once from the tray button's geometry and
/// reused for every (re)build of a column — including submenu columns created
/// during reconcile.
#[derive(Clone, Copy)]
pub(super) struct BuildCtx {
    /// Cap for the root column: the real space on the menu's side of the bar, so
    /// its natural height never exceeds what the compositor grants past the panel
    /// (see [`root_available_height`]).
    root_max_height: i32,
    /// Cap for submenu columns — a monitor fraction (see [`MAX_HEIGHT_FRACTION`]).
    submenu_max_height: i32,
    /// The configured bar scale (`bar.scale`), for the root menu's bar-gap offset,
    /// which matches the dropdown panels' `DropdownMargins` (see [`realign_root`]).
    scale: f32,
    /// The configured styling scale (`styling.scale` / `--global-scale`), for the
    /// submenu's flush offset, which matches the panel's `space-xs` contents padding
    /// (see [`realign_submenu`]).
    styling_scale: f32,
}

/// Compute the height caps from the tray button's on-screen geometry; `scale` is the
/// bar scale (root bar-gap) and `styling_scale` the styling scale (submenu flush).
pub(super) fn build_ctx(tray_button: &gtk::Widget, scale: f32, styling_scale: f32) -> BuildCtx {
    BuildCtx {
        root_max_height: root_available_height(tray_button),
        submenu_max_height: submenu_max_height(tray_button),
        scale,
        styling_scale,
    }
}

/// Build the root column, parenting its popover to `tray_button`; submenu popovers
/// hang off their owning row and are built eagerly here too.
pub(super) fn build_root(
    ctx: &BuildCtx,
    root_item: &MenuItem,
    tray_button: &gtk::Widget,
) -> Rc<MenuColumn> {
    build_column(
        ctx,
        &root_item.children,
        0,
        Weak::new(),
        tray_button,
        gtk::PositionType::Bottom,
    )
}

fn build_column(
    ctx: &BuildCtx,
    children: &[MenuItem],
    depth: usize,
    parent: Weak<MenuColumn>,
    parent_widget: &gtk::Widget,
    position: gtk::PositionType,
) -> Rc<MenuColumn> {
    let popover = gtk::Popover::new();
    popover.set_has_arrow(false);
    // No grab. Nested *autohide* popovers deadlock the wlroots/Hyprland popup
    // grab chain (moving child→parent leaves a stuck grab that freezes all
    // input); *non-autohide* children under an autohide root get no pointer
    // events at all. With no grab anywhere, every popover surface receives its
    // own pointer/scroll/keyboard events and nothing can wedge. Trade-off: no
    // auto-dismiss on outside click — the menu closes on Escape, item
    // activation, or a tray re-click.
    popover.set_autohide(false);
    // Popping a popover down still closes its descendants (deepest-first).
    popover.set_cascade_popdown(true);
    popover.set_position(position);
    popover.add_css_class("systray-menu");
    popover.set_parent(parent_widget);

    // The root column is capped to the space on the menu's side of the bar; deeper
    // (submenu) columns open sideways and use the monitor-fraction fallback.
    let max_content_height = if depth == 0 {
        ctx.root_max_height
    } else {
        ctx.submenu_max_height
    };

    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_propagate_natural_height(true);
    scrolled.set_propagate_natural_width(true);
    scrolled.set_max_content_height(max_content_height);
    scrolled.set_focusable(false);
    scrolled.add_css_class("systray-menu-column");

    let list = gtk::Box::new(gtk::Orientation::Vertical, 0);
    scrolled.set_child(Some(&list));
    popover.set_child(Some(&scrolled));

    Rc::new_cyclic(|weak_self| {
        let (rows, separators) = populate(ctx, &list, children, depth, weak_self);
        MenuColumn {
            popover,
            list,
            rows: RefCell::new(rows),
            separators: RefCell::new(separators),
            cursor: Cell::new(None),
            depth,
            open_child: RefCell::new(None),
            parent,
        }
    })
}

/// Applies the DBusMenu visibility + separator-collapsing rules to `children`,
/// yielding the rows to render in visual order. Leading, trailing, and
/// consecutive separators are collapsed into `sep_before` flags.
struct RowPlan<'a> {
    item: &'a MenuItem,
    has_submenu: bool,
    sep_before: bool,
}

fn plan_rows(children: &[MenuItem]) -> Vec<RowPlan<'_>> {
    let mut plan = Vec::new();
    let mut has_items = false;
    let mut pending_separator = false;

    for child in children {
        if !child.visible {
            continue;
        }
        if child.item_type == MenuItemType::Separator {
            pending_separator = has_items;
            continue;
        }
        // A submenu row with no visible children degrades to a plain leaf.
        let has_submenu = child.has_children() && child.children.iter().any(|item| item.visible);
        plan.push(RowPlan {
            item: child,
            has_submenu,
            sep_before: pending_separator,
        });
        pending_separator = false;
        has_items = true;
    }

    plan
}

fn populate(
    ctx: &BuildCtx,
    list: &gtk::Box,
    children: &[MenuItem],
    depth: usize,
    parent: &Weak<MenuColumn>,
) -> (Vec<MenuRow>, Vec<gtk::Separator>) {
    let mut rows = Vec::new();
    let mut separators = Vec::new();

    for plan in plan_rows(children) {
        if plan.sep_before {
            let sep = separator();
            list.append(&sep);
            separators.push(sep);
        }
        let row = build_row(ctx, &plan, depth, parent);
        list.append(&row.button);
        rows.push(row);
    }

    (rows, separators)
}

/// Build one row (button + content, plus its submenu column eagerly if it has
/// one). Not yet added to any list or wired.
fn build_row(ctx: &BuildCtx, plan: &RowPlan, depth: usize, parent: &Weak<MenuColumn>) -> MenuRow {
    let (button, content) = build_button(plan.item, plan.has_submenu);

    let kind = if plan.has_submenu {
        let child = build_column(
            ctx,
            &plan.item.children,
            depth + 1,
            parent.clone(),
            button.upcast_ref(),
            gtk::PositionType::Right,
        );
        RowKind::Submenu { column: child }
    } else {
        RowKind::Leaf
    };

    MenuRow {
        id: plan.item.id,
        button,
        content,
        kind: RefCell::new(kind),
    }
}

fn build_button(item: &MenuItem, is_submenu: bool) -> (gtk::Button, gtk::Box) {
    let button = gtk::Button::new();
    button.add_css_class("systray-menu-item-button");
    button.set_sensitive(item.enabled);
    // Selection is CSS-driven (`.selected`), not GTK focus; buttons stay
    // unfocusable so keyboard focus never leaves the bar/scrim, from which nav keys
    // are forwarded to the active column via the coordinator (see the `menu`
    // module's `key_handler`/`handle_key`).
    button.set_focusable(false);

    let content = build_content(item, is_submenu);
    button.set_child(Some(&content));
    (button, content)
}

/// Build a row's content box (icon/indicator + label + submenu-arrow/accel). This
/// is rebuilt wholesale by [`update_row`] on reconcile — cheap, and it reuses the
/// enclosing button and (for submenu rows) the child popover cascade, which are
/// the expensive parts.
fn build_content(item: &MenuItem, is_submenu: bool) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    content.add_css_class("systray-menu-item");
    apply_disposition_class(&content, item.disposition);

    if item.is_checkable() {
        content.append(&toggle_indicator(item));
        if item.toggle_state == ToggleState::Checked {
            content.add_css_class("active");
        }
    } else if let Some(icon) = build_icon(item) {
        content.append(&icon);
    }

    let label = gtk::Label::new(None);
    label.set_use_underline(true);
    label.set_label(item.label.as_deref().unwrap_or_default());
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.add_css_class("systray-menu-item-label");
    content.append(&label);

    if is_submenu {
        content.add_css_class("has-submenu");
        let arrow = gtk::Image::from_icon_name("go-next-symbolic");
        arrow.add_css_class("systray-submenu-arrow");
        content.append(&arrow);
    } else if let Some(shortcut) = item.shortcut.as_deref().and_then(format_shortcut) {
        let accel = gtk::Label::new(Some(&shortcut));
        accel.add_css_class("systray-menu-item-shortcut");
        content.append(&accel);
    }

    content
}

// ---------------------------------------------------------------------------
// Reconcile — patch a built column in place from a fresh child list.
// ---------------------------------------------------------------------------

/// Update `column` in place to match `new_children`, reusing the existing popover,
/// buttons, and submenu columns. Rows are matched by position: overlapping rows
/// are patched in place, surplus rows are removed, and new rows are appended — the
/// common menu update (a toggle flip, a label/enabled change) touches only content
/// boxes, never a popover. Recurses into built submenu columns. Position matching
/// (rather than id matching) is deliberate: it is robust to apps that renumber,
/// reuse, or zero their DBusMenu ids, and menu edits are overwhelmingly in-place
/// property changes at stable positions.
pub(super) fn reconcile_column(
    inner: &Rc<MenuInner>,
    column: &Rc<MenuColumn>,
    new_children: &[MenuItem],
) {
    let plan = plan_rows(new_children);
    let ctx = inner.ctx;
    // Submenu columns to tear down / recurse into, collected under the rows borrow
    // and processed after it drops — tearing down a submenu pops it down, which
    // must not run while `rows` is borrowed.
    let mut teardowns: Vec<Rc<MenuColumn>> = Vec::new();
    // Borrow the children slices from `new_children` (which outlives this call)
    // rather than cloning the MenuItem subtrees — a reconcile runs on every layout
    // tick and the subtrees carry icon-data bytes. Only the `Rc<MenuColumn>` is
    // cloned (a refcount bump), since it must outlive the `rows` borrow below.
    let mut recurse: Vec<(Rc<MenuColumn>, &[MenuItem])> = Vec::new();

    {
        let mut rows = column.rows.borrow_mut();

        // Drop surplus tail rows.
        while rows.len() > plan.len() {
            let Some(row) = rows.pop() else {
                break;
            };
            column.list.remove(&row.button);
            if let RowKind::Submenu { column: child } = row.kind.into_inner() {
                teardowns.push(child);
            }
        }

        // Patch overlapping rows; append any new tail rows.
        for (index, plan) in plan.iter().enumerate() {
            if index < rows.len() {
                if let Some(orphan) = update_row(inner, &ctx, column, &mut rows[index], plan) {
                    teardowns.push(orphan);
                }
            } else {
                let row = build_row(&ctx, plan, column.depth, &Rc::downgrade(column));
                column.list.append(&row.button);
                wire_row(inner, column, &row);
                if let RowKind::Submenu { column: child } = &*row.kind.borrow() {
                    wire_columns(inner, child);
                }
                rows.push(row);
            }
        }

        // Collect built submenu columns to reconcile after the borrow drops.
        for (index, plan) in plan.iter().enumerate() {
            if plan.has_submenu
                && let RowKind::Submenu { column: child } = &*rows[index].kind.borrow()
            {
                recurse.push((child.clone(), plan.item.children.as_slice()));
            }
        }
    }

    for child in teardowns {
        teardown_column(&child);
    }

    rederive_separators(column, &plan);

    // Re-apply the selection by index (force the class on, since the row at that
    // index may now be a different one after inserts/removes).
    column.reselect(column.cursor.get());

    for (child, children) in recurse {
        reconcile_column(inner, &child, children);
    }

    // A visible popover whose content/anchor changed needs its offset recomputed
    // (the offsets are one-shot on map otherwise): the root keeps its bar-gap, a
    // submenu its top-alignment.
    if column.popover.is_visible() {
        if column.depth == 0 {
            realign_root(&column.popover, ctx.scale);
        } else {
            realign_submenu(&column.popover, ctx.styling_scale);
        }
    }
}

/// Patch a single existing row to match `plan`. Rebuilds the content box (cheap,
/// keeps the button + any submenu column) and applies a Leaf<->Submenu transition.
/// Returns a submenu column orphaned by a Submenu->Leaf transition, for the caller
/// to tear down outside the `rows` borrow.
fn update_row(
    inner: &Rc<MenuInner>,
    ctx: &BuildCtx,
    column: &Rc<MenuColumn>,
    row: &mut MenuRow,
    plan: &RowPlan,
) -> Option<Rc<MenuColumn>> {
    row.id = plan.item.id;
    row.button.set_sensitive(plan.item.enabled);

    let content = build_content(plan.item, plan.has_submenu);
    row.button.set_child(Some(&content));
    row.content = content;

    let currently_submenu = matches!(&*row.kind.borrow(), RowKind::Submenu { .. });

    if plan.has_submenu && !currently_submenu {
        // Leaf -> Submenu: build and wire a fresh child column (safe under the rows
        // borrow — it is an independent surface and touches no shared state here).
        let child = build_column(
            ctx,
            &plan.item.children,
            column.depth + 1,
            Rc::downgrade(column),
            row.button.upcast_ref(),
            gtk::PositionType::Right,
        );
        wire_columns(inner, &child);
        *row.kind.borrow_mut() = RowKind::Submenu { column: child };
        None
    } else if !plan.has_submenu && currently_submenu {
        // Submenu -> Leaf: orphan the child column for teardown after the borrow.
        match std::mem::replace(&mut *row.kind.borrow_mut(), RowKind::Leaf) {
            RowKind::Submenu { column: child } => Some(child),
            RowKind::Leaf => None,
        }
    } else {
        None
    }
}

/// Drop all separators and re-insert them per `plan` (they carry no identity, so
/// re-deriving is simpler and safer than diffing). Must run after the row buttons
/// are in their final `list` positions.
fn rederive_separators(column: &Rc<MenuColumn>, plan: &[RowPlan]) {
    let mut separators = column.separators.borrow_mut();
    for sep in separators.drain(..) {
        column.list.remove(&sep);
    }

    let rows = column.rows.borrow();
    for (index, plan) in plan.iter().enumerate() {
        // `sep_before` is never set on the first row (leading separators collapse),
        // so `index - 1` is always valid here.
        if plan.sep_before
            && let Some(prev) = index.checked_sub(1).and_then(|i| rows.get(i))
        {
            let sep = separator();
            column.list.insert_child_after(&sep, Some(&prev.button));
            separators.push(sep);
        }
    }
}

// ---------------------------------------------------------------------------
// Wiring — popover lifecycle + per-row hover/click.
// ---------------------------------------------------------------------------

/// Wire a column's popover and all its rows, recursing into submenu columns.
pub(super) fn wire_columns(inner: &Rc<MenuInner>, column: &Rc<MenuColumn>) {
    wire_popover(column, inner.ctx.scale, inner.ctx.styling_scale);

    let children: Vec<Rc<MenuColumn>> = {
        let rows = column.rows.borrow();
        for row in rows.iter() {
            wire_row(inner, column, row);
        }
        rows.iter()
            .filter_map(|row| match &*row.kind.borrow() {
                RowKind::Submenu { column } => Some(column.clone()),
                RowKind::Leaf => None,
            })
            .collect()
    };

    for child in children {
        wire_columns(inner, &child);
    }
}

fn wire_popover(column: &Rc<MenuColumn>, scale: f32, styling_scale: f32) {
    // No per-popover key controller: keyboard focus sits on the bar/scrim, not the
    // (non-focusable) menu columns, so nav keys are forwarded from there via the
    // coordinator (see `menu`'s `key_handler`/`handle_key`).
    if column.depth == 0 {
        // The root anchors to a tray icon button that is shorter than the bar, so
        // GTK would place the menu overlapping the bar; offset it out to the bar's
        // outer edge + a gap on `map` so it sits clear of the bar (matching the
        // dropdown panels) and the compositor never has to re-constrain (shrink) it.
        column
            .popover
            .connect_map(move |popover| realign_root(popover, scale));
        return;
    }

    // Submenus nudge themselves on map so their first row lines up with the row that
    // opened them and their card is flush with the parent panel edge. The flush is
    // baked into the anchor rect (see `realign_submenu`), so it holds whether the
    // compositor opens the submenu to the right of the row or flips it to the left, and
    // survives re-presents (e.g. scrolling the parent) without the popover shifting
    // sides — no post-placement re-run is needed.
    column
        .popover
        .connect_map(move |popover| realign_submenu(popover, styling_scale));

    // When a submenu closes for any reason (Left/Escape, hovering away, or a
    // cascade dismiss), drop the parent's pointer to it so the state stays in
    // sync.
    column.popover.connect_closed({
        let parent = column.parent.clone();
        let this = Rc::downgrade(column);
        move |_| {
            let Some(parent) = parent.upgrade() else {
                return;
            };
            if let Some(this) = this.upgrade() {
                let is_ours = parent
                    .open_child
                    .borrow()
                    .as_ref()
                    .is_some_and(|open| Rc::ptr_eq(open, &this));
                if is_ours {
                    parent.open_child.borrow_mut().take();
                }
            }
        }
    });
}

/// Wire one row's hover + click controllers. The row's index is looked up by
/// button identity at fire time (not captured), and its Leaf/Submenu action is
/// read at fire time, so reconcile's insert/remove/reorder and Leaf<->Submenu
/// transitions need no re-wiring.
fn wire_row(inner: &Rc<MenuInner>, column: &Rc<MenuColumn>, row: &MenuRow) {
    let motion = gtk::EventControllerMotion::new();

    // Hover-select on the pointer crossing into a row — but NOT when the crossing is
    // synthetic, from the menu scrolling a new row under a stationary pointer during
    // keyboard nav (`keyboard_nav`), which would snap the selection off the keyboard
    // cursor. The first real pointer motion clears the flag and snaps to the row the
    // cursor is over.
    motion.connect_enter({
        let inner = Rc::downgrade(inner);
        let column = Rc::downgrade(column);
        let button = row.button.downgrade();
        move |_, _, _| {
            if let (Some(inner), Some(column), Some(button)) =
                (inner.upgrade(), column.upgrade(), button.upgrade())
                && !inner.keyboard_nav.get()
                && let Some(index) = column.index_of_button(&button)
            {
                inner.hover_row(&column, index);
            }
        }
    });
    motion.connect_motion({
        let inner = Rc::downgrade(inner);
        let column = Rc::downgrade(column);
        let button = row.button.downgrade();
        move |controller, _, _| {
            let Some(inner) = inner.upgrade() else {
                return;
            };
            // Ignore SYNTHETIC motion — GTK re-runs pointer targeting (firing a motion)
            // when a popdown/reconcile/scroll slides content under a stationary pointer,
            // which must NOT hand control back to the pointer or it would snap the
            // selection off the keyboard cursor. A real move changes the pointer's
            // surface position; a synthetic one doesn't. (Use the surface position, not
            // the row-local x/y the signal passes: the latter shifts when the row itself
            // scrolls/relayouts under a still pointer.)
            let Some(pos) = controller.current_event().and_then(|event| event.position()) else {
                return;
            };
            if inner.pointer_pos.replace(Some(pos)) == Some(pos) {
                return;
            }
            // Real move: the pointer owns the selection now — end keyboard nav and
            // select the row it is over. This usually just duplicates `connect_enter`
            // (a no-op when the row is already selected), but it is ALSO the only path
            // that selects when `enter` never fires: after the popover closes and
            // reopens under a stationary pointer, wlroots leaves the crossing state
            // stale ("inside" the row it closed over), so moving onto that row produces
            // motion events but no fresh `enter` until the pointer leaves and re-enters.
            inner.keyboard_nav.set(false);
            if let (Some(column), Some(button)) = (column.upgrade(), button.upgrade())
                && let Some(index) = column.index_of_button(&button)
            {
                inner.hover_row(&column, index);
            }
        }
    });
    // De-hovering a row deselects it (unless it opens a submenu — see `hover_leave`),
    // so a highlighted button never stays highlighted once the pointer moves off it.
    motion.connect_leave({
        let inner = Rc::downgrade(inner);
        let column = Rc::downgrade(column);
        let button = row.button.downgrade();
        move |_| {
            if let (Some(inner), Some(column), Some(button)) =
                (inner.upgrade(), column.upgrade(), button.upgrade())
                && let Some(index) = column.index_of_button(&button)
            {
                inner.hover_leave(&column, index);
            }
        }
    });
    row.button.add_controller(motion);

    row.button.connect_clicked({
        let inner = Rc::downgrade(inner);
        let column = Rc::downgrade(column);
        let button = row.button.downgrade();
        move |_| {
            let (Some(inner), Some(column), Some(button)) =
                (inner.upgrade(), column.upgrade(), button.upgrade())
            else {
                return;
            };
            let Some(index) = column.index_of_button(&button) else {
                return;
            };
            match column.activation_at(index) {
                Some(RowActivation::Leaf(id)) => inner.activate_leaf(id),
                Some(RowActivation::Submenu) => inner.hover_row(&column, index),
                None => {}
            }
        }
    });
}

/// Unparent every popover in the cascade, deepest-first (submenus are parented to
/// buttons inside their parent popover, so they must go before the parent).
pub(super) fn teardown_column(column: &Rc<MenuColumn>) {
    let children: Vec<Rc<MenuColumn>> = column
        .rows
        .borrow()
        .iter()
        .filter_map(|row| match &*row.kind.borrow() {
            RowKind::Submenu { column } => Some(column.clone()),
            RowKind::Leaf => None,
        })
        .collect();
    for child in children {
        teardown_column(&child);
    }

    column.open_child.borrow_mut().take();
    column.popover.popdown();
    column.popover.set_child(gtk::Widget::NONE);
    column.popover.unparent();
}

/// Offset a submenu popover so (vertically) its top lines up with the row it
/// opened from — instead of GTK's default vertical centring, which pushes tall
/// submenus off the top of the screen — and (horizontally) its card is flush with
/// the parent *panel's* edge rather than the parent *row button's* edge. Wired on
/// `map` and re-run by reconcile when a visible submenu's height changes.
fn realign_submenu(popover: &gtk::Popover, styling_scale: f32) {
    let (Some(anchor), Some(content)) = (popover.parent(), popover.child()) else {
        return;
    };

    // Vertical: line the submenu's top up with the row that opened it. Measure rather
    // than read `height()`: on `map` the content isn't allocated yet, so `height()` is
    // 0 and the offset would never apply. This is a purely VERTICAL offset, so it does
    // not affect the horizontal flip and stays put across re-presents.
    let anchor_height = anchor.height();
    let (_, content_height, _, _) = content.measure(gtk::Orientation::Vertical, -1);
    let offset_y = if anchor_height > 0 && content_height > 0 {
        // GTK centres the popover on the anchor row; shift it down by half the height
        // difference so the popover's top sits at the row's top.
        (content_height - anchor_height) / 2
    } else {
        0
    };
    popover.set_offset(0, offset_y);

    // Horizontal flush, flip-safe. A submenu opens to the RIGHT of its row, but the
    // compositor FLIPS it to the LEFT near the screen's right edge; either way its card
    // should sit flush with the parent panel edge, not tuck under the row button (see
    // `SUBMENU_FLUSH_REM`).
    //
    // Achieve this by widening the anchor RECT symmetrically by the flush amount, NOT
    // with a positioner offset. A horizontal offset shifts the popup, which biases the
    // compositor's flip decision: on any re-present (e.g. scrolling the parent moves the
    // anchor) it re-solves, un-flips the popup, and the now-wrong-direction offset drags
    // it over the parent — the side then oscillates. A widened rect is stable: the popup
    // anchors flush at the rect's right edge opening right and at its left edge when
    // flipped left, the flip mirrors it automatically, and the rect never depends on the
    // resolved side, so re-presents change nothing.
    let anchor_width = anchor.width();
    if anchor_width > 0 && anchor_height > 0 {
        let flush = (SUBMENU_FLUSH_REM * REM_PX * styling_scale).round() as i32;
        popover.set_pointing_to(Some(&gdk::Rectangle::new(
            -flush,
            0,
            anchor_width + 2 * flush,
            anchor_height,
        )));
    }
}

/// Offset the root popover so its bar-facing edge sits at the bar's OUTER edge
/// plus a gap, instead of at the (short, bar-centred) tray icon button's edge.
/// GTK anchors the popover to the button, whose bar-facing edge is inside the bar
/// strip; without this the menu overlaps the bar (too close), and any later
/// re-layout lets the compositor re-constrain the overlapping popup off the bar —
/// dropping its top and shrinking it. Positioning it a fixed gap clear of the bar
/// (matching the dropdown panels) makes both the resting distance correct and the
/// re-constrain unnecessary. Recomputed on every map and on reconcile; idempotent.
fn realign_root(popover: &gtk::Popover, scale: f32) {
    let Some(anchor) = popover.parent() else {
        return;
    };
    let Some(bar) = anchor.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let Some(button) = anchor.compute_bounds(&bar) else {
        return;
    };

    let (bar_w, bar_h) = (bar.width(), bar.height());
    if bar_w <= 0 || bar_h <= 0 {
        // Not yet allocated — leave GTK's default anchor; reconcile re-runs this.
        return;
    }

    // Align to the bar SECTION's edge, not the bar window's edge. The sections are
    // inset from the window by the bar padding (`.bar-section { margin }`), and a
    // dropdown panel aligns to its `BarButton` which fills the section — so aligning
    // to the section edge matches the dropdown gap; the window edge is one bar-
    // padding too far (which made the menu sit lower than the panels). Default to
    // the window edges if no `bar-section` ancestor is found.
    let (mut sec_top, mut sec_bottom, mut sec_left, mut sec_right) = (0, bar_h, 0, bar_w);
    let mut ancestor = anchor.parent();
    while let Some(widget) = ancestor {
        if widget.has_css_class("bar-section") {
            if let Some(section) = widget.compute_bounds(&bar) {
                sec_top = section.y().round() as i32;
                sec_bottom = (section.y() + section.height()).round() as i32;
                sec_left = section.x().round() as i32;
                sec_right = (section.x() + section.width()).round() as i32;
            }
            break;
        }
        ancestor = widget.parent();
    }

    let gap = (ROOT_GAP_REM * REM_PX * scale).round() as i32;
    let btn_top = button.y().round() as i32;
    let btn_bottom = (button.y() + button.height()).round() as i32;
    let btn_left = button.x().round() as i32;
    let btn_right = (button.x() + button.width()).round() as i32;

    // Shift the popover's bar-facing edge from the (short, bar-centred) tray button
    // to the section's outer edge + gap, in the direction the menu opens (read from
    // the bar window's location CSS class, like the dropdowns' position detection).
    if bar.has_css_class("bottom") {
        popover.set_offset(0, (sec_top - btn_top) - gap); // bottom bar: opens up
    } else if bar.has_css_class("left") {
        popover.set_offset((sec_right - btn_right) + gap, 0); // left bar: opens right
    } else if bar.has_css_class("right") {
        popover.set_offset((sec_left - btn_left) - gap, 0); // right bar: opens left
    } else {
        popover.set_offset(0, (sec_bottom - btn_bottom) + gap); // top bar: opens down
    }
}

// ---------------------------------------------------------------------------
// Geometry / small builders (unchanged).
// ---------------------------------------------------------------------------

/// The monitor height (logical px) the `anchor` sits on, the bar's thin
/// dimension, and whether the bar is horizontal — everything the height caps
/// need. `None` before the anchor's surface is realized.
fn anchor_geometry(anchor: &gtk::Widget) -> Option<(i32, i32, bool)> {
    let surface = anchor.native()?.surface()?;
    let monitor_height = anchor
        .display()
        .monitor_at_surface(&surface)?
        .geometry()
        .height();
    if monitor_height <= 0 {
        return None;
    }

    let bar = anchor.root().and_downcast::<gtk::Window>()?;
    let (bar_width, bar_height) = (bar.width(), bar.height());
    // A bar is a thin strip: horizontal (top/bottom) when wider than tall. Its
    // thin dimension is the extent it occupies along the axis the menu opens on.
    let horizontal = bar_width >= bar_height;
    let bar_thickness = bar_width.min(bar_height);

    Some((monitor_height, bar_thickness, horizontal))
}

/// Max height for the *root* column: the space on the menu's side of the bar
/// (monitor height minus the bar's strip for a top/bottom bar; nearly the whole
/// monitor for a side bar), less a safety margin. Capping the ScrolledWindow to
/// this keeps its natural height within what the compositor can grant, so the
/// xdg_popup is never RESIZE/SLIDE-clamped — its top stays put and it scrolls
/// only when the content genuinely can't fit (the true maximum). The value
/// depends only on stable geometry, not the bar's transient layer state, so
/// first-open and rebuild render identically. Works for a top *or* bottom bar
/// (the space is symmetric: monitor minus the bar strip on either edge).
fn root_available_height(anchor: &gtk::Widget) -> i32 {
    let Some((monitor_height, bar_thickness, horizontal)) = anchor_geometry(anchor) else {
        return (f64::from(FALLBACK_MONITOR_HEIGHT) * MAX_HEIGHT_FRACTION) as i32;
    };

    let available = if horizontal {
        monitor_height - bar_thickness - MENU_EDGE_MARGIN
    } else {
        monitor_height - MENU_EDGE_MARGIN
    };

    available.clamp(MIN_MENU_HEIGHT, monitor_height - MENU_EDGE_MARGIN)
}

/// Max height for a *submenu* column. A submenu opens sideways from a row whose
/// on-screen position can't be read across Wayland surfaces, so it falls back to
/// a fraction of the monitor.
fn submenu_max_height(anchor: &gtk::Widget) -> i32 {
    let monitor_height =
        anchor_geometry(anchor).map_or(FALLBACK_MONITOR_HEIGHT, |(height, _, _)| height);

    (f64::from(monitor_height) * MAX_HEIGHT_FRACTION) as i32
}

fn separator() -> gtk::Separator {
    let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
    separator.add_css_class("systray-menu-divider");
    separator
}

fn apply_disposition_class(widget: &gtk::Box, disposition: Disposition) {
    match disposition {
        Disposition::Warning => widget.add_css_class("warning"),
        Disposition::Alert => widget.add_css_class("alert"),
        Disposition::Normal | Disposition::Informative => {}
    }
}

/// A left-aligned check/radio indicator. The image is empty when unchecked; CSS
/// reserves its width so labels stay aligned.
fn toggle_indicator(item: &MenuItem) -> gtk::Image {
    let indicator = gtk::Image::new();
    indicator.add_css_class("systray-menu-item-indicator");

    if item.toggle_state == ToggleState::Checked {
        let icon = match item.toggle_type {
            ToggleType::Radio => "radio-checked-symbolic",
            _ => "object-select-symbolic",
        };
        indicator.set_icon_name(Some(icon));
    }

    indicator
}

fn build_icon(item: &MenuItem) -> Option<gtk::Image> {
    if let Some(name) = item.icon_name.as_deref().filter(|name| !name.is_empty()) {
        let image = gtk::Image::from_icon_name(name);
        image.add_css_class("systray-menu-item-icon");
        return Some(image);
    }

    if let Some(data) = item.icon_data.as_deref().filter(|data| !data.is_empty()) {
        let bytes = glib::Bytes::from(data);
        if let Ok(texture) = gdk::Texture::from_bytes(&bytes) {
            let image = gtk::Image::from_paintable(Some(&texture));
            image.add_css_class("systray-menu-item-icon");
            return Some(image);
        }
    }

    None
}

/// Format a DBusMenu shortcut (`[["Control", "q"]]`) as `Control+q` for display.
fn format_shortcut(shortcut: &[Vec<String>]) -> Option<String> {
    let combo = shortcut.first()?;
    if combo.is_empty() {
        return None;
    }
    Some(combo.join("+"))
}
