use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_bluetooth::{
    BluetoothService,
    core::device::{Device, DeviceInfo},
};
use wayle_config::schemas::modules::BluetoothConfig;
use wayle_core::DeferredService;
use wayle_widgets::{
    watch_cancellable, watch_deferred,
    watchers::{BoxedStream, each_changes, key_changes},
};

use super::{BluetoothModule, messages::BluetoothCmd};

pub(super) fn spawn_service_watcher(
    sender: &ComponentSender<BluetoothModule>,
    bluetooth: &DeferredService<BluetoothService>,
) {
    watch_deferred!(sender, bluetooth, BluetoothCmd::ServiceReady);
}

pub(super) fn spawn_watchers(
    sender: &ComponentSender<BluetoothModule>,
    token: CancellationToken,
    config: &BluetoothConfig,
    bt: &Arc<BluetoothService>,
) {
    let available = bt.available.clone();
    let enabled = bt.enabled.clone();
    let discovering = bt.discovering.clone();
    let connected = bt.connected.clone();

    // The label shows a connected device's alias, so renaming one refreshes it
    // too (and nothing else about them does).
    let aliases = each_changes(connected.watch(), |device: &Arc<Device>| -> BoxedStream {
        Box::pin(key_changes(
            device.info.watch(),
            None,
            |info: &Arc<DeviceInfo>| info.alias.clone(),
        ))
    });

    watch_cancellable!(
        sender,
        token.clone(),
        [
            available.watch(),
            enabled.watch(),
            discovering.watch(),
            connected.watch(),
            aliases
        ],
        |out| {
            let _ = out.send(BluetoothCmd::StateChanged);
        }
    );

    let disabled_icon = config.disabled_icon.clone();
    let disconnected_icon = config.disconnected_icon.clone();
    let connected_icon = config.connected_icon.clone();
    let searching_icon = config.searching_icon.clone();

    watch_cancellable!(
        sender,
        token,
        [
            disabled_icon.watch(),
            disconnected_icon.watch(),
            connected_icon.watch(),
            searching_icon.watch()
        ],
        |out| {
            let _ = out.send(BluetoothCmd::IconConfigChanged);
        }
    );
}
