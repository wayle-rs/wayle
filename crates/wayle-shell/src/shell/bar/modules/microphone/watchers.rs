use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_audio::{AudioService, core::device::input::InputDevice};
use wayle_common::{watch, watch_cancellable};
use wayle_config::schemas::{modules::MicrophoneConfig, styling::evaluate_thresholds};

use super::{MicrophoneModule, messages::MicrophoneCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<MicrophoneModule>,
    config: &MicrophoneConfig,
    audio: &Arc<AudioService>,
) {
    let default_input = audio.default_input.clone();
    watch!(sender, [default_input.watch()], |out| {
        let _ = out.send(MicrophoneCmd::DeviceChanged(default_input.get()));
    });

    let icon_active = config.icon_active.clone();
    let icon_muted = config.icon_muted.clone();
    watch!(sender, [icon_active.watch(), icon_muted.watch()], |out| {
        let _ = out.send(MicrophoneCmd::IconConfigChanged);
    });
}

pub(super) fn spawn_device_watchers(
    sender: &ComponentSender<MicrophoneModule>,
    config: &MicrophoneConfig,
    device: &Arc<InputDevice>,
    token: CancellationToken,
) {
    let volume = device.volume.clone();
    let muted = device.muted.clone();
    let thresholds = config.thresholds.clone();
    let threshold_volume = device.volume.clone();
    watch_cancellable!(sender, token, [volume.watch(), muted.watch()], |out| {
        let _ = out.send(MicrophoneCmd::VolumeOrMuteChanged);

        let percentage = threshold_volume.get().average_percentage();
        let colors = evaluate_thresholds(percentage, &thresholds.get());
        let _ = out.send(MicrophoneCmd::UpdateThresholdColors(colors));
    });
}
