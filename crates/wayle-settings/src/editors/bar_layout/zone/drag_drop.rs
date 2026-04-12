//! Drag-and-drop payload encoding and drop handling.

use gtk::prelude::*;
use relm4::{
    factory::FactorySender,
    gtk,
    gtk::{gdk, glib},
};

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
    chips_box: &gtk::Box,
    card_index: usize,
    zone: ZoneId,
    sender: &FactorySender<LayoutCard>,
) {
    let drop = gtk::DropTarget::new(glib::Type::STRING, gdk::DragAction::MOVE);

    let motion_box = chips_box.clone();

    drop.connect_motion(move |_target, x, _y| {
        let position = compute_drop_position(&motion_box, x);
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

    drop.connect_drop(move |_target, value, x, _y| {
        clear_drop_highlight(&drop_box);

        let Ok(payload_str) = value.get::<String>() else {
            return false;
        };

        let Some(from) = DragPayload::decode(&payload_str) else {
            return false;
        };

        let position = compute_drop_position(&drop_box, x);

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

fn compute_drop_position(container: &gtk::Box, drop_x: f64) -> usize {
    let mut position = 0;
    let mut child = container.first_child();

    while let Some(widget) = child {
        let Some(bounds) = widget.compute_bounds(container) else {
            child = widget.next_sibling();
            continue;
        };

        let center_x = f64::from(bounds.x() + bounds.width() / 2.0);

        if drop_x < center_x {
            return position;
        }

        position += 1;
        child = widget.next_sibling();
    }

    position
}

fn highlight_drop_position(container: &gtk::Box, position: usize) {
    let mut index = 0;
    let mut child = container.first_child();
    let mut last_child: Option<gtk::Widget> = None;

    while let Some(widget) = child {
        if index == position {
            widget.add_css_class("drop-before");
        }

        last_child = Some(widget.clone());
        index += 1;
        child = widget.next_sibling();
    }

    if position >= index
        && let Some(last) = last_child
    {
        last.add_css_class("drop-after");
    }
}

fn clear_drop_highlight(container: &gtk::Box) {
    let mut child = container.first_child();

    while let Some(widget) = child {
        widget.remove_css_class("drop-before");
        widget.remove_css_class("drop-after");
        child = widget.next_sibling();
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
