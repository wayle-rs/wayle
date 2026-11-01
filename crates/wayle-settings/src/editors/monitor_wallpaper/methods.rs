//! Behavior methods for `MonitorWallpaperControl`: add/remove cards,
//! commit to config, and refresh from external updates.

use relm4::prelude::*;
use wayle_config::schemas::wallpaper::{FitMode, MonitorWallpaperConfig};

use super::MonitorWallpaperControl;

impl MonitorWallpaperControl {
    pub(super) fn on_add(&mut self) {
        let new_config = MonitorWallpaperConfig {
            name: String::new(),
            fit_mode: FitMode::Fill,
            wallpaper: String::new(),
        };

        self.cards.guard().push_back(new_config);
        self.commit();
    }

    pub(super) fn on_remove(&mut self, index: DynamicIndex) {
        self.cards.guard().remove(index.current_index());
        self.commit();
    }

    pub(super) fn on_refresh(&mut self) {
        let incoming = self.property.get();
        let current: Vec<MonitorWallpaperConfig> =
            self.cards.iter().map(|card| card.to_config()).collect();

        if incoming == current {
            return;
        }

        let mut guard = self.cards.guard();
        guard.clear();

        for config in incoming {
            guard.push_back(config);
        }
    }

    pub(super) fn commit(&self) {
        let configs: Vec<MonitorWallpaperConfig> =
            self.cards.iter().map(|card| card.to_config()).collect();

        self.property.set(configs);
    }
}
