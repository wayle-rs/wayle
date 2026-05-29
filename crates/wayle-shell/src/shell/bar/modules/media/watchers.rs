use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::schemas::modules::{MediaConfig, MediaIconType};
use wayle_media::{MediaService, core::player::Player};
use wayle_widgets::{watch, watch_cancellable};

use super::{MediaModule, messages::MediaCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<MediaModule>,
    config: &MediaConfig,
    media: &Arc<MediaService>,
) {
    let active_player = media.active_player.clone();
    watch!(sender, [active_player.watch()], |out| {
        let _ = out.send(MediaCmd::PlayerChanged(active_player.get()));
    });

    let format = config.format.clone();
    watch!(sender, [format.watch()], |out| {
        let _ = out.send(MediaCmd::MetadataChanged);
    });

    let icon_name = config.icon_name.clone();
    let icon_type = config.icon_type.clone();
    watch!(sender, [icon_name.watch()], |out| {
        if icon_type.get() == MediaIconType::Default {
            let _ = out.send(MediaCmd::UpdateIcon(icon_name.get().clone()));
        }
    });

    let spinning_disc_icon = config.spinning_disc_icon.clone();
    let icon_type_for_disc = config.icon_type.clone();
    watch!(sender, [spinning_disc_icon.watch()], |out| {
        if icon_type_for_disc.get() == MediaIconType::SpinningDisc {
            let _ = out.send(MediaCmd::UpdateIcon(spinning_disc_icon.get().clone()));
        }
    });

    watch!(sender, [config.icon_type.watch()], |out| {
        let _ = out.send(MediaCmd::IconTypeChanged);
    });

    watch!(sender, [config.visibility.watch()], |out| {
        let _ = out.send(MediaCmd::VisibilityChanged);
    });
}

pub(super) fn spawn_player_watchers(
    sender: &ComponentSender<MediaModule>,
    player: &Arc<Player>,
    token: CancellationToken,
) {
    let metadata = player.metadata.clone();
    let metadata_token = token.clone();
    watch_cancellable!(sender, metadata_token, [metadata.watch()], |out| {
        let _ = out.send(MediaCmd::MetadataChanged);
    });

    let playback_state = player.playback_state.clone();
    watch_cancellable!(sender, token, [playback_state.watch()], |out| {
        let _ = out.send(MediaCmd::PlaybackStateChanged);
    });
}
