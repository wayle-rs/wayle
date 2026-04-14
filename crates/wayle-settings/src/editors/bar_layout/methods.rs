//! Behavior methods for `BarLayoutControl`: add/remove card, commit to
//! config, and cross-card drag-drop handling.

use relm4::prelude::*;
use wayle_config::schemas::bar::BarLayout;

use super::{BarLayoutControl, DragPayload, DropLocation, LayoutCardInit};

impl BarLayoutControl {
    pub(super) fn on_add(&mut self) {
        self.cards.guard().push_back(LayoutCardInit {
            layout: BarLayout::default(),
            custom_modules: self.custom_modules.clone(),
        });
        self.commit();
    }

    pub(super) fn on_remove(&mut self, index: DynamicIndex) {
        self.cards.guard().remove(index.current_index());
        self.commit();
    }

    pub(super) fn on_item_dropped(&mut self, from: DragPayload, to: DropLocation) {
        self.handle_drop(from, to);
        self.commit();
        self.rebuild_cards();
    }

    pub(super) fn on_refresh(&mut self) {
        let incoming = self.property.get();
        let current: Vec<BarLayout> = self.cards.iter().map(|card| card.to_layout()).collect();

        if incoming == current {
            return;
        }

        self.rebuild_cards();
    }

    pub(super) fn commit(&self) {
        let layouts: Vec<BarLayout> = self.cards.iter().map(|card| card.to_layout()).collect();
        self.property.set(layouts);
    }

    fn handle_drop(&mut self, from: DragPayload, to: DropLocation) {
        let mut guard = self.cards.guard();
        let same_card = from.card_index == to.card_index;

        if guard.get(from.card_index).is_none() {
            return;
        }

        if !same_card && guard.get(to.card_index).is_none() {
            return;
        }

        let source_card = &mut guard[from.card_index];
        let source_zone = source_card.zone_mut(from.zone);

        if from.item_index >= source_zone.len() {
            return;
        }

        let item = source_zone.remove(from.item_index);

        if !same_card {
            let target_card = &mut guard[to.card_index];
            let target_zone = target_card.zone_mut(to.zone);
            let position = to.position.min(target_zone.len());
            target_zone.insert(position, item);
            return;
        }

        let same_zone = from.zone == to.zone;
        let mut target_pos = to.position;

        if same_zone && from.item_index < target_pos {
            target_pos = target_pos.saturating_sub(1);
        }

        let target_zone = source_card.zone_mut(to.zone);
        let position = target_pos.min(target_zone.len());
        target_zone.insert(position, item);
    }

    fn rebuild_cards(&mut self) {
        let layouts = self.property.get();
        let mut guard = self.cards.guard();
        guard.clear();

        for layout in layouts {
            guard.push_back(LayoutCardInit {
                layout,
                custom_modules: self.custom_modules.clone(),
            });
        }
    }
}
