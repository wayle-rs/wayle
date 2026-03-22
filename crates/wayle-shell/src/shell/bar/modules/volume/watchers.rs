use std::{sync::Arc, time::Duration};

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_audio::{AudioService, core::device::output::OutputDevice};
use wayle_common::{watch, watch_cancellable_throttled};
use wayle_config::schemas::{modules::VolumeConfig, styling::evaluate_thresholds};

const VOLUME_THROTTLE: Duration = Duration::from_millis(30);

use super::{VolumeModule, messages::VolumeCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<VolumeModule>,
    config: &VolumeConfig,
    audio: &Arc<AudioService>,
) {
    let default_output = audio.default_output.clone();
    watch!(sender, [default_output.watch()], |out| {
        let _ = out.send(VolumeCmd::DeviceChanged(default_output.get()));
    });

    let level_icons = config.level_icons.clone();
    let muted_icon = config.icon_muted.clone();
    watch!(sender, [level_icons.watch(), muted_icon.watch()], |out| {
        let _ = out.send(VolumeCmd::IconConfigChanged);
    });

    let thresholds = config.thresholds.clone();
    let audio_thresholds = audio.default_output.clone();
    watch!(sender, [thresholds.watch()], |out| {
        if let Some(device) = audio_thresholds.get() {
            let percentage = device.volume.get().average_percentage().round() as u16;
            let colors = evaluate_thresholds(percentage as f64, &thresholds.get());
            let _ = out.send(VolumeCmd::UpdateThresholdColors(colors));
        }
    });
}

pub(super) fn spawn_device_watchers(
    sender: &ComponentSender<VolumeModule>,
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
            let _ = out.send(VolumeCmd::VolumeOrMuteChanged);
        }
    );
}
