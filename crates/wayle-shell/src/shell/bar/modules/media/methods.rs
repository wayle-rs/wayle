use relm4::{gtk, gtk::prelude::*};
use wayle_config::schemas::modules::{MediaConfig, MediaVisibility};
use wayle_media::{MediaService, types::PlaybackState};

use super::MediaModule;

impl MediaModule {
    /// Determines if the module should be visible based on config and playback state.
    pub(super) fn update_visibility(config: &MediaConfig, media: &MediaService) -> bool {
        match config.visibility.get() {
            MediaVisibility::Always => true,
            MediaVisibility::Playing => {
                // Only show if there is an active player that is currently playing.
                if let Some(player) = media.active_player() {
                    player.playback_state.get() == PlaybackState::Playing
                } else {
                    false
                }
            }
            MediaVisibility::Active => media.active_player().is_some(),
        }
    }

    pub(super) fn update_disc_mode(root: &gtk::Box, enabled: bool) {
        if enabled {
            root.add_css_class("media-disc");
        } else {
            root.remove_css_class("media-disc");
        }
    }

    pub(super) fn update_spinning_state(root: &gtk::Box, state: PlaybackState) {
        match state {
            PlaybackState::Playing => {
                root.add_css_class("media-spinning");
            }
            PlaybackState::Paused | PlaybackState::Stopped => {
                root.remove_css_class("media-spinning");
            }
        }
    }
}
