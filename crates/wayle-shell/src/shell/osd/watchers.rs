use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_audio::{
    AudioService,
    core::device::{input::InputDevice, output::OutputDevice},
    volume::types::Volume,
};
use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::ConfigService;
use wayle_core::Property;
use wayle_widgets::{watch, watch_cancellable, watch_cancellable_throttled};

use super::{
    Osd,
    messages::{OsdCmd, ToggleEvent},
    toggles,
};
use crate::services::shell_ipc::{OsdDevice, ShellIpcState};

const VOLUME_THROTTLE: Duration = Duration::from_millis(30);

pub(super) fn spawn(
    sender: &ComponentSender<Osd>,
    config: &Arc<ConfigService>,
    audio: &Option<Arc<AudioService>>,
    brightness: &Option<Arc<BrightnessService>>,
    ipc: &ShellIpcState,
) {
    spawn_config_watcher(sender, config);

    if let Some(audio) = audio {
        spawn_audio_watcher(sender, audio);
    }

    if let Some(brightness) = brightness {
        spawn_brightness_service_watcher(sender, brightness);
    }

    spawn_toggle_watchers(sender);
    spawn_ipc_watcher(sender, ipc);
}

/// Watches for CLI-initiated show requests. The component drops replays by
/// sequence, so this forwards every emission.
fn spawn_ipc_watcher(sender: &ComponentSender<Osd>, ipc: &ShellIpcState) {
    let osd_request = ipc.osd_request.clone();

    watch!(sender, [osd_request.watch()], |out| {
        let _ = out.send(OsdCmd::ShowRequested(osd_request.get()));
    });
}

fn spawn_config_watcher(sender: &ComponentSender<Osd>, config: &Arc<ConfigService>) {
    let full_config = config.config();
    let osd = &full_config.osd;

    let position = osd.position.clone();
    let duration = osd.duration.clone();
    let monitor = osd.monitor.clone();
    let margin = osd.margin.clone();
    let border = osd.border.clone();
    let layer = osd.layer.clone();
    let scale = full_config.styling.scale.clone();
    let tearing_mode = full_config.general.tearing_mode.clone();

    watch!(
        sender,
        [
            position.watch(),
            duration.watch(),
            monitor.watch(),
            margin.watch(),
            border.watch(),
            layer.watch(),
            scale.watch(),
            tearing_mode.watch(),
        ],
        |out| {
            let _ = out.send(OsdCmd::ConfigChanged);
        }
    );
}

fn spawn_audio_watcher(sender: &ComponentSender<Osd>, audio: &Arc<AudioService>) {
    let default_output = audio.default_output.clone();

    watch!(sender, [default_output.watch()], |out| {
        let _ = out.send(OsdCmd::DeviceChanged(default_output.get()));
    });

    let default_input = audio.default_input.clone();

    watch!(sender, [default_input.watch()], |out| {
        let _ = out.send(OsdCmd::InputDeviceChanged(default_input.get()));
    });
}

pub(super) fn spawn_device_watchers(
    sender: &ComponentSender<Osd>,
    device: &Arc<OutputDevice>,
    token: CancellationToken,
) {
    let volume = device.volume.clone();
    let muted = device.muted.clone();

    watch_cancellable_throttled!(
        sender,
        token,
        VOLUME_THROTTLE,
        [volume.watch(), muted.watch()],
        |out| {
            let _ = out.send(OsdCmd::VolumeChanged);
        }
    );
}

pub(super) fn spawn_input_device_watchers(
    sender: &ComponentSender<Osd>,
    device: &Arc<InputDevice>,
    token: CancellationToken,
) {
    let volume = device.volume.clone();
    let muted = device.muted.clone();

    watch_cancellable_throttled!(
        sender,
        token,
        VOLUME_THROTTLE,
        [volume.watch(), muted.watch()],
        |out| {
            let _ = out.send(OsdCmd::InputVolumeChanged);
        }
    );
}

fn spawn_brightness_service_watcher(
    sender: &ComponentSender<Osd>,
    brightness: &Arc<BrightnessService>,
) {
    let primary = brightness.primary.clone();

    watch!(sender, [primary.watch()], |out| {
        let _ = out.send(OsdCmd::BrightnessDeviceChanged(primary.get()));
    });
}

fn spawn_toggle_watchers(sender: &ComponentSender<Osd>) {
    let keyboards = toggles::find_keyboards();

    for mut stream in keyboards {
        sender.command(move |out, shutdown| async move {
            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            loop {
                tokio::select! {
                    _ = &mut shutdown_fut => return,

                    event = stream.next() => {
                        let Some(result) = event else { return };

                        let Ok(event) = result else { return };

                        let toggle_key = toggles::detect_toggle(
                            event.event_type(),
                            event.code(),
                            event.value(),
                        );

                        let Some(key) = toggle_key else {
                            continue;
                        };

                        tokio::time::sleep(toggles::LED_DELAY).await;

                        let active = toggles::read_led_state(&stream, key);

                        let toggle = ToggleEvent { key, active };
                        let _ = out.send(OsdCmd::ToggleChanged(toggle));
                    }
                }
            }
        });
    }
}

pub(super) fn spawn_brightness_watcher(
    sender: &ComponentSender<Osd>,
    device: &Arc<BacklightDevice>,
    token: CancellationToken,
) {
    let brightness = device.brightness.clone();

    watch_cancellable!(sender, token, [brightness.watch()], |out| {
        let _ = out.send(OsdCmd::BrightnessChanged);
    });
}

/// Watches the device currently on screen so its reading stays live.
///
/// The watchers above follow the *default* device to decide when to open the
/// OSD; this one follows whatever is displayed, including a device named on
/// the command line, and only refreshes the content.
pub(super) fn spawn_displayed_watcher(
    sender: &ComponentSender<Osd>,
    device: &OsdDevice,
    token: CancellationToken,
) {
    match device {
        OsdDevice::Speaker(device) => {
            spawn_volume_refresh(sender, &device.volume, &device.muted, token);
        }

        OsdDevice::Microphone(device) => {
            spawn_volume_refresh(sender, &device.volume, &device.muted, token);
        }

        OsdDevice::Brightness(device) => {
            let brightness = device.brightness.clone();

            watch_cancellable!(sender, token, [brightness.watch()], |out| {
                let _ = out.send(OsdCmd::DisplayedValueChanged);
            });
        }
    }
}

fn spawn_volume_refresh(
    sender: &ComponentSender<Osd>,
    volume: &Property<Volume>,
    muted: &Property<bool>,
    token: CancellationToken,
) {
    let volume = volume.clone();
    let muted = muted.clone();

    watch_cancellable_throttled!(
        sender,
        token,
        VOLUME_THROTTLE,
        [volume.watch(), muted.watch()],
        |out| {
            let _ = out.send(OsdCmd::DisplayedValueChanged);
        }
    );
}
