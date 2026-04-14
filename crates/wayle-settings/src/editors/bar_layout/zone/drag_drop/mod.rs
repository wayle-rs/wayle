//! Drag-and-drop payload encoding and drop handling.

mod helpers;

use gtk::prelude::*;
use relm4::{
    factory::FactorySender,
    gtk,
    gtk::{gdk, glib},
    prelude::DynamicIndex,
};
use tracing::warn;

use self::helpers::{clear_drop_highlight, compute_drop_position, highlight_drop_position};
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

pub(crate) fn attach_drag_source(
    widget: &gtk::Box,
    card_index: DynamicIndex,
    zone: ZoneId,
    self_index: DynamicIndex,
) {
    let drag = gtk::DragSource::new();
    drag.set_actions(gdk::DragAction::MOVE);

    drag.connect_prepare(move |_source, _x, _y| {
        let payload = DragPayload {
            card_index: card_index.current_index(),
            zone,
            item_index: self_index.current_index(),
        }
        .encode();
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

pub(crate) fn attach_drop_target(
    chips_box: &gtk::FlowBox,
    card_index: DynamicIndex,
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
            card_index: card_index.current_index(),
            zone,
            position,
        };

        let _ = drop_sender.send(LayoutCardOutput::ItemDropped(from, to));
        true
    });

    chips_box.add_controller(drop);
}
