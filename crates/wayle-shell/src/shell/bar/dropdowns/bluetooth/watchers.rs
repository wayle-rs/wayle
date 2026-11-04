use std::{
    mem::{Discriminant, discriminant},
    sync::Arc,
};

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_bluetooth::{
    BluetoothService,
    core::{
        adapter::{Adapter, AdapterInfo},
        device::{Device, DeviceInfo, DeviceSignal},
    },
    types::agent::PairingRequest,
};
use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_widgets::{
    watch, watch_cancellable, watch_deferred,
    watchers::{BoxedStream, each_changes, key_changes},
};
use zbus::zvariant::OwnedObjectPath;

use super::{
    BluetoothDropdown,
    helpers::{is_listed, is_mine, snapshot_of},
    messages::BluetoothDropdownCmd,
};

pub(super) fn spawn_config_watcher(
    sender: &ComponentSender<BluetoothDropdown>,
    config: &Arc<ConfigService>,
) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(BluetoothDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

pub(super) fn spawn_service_watcher(
    sender: &ComponentSender<BluetoothDropdown>,
    bluetooth: &DeferredService<BluetoothService>,
) {
    watch_deferred!(sender, bluetooth, BluetoothDropdownCmd::ServiceReady);
}

pub(super) fn spawn_bt_watchers(
    sender: &ComponentSender<BluetoothDropdown>,
    bluetooth: &Arc<BluetoothService>,
    token: CancellationToken,
) {
    let available = bluetooth.available.clone();
    let enabled = bluetooth.enabled.clone();
    let powered = bluetooth.powered.clone();
    let discovering = bluetooth.discovering.clone();
    let adapter_errors = each_changes(
        bluetooth.adapters.watch(),
        |adapter: &Arc<Adapter>| -> BoxedStream {
            Box::pin(key_changes(
                adapter.info.watch(),
                None,
                |info: &Arc<AdapterInfo>| info.last_error.clone(),
            ))
        },
    );

    watch_cancellable!(
        sender,
        token.clone(),
        [
            available.watch(),
            enabled.watch(),
            powered.watch(),
            discovering.watch(),
            adapter_errors
        ],
        |out| {
            let _ = out.send(BluetoothDropdownCmd::StateChanged);
        }
    );

    let radio_block = bluetooth.radio_block.clone();

    watch_cancellable!(sender, token.clone(), [radio_block.watch()], |out| {
        let _ = out.send(BluetoothDropdownCmd::RadioBlockChanged);
    });

    let devices = bluetooth.devices.clone();

    watch_cancellable!(sender, token.clone(), [devices.watch()], |out| {
        let _ = out.send(BluetoothDropdownCmd::DevicesChanged);
    });

    let pairing = bluetooth.pairing_request.clone();
    let mut shown_prompt = None;

    watch_cancellable!(sender, token, [pairing.watch()], |out| {
        let request = pairing.get();
        let prompt = request.as_ref().map(prompt_of);
        let new_prompt = prompt != shown_prompt;
        shown_prompt = prompt;
        let _ = out.send(BluetoothDropdownCmd::PairingRequested {
            request,
            new_prompt,
        });
    });
}

/// What a pairing request asks about, whatever details it updates (such as
/// how many digits of a passkey were typed).
fn prompt_of(request: &PairingRequest) -> (Discriminant<PairingRequest>, OwnedObjectPath) {
    (discriminant(request), request.device_path().clone())
}

/// Watches the parts of `device`'s state that affect the lists: what its row
/// shows, and, while it's an available device, the RSSI those are sorted by.
/// A device without a row (most unnamed ones) triggers no rebuilds, and
/// neither does anything its row doesn't show.
///
/// Each is compared with the value read here: the lists are rebuilt after the
/// watcher is spawned, so they reflect that value or a later one.
pub(super) fn spawn_device_watcher(
    sender: &ComponentSender<BluetoothDropdown>,
    device: &Arc<Device>,
    token: CancellationToken,
) {
    let row = {
        let device = Arc::clone(device);
        move |info: &Arc<DeviceInfo>| snapshot_of(&device, info)
    };
    let shown = row(&device.info.get());
    let row_changes = key_changes(device.info.watch(), Some(shown), row);

    let placement = {
        let device = Arc::clone(device);
        move |signal: &Arc<DeviceSignal>| {
            let info = device.info.get();
            let available = is_listed(&info) && !is_mine(&info);
            available.then_some(signal.rssi)
        }
    };
    let placed = placement(&device.signal.get());
    let rssi_changes = key_changes(device.signal.watch(), Some(placed), placement);

    watch_cancellable!(sender, token, [row_changes, rssi_changes], |out| {
        let _ = out.send(BluetoothDropdownCmd::DevicePropertyChanged);
    });
}
