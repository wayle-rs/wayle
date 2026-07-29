use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::schemas::modules::MullvadConfig;
use wayle_core::DeferredService;
use wayle_mullvad::MullvadService;
use wayle_widgets::{watch, watch_cancellable, watch_deferred};

use super::{MullvadModule, messages::MullvadCmd};

pub(super) fn spawn_service_watcher(
    sender: &ComponentSender<MullvadModule>,
    mullvad: &DeferredService<MullvadService>,
) {
    watch_deferred!(sender, mullvad, MullvadCmd::ServiceReady);
}

/// Persistent watcher for the icon-name config properties. Armed at init (not
/// tied to the service) so icon-name config changes redraw the button even
/// while the daemon is unavailable.
pub(super) fn spawn_icon_config_watcher(
    sender: &ComponentSender<MullvadModule>,
    config: &MullvadConfig,
) {
    let connected_icon = config.connected_icon.clone();
    let connecting_icon = config.connecting_icon.clone();
    let disconnected_icon = config.disconnected_icon.clone();
    let blocked_icon = config.blocked_icon.clone();
    let disabled_icon = config.disabled_icon.clone();

    watch!(
        sender,
        [
            connected_icon.watch(),
            connecting_icon.watch(),
            disconnected_icon.watch(),
            blocked_icon.watch(),
            disabled_icon.watch()
        ],
        |out| {
            let _ = out.send(MullvadCmd::IconConfigChanged);
        }
    );
}

/// Service-scoped watcher for the reactive VPN state. Re-armed (with a fresh
/// cancellation token) each time the service becomes ready.
pub(super) fn spawn_state_watchers(
    sender: &ComponentSender<MullvadModule>,
    token: CancellationToken,
    mullvad: &Arc<MullvadService>,
) {
    let status = mullvad.mullvad.status.clone();

    watch_cancellable!(sender, token, [status.watch()], |out| {
        let _ = out.send(MullvadCmd::StateChanged);
    });
}
