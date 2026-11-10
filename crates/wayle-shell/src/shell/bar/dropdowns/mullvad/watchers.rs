use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_mullvad::MullvadService;
use wayle_widgets::{watch, watch_cancellable, watch_deferred};

use super::{MullvadDropdown, messages::MullvadDropdownCmd};

pub(super) fn spawn_config_watcher(
    sender: &ComponentSender<MullvadDropdown>,
    config: &Arc<ConfigService>,
) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(MullvadDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

pub(super) fn spawn_service_watcher(
    sender: &ComponentSender<MullvadDropdown>,
    mullvad: &DeferredService<MullvadService>,
) {
    watch_deferred!(sender, mullvad, MullvadDropdownCmd::ServiceReady);
}

pub(super) fn spawn_state_watchers(
    sender: &ComponentSender<MullvadDropdown>,
    token: CancellationToken,
    mullvad: &Arc<MullvadService>,
) {
    let networks = mullvad.mullvad.networks.clone();

    watch_cancellable!(sender, token.clone(), [networks.watch()], |out| {
        let _ = out.send(MullvadDropdownCmd::RelaysChanged);
    });

    let status = mullvad.mullvad.status.clone();
    let selected = mullvad.mullvad.selected.clone();

    watch_cancellable!(sender, token, [status.watch(), selected.watch()], |out| {
        let _ = out.send(MullvadDropdownCmd::TunnelChanged);
    });
}
