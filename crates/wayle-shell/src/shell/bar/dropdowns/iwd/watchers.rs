use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_config::ConfigService;
use wayle_iwd::IwdService;
use wayle_widgets::{watch, watch_cancellable};

use super::{IwdDropdown, messages::IwdDropdownCmd};

pub(super) fn spawn(
    sender: &ComponentSender<IwdDropdown>,
    config: &Arc<ConfigService>,
    iwd: &Arc<IwdService>,
) {
    let scale = config.config().styling.scale.clone();
    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(IwdDropdownCmd::ScaleChanged(scale.get().value()));
    });

    let station = iwd.station.clone();
    watch!(sender, [station.watch()], |out| {
        let _ = out.send(IwdDropdownCmd::StationDeviceChanged);
    });
}

pub(super) fn spawn_station_watchers(
    sender: &ComponentSender<IwdDropdown>,
    iwd: &Arc<IwdService>,
    token: CancellationToken,
) {
    let Some(station) = iwd.station.get() else {
        return;
    };

    let powered = station.powered.clone();
    let scanning = station.scanning.clone();
    watch_cancellable!(sender, token, [powered.watch(), scanning.watch()], |out| {
        let _ = out.send(IwdDropdownCmd::StationFlags {
            powered: powered.get(),
            scanning: scanning.get(),
        });
    });
}
