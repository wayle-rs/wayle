use std::{
    panic::{self, AssertUnwindSafe},
    sync::Arc,
};

use libpulse_binding::{
    callbacks::ListResult,
    context::{Context, subscribe::Facility},
    volume::ChannelVolumes,
};
use tracing::{debug, error};

use crate::{
    backend::{
        conversion::device::{from_sink, from_source},
        types::{DeviceStore, EventSender},
    },
    events::AudioEvent,
    types::device::{Device, DeviceKey, DeviceType},
};

/// 2026-09-07 (local vendor patch, see root Cargo.toml `[patch.crates-io]`):
/// every `introspect.*` call in this file allocates a libpulse-binding
/// `Operation` from a raw `pa_operation*` C pointer, and
/// `Operation::from_raw` does `assert!(!ptr.is_null())`. PulseAudio returns
/// NULL from those C functions (instead of erroring asynchronously) when
/// the call can't even be started -- e.g. context not `PA_CONTEXT_READY`,
/// or (for volume calls) an invalid `ChannelVolumes` for the target sink's
/// current channel count. That assert then panics the tokio worker thread
/// running this command, which has no supervisor: the panic isn't logged
/// anywhere but stderr (bypasses `tracing`, so it's invisible in Wayle's
/// own log files), and the audio backend is left broken for the rest of
/// the session -- this is the confirmed cause of a recurring bug where
/// Wayle's volume controls (including scroll-to-adjust on the media
/// player) silently stop working until Wayle is relaunched. Seen 4 times
/// since 2026-08-31 in production, always this exact panic.
///
/// `run_guarded` isolates each such call: it logs the call's context right
/// before attempting it, and if it panics, catches the unwind and logs the
/// full context plus the panic payload as an ERROR instead of taking down
/// the worker thread. This doesn't fix the underlying PulseAudio-level
/// race (still unknown -- that requires reproducing with this logging in
/// place), but it turns a silent, session-wide, worker-killing crash into
/// a single logged failure that this one call recovers from.
///
/// Safe to `AssertUnwindSafe`: the panic originates in pure Rust
/// (`Operation::from_raw`'s assert), strictly after the FFI call into
/// PulseAudio has already returned -- no unwind crosses an FFI boundary,
/// and the only captured state (`&mut Introspector`, a thin owned wrapper
/// around a duplicated context reference) is a local we don't touch again
/// after this call regardless of outcome.
fn run_guarded(
    op_name: &str,
    context: impl FnOnce() -> String,
    f: impl FnOnce() + panic::UnwindSafe,
) {
    if let Err(panic_payload) = panic::catch_unwind(f) {
        let panic_msg = panic_payload
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| panic_payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| String::from("<non-string panic payload>"));

        error!(
            operation = op_name,
            context = %context(),
            panic = %panic_msg,
            "PulseAudio introspect call panicked (libpulse-binding NULL pa_operation) -- \
             recovered, audio worker kept alive"
        );
    }
}

pub(crate) fn trigger_discovery(context: &Context, devices: &DeviceStore, events_tx: &EventSender) {
    let introspect = context.introspect();

    let devices_clone = Arc::clone(devices);
    let events_tx_clone = events_tx.clone();

    run_guarded(
        "get_sink_info_list",
        || String::from("(no device_key -- full sink list)"),
        AssertUnwindSafe(|| {
            introspect.get_sink_info_list(move |sink_list| {
                if let ListResult::Item(sink) = sink_list {
                    let sink_info = from_sink(sink);
                    let device_key = sink_info.key();
                    let device_data = Device::Sink(sink_info);

                    process_device_update(device_key, device_data, &devices_clone, &events_tx_clone);
                }
            });
        }),
    );

    let devices_clone = Arc::clone(devices);
    let events_tx_clone = events_tx.clone();

    run_guarded(
        "get_source_info_list",
        || String::from("(no device_key -- full source list)"),
        AssertUnwindSafe(|| {
            introspect.get_source_info_list(move |source_list| {
                if let ListResult::Item(source) = source_list {
                    let source_info = from_source(source);
                    let device_key = source_info.key();
                    let device_data = Device::Source(source_info);

                    process_device_update(device_key, device_data, &devices_clone, &events_tx_clone);
                }
            });
        }),
    );
}

pub(crate) fn trigger_refresh(
    context: &Context,
    devices: &DeviceStore,
    events_tx: &EventSender,
    device_key: DeviceKey,
    facility: Facility,
) {
    let introspect = context.introspect();

    let devices_clone = Arc::clone(devices);
    let events_tx_clone = events_tx.clone();

    match facility {
        Facility::Sink => {
            run_guarded(
                "get_sink_info_by_index",
                || format!("device_key={device_key:?}"),
                AssertUnwindSafe(|| {
                    introspect.get_sink_info_by_index(device_key.index, move |sink_list| {
                        if let ListResult::Item(sink) = sink_list {
                            let sink_info = from_sink(sink);
                            let device_data = Device::Sink(sink_info);

                            process_device_update(
                                device_key,
                                device_data,
                                &devices_clone,
                                &events_tx_clone,
                            );
                        }
                    });
                }),
            );
        }
        Facility::Source => {
            run_guarded(
                "get_source_info_by_index",
                || format!("device_key={device_key:?}"),
                AssertUnwindSafe(|| {
                    introspect.get_source_info_by_index(device_key.index, move |source_list| {
                        if let ListResult::Item(source) = source_list {
                            let source_info = from_source(source);
                            let device_data = Device::Source(source_info);

                            process_device_update(
                                device_key,
                                device_data,
                                &devices_clone,
                                &events_tx_clone,
                            );
                        }
                    });
                }),
            );
        }
        _ => {}
    }
}

pub(crate) fn process_device_update(
    device_key: DeviceKey,
    device_data: Device,
    devices: &DeviceStore,
    events_tx: &EventSender,
) {
    let Ok(mut devices_guard) = devices.write() else {
        return;
    };

    let is_new = !devices_guard.contains_key(&device_key);
    devices_guard.insert(device_key, device_data.clone());

    let event = if is_new {
        AudioEvent::DeviceAdded(device_data)
    } else {
        AudioEvent::DeviceChanged(device_data)
    };

    let _ = events_tx.send(event);
}

pub(crate) fn set_device_volume(
    context: &Context,
    device_key: DeviceKey,
    volume: ChannelVolumes,
    devices: &DeviceStore,
) {
    let devices_clone = Arc::clone(devices);
    let mut introspect = context.introspect();

    let device_info = {
        if let Ok(devices_guard) = devices_clone.read() {
            devices_guard
                .values()
                .find(|d| d.key() == device_key)
                .cloned()
        } else {
            return;
        }
    };

    let Some(info) = device_info else {
        debug!(
            device_key = ?device_key,
            "set_device_volume: device_key not found in local store, skipping (stale scroll/UI event?)"
        );
        return;
    };

    debug!(
        device_key = ?device_key,
        device_type = ?info.key().device_type,
        volume_channels = volume.len(),
        volume_valid = volume.is_valid(),
        "set_device_volume: issuing PulseAudio volume change"
    );

    match info.key().device_type {
        DeviceType::Output => {
            run_guarded(
                "set_sink_volume_by_index",
                || {
                    format!(
                        "device_key={device_key:?} volume_channels={} volume_valid={}",
                        volume.len(),
                        volume.is_valid()
                    )
                },
                AssertUnwindSafe(|| {
                    introspect.set_sink_volume_by_index(device_key.index, &volume, None);
                }),
            );
        }
        DeviceType::Input => {
            run_guarded(
                "set_source_volume_by_index",
                || {
                    format!(
                        "device_key={device_key:?} volume_channels={} volume_valid={}",
                        volume.len(),
                        volume.is_valid()
                    )
                },
                AssertUnwindSafe(|| {
                    introspect.set_source_volume_by_index(device_key.index, &volume, None);
                }),
            );
        }
    }
}

pub(crate) fn set_device_mute(
    context: &Context,
    device_key: DeviceKey,
    muted: bool,
    devices: &DeviceStore,
) {
    let devices_clone = Arc::clone(devices);
    let mut introspect = context.introspect();

    let device_info = {
        if let Ok(devices_guard) = devices_clone.read() {
            devices_guard
                .values()
                .find(|d| d.key() == device_key)
                .cloned()
        } else {
            return;
        }
    };

    if let Some(info) = device_info {
        match info.key().device_type {
            DeviceType::Output => {
                run_guarded(
                    "set_sink_mute_by_index",
                    || format!("device_key={device_key:?} muted={muted}"),
                    AssertUnwindSafe(|| {
                        introspect.set_sink_mute_by_index(device_key.index, muted, None);
                    }),
                );
            }
            DeviceType::Input => {
                run_guarded(
                    "set_source_mute_by_index",
                    || format!("device_key={device_key:?} muted={muted}"),
                    AssertUnwindSafe(|| {
                        introspect.set_source_mute_by_index(device_key.index, muted, None);
                    }),
                );
            }
        }
    }
}

pub(crate) fn set_device_port(
    context: &Context,
    device_key: DeviceKey,
    port: String,
    devices: &DeviceStore,
) {
    let devices_clone = Arc::clone(devices);
    let mut introspect = context.introspect();

    let device_info = {
        if let Ok(devices_guard) = devices_clone.read() {
            devices_guard
                .values()
                .find(|d| d.key() == device_key)
                .cloned()
        } else {
            return;
        }
    };

    if let Some(info) = device_info {
        match info.key().device_type {
            DeviceType::Output => {
                run_guarded(
                    "set_sink_port_by_index",
                    || format!("device_key={device_key:?} port={port}"),
                    AssertUnwindSafe(|| {
                        introspect.set_sink_port_by_index(device_key.index, &port, None);
                    }),
                );
            }
            DeviceType::Input => {
                run_guarded(
                    "set_source_port_by_index",
                    || format!("device_key={device_key:?} port={port}"),
                    AssertUnwindSafe(|| {
                        introspect.set_source_port_by_index(device_key.index, &port, None);
                    }),
                );
            }
        }
    }
}
