//! Drag-and-drop payload encoding and drop handling.

use gtk::prelude::*;
use relm4::{
    factory::FactorySender,
    gtk,
    gtk::{gdk, glib},
};
use tracing::warn;

use super::{
    super::card::{LayoutCard, LayoutCardOutput},
    ZoneId,
};

#[derive(Debug, Clone)]
pub(crate) struct DragPayload {
    pub(crate) card_index: usize,
    pub(crate) zone: ZoneId,
    pub(crate) item_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct DropLocation {
    pub(crate) card_index: usize,
    pub(crate) zone: ZoneId,
    pub(crate) position: usize,
}

pub(super) fn attach_drag_source(
    widget: &gtk::Box,
    card_index: usize,
    zone: ZoneId,
    item_index: usize,
) {
    let drag = gtk::DragSource::new();
    drag.set_actions(gdk::DragAction::MOVE);

    let payload = DragPayload {
        card_index,
        zone,
        item_index,
    }
    .encode();

    drag.connect_prepare(move |_source, _x, _y| {
        Some(gdk::ContentProvider::for_value(&payload.to_value()))
    });

    drag.connect_drag_begin(|source, _drag| {
        if let Some(widget) = source.widget() {
            widget.add_css_class("chip-dragging");
        }
    });

    drag.connect_drag_end(|source, _drag, _delete| {
        if let Some(widget) = source.widget() {
            widget.remove_css_class("chip-dragging");
        }
    });

    widget.add_controller(drag);
}

pub(super) fn attach_drop_target(
    chips_box: &gtk::FlowBox,
    card_index: usize,
    zone: ZoneId,
    sender: &FactorySender<LayoutCard>,
) {
    let drop = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);

    let motion_box = chips_box.clone();

    drop.connect_motion(move |_target, x, y| {
        let position = compute_drop_position(&motion_box, x, y);
        clear_drop_highlight(&motion_box);
        highlight_drop_position(&motion_box, position);
        gdk::DragAction::MOVE
    });

    let leave_box = chips_box.clone();

    drop.connect_leave(move |_target| {
        clear_drop_highlight(&leave_box);
    });

    let drop_box = chips_box.clone();
    let drop_sender = sender.output_sender().clone();

    drop.connect_drop(move |_target, value, x, y| {
        clear_drop_highlight(&drop_box);

        let payload_str = match value.get::<String>() {
            Ok(payload) => payload,
            Err(err) => {
                warn!(error = %err, "drop payload was not a string");
                return false;
            }
        };

        let Some(from) = DragPayload::decode(&payload_str) else {
            warn!(payload = %payload_str, "cannot decode drop payload");
            return false;
        };

        let position = compute_drop_position(&drop_box, x, y);

        let to = DropLocation {
            card_index,
            zone,
            position,
        };

        let _ = drop_sender.send(LayoutCardOutput::ItemDropped(from, to));
        true
    });

    chips_box.add_controller(drop);
}

fn flow_children(container: &gtk::FlowBox) -> Vec<gtk::FlowBoxChild> {
    let mut children = Vec::new();
    let mut index = 0;

    while let Some(child) = container.child_at_index(index) {
        children.push(child);
        index += 1;
    }

    children
}

fn compute_drop_position(container: &gtk::FlowBox, drop_x: f64, drop_y: f64) -> usize {
    let children = flow_children(container);

    if let Some(hit) = container.child_at_pos(drop_x as i32, drop_y as i32) {
        let hit_index = hit.index() as usize;
        let Some(bounds) = hit.compute_bounds(container) else {
            return hit_index;
        };

        let center_x = f64::from(bounds.x() + bounds.width() / 2.0);
        if drop_x < center_x {
            return hit_index;
        }
        return hit_index + 1;
    }

    let mut row_last: Option<usize> = None;

    for (index, child) in children.iter().enumerate() {
        let Some(bounds) = child.compute_bounds(container) else {
            continue;
        };

        let row_top = f64::from(bounds.y());
        let row_bottom = f64::from(bounds.y() + bounds.height());

        if drop_y < row_top || drop_y > row_bottom {
            continue;
        }

        row_last = Some(index);
    }

    row_last.map_or(children.len(), |index| index + 1)
}

fn highlight_drop_position(container: &gtk::FlowBox, position: usize) {
    let children = flow_children(container);

    if let Some(target) = children.get(position) {
        target.add_css_class("drop-before");
    } else if let Some(last) = children.last() {
        last.add_css_class("drop-after");
    }
}

fn clear_drop_highlight(container: &gtk::FlowBox) {
    for child in flow_children(container) {
        child.remove_css_class("drop-before");
        child.remove_css_class("drop-after");
    }
}

impl DragPayload {
    const SEPARATOR: char = ':';

    fn encode(&self) -> String {
        format!(
            "{}{sep}{}{sep}{}",
            self.card_index,
            self.zone,
            self.item_index,
            sep = Self::SEPARATOR,
        )
    }

    fn decode(encoded: &str) -> Option<Self> {
        let mut parts = encoded.splitn(3, Self::SEPARATOR);

        let card_index = parts.next()?.parse().ok()?;
        let zone = parts.next()?.parse().ok()?;
        let item_index = parts.next()?.parse().ok()?;

        Some(Self {
            card_index,
            zone,
            item_index,
        })
    }
}
