use std::{sync::Arc, time::Duration};

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::schemas::{modules::BrightnessConfig, styling::evaluate_thresholds};
use wayle_widgets::{watch, watch_cancellable_throttled};

use super::{BrightnessModule, messages::BrightnessCmd};

const BRIGHTNESS_THROTTLE: Duration = Duration::from_millis(30);

pub(super) fn spawn_watchers(
    sender: &ComponentSender<BrightnessModule>,
    config: &BrightnessConfig,
    brightness: &Arc<BrightnessService>,
) {
    let primary = brightness.primary.clone();
    watch!(sender, [primary.watch()], |out| {
        let _ = out.send(BrightnessCmd::DeviceChanged(primary.get()));
    });

    let level_icons = config.level_icons.clone();
    let format = config.format.clone();
    watch!(sender, [level_icons.watch(), format.watch()], |out| {
        let _ = out.send(BrightnessCmd::ConfigChanged);
    });

    let thresholds = config.thresholds.clone();
    let primary_thresholds = brightness.primary.clone();
    watch!(sender, [thresholds.watch()], |out| {
        if let Some(device) = primary_thresholds.get() {
            let percentage = device.percentage().value();
            let colors = evaluate_thresholds(percentage, &thresholds.get());
            let _ = out.send(BrightnessCmd::UpdateThresholdColors(colors));
        }
    });
}

pub(super) fn spawn_device_watchers(
    sender: &ComponentSender<BrightnessModule>,
    device: &Arc<BacklightDevice>,
    token: CancellationToken,
) {
    let brightness = device.brightness.clone();
    watch_cancellable_throttled!(
        sender,
        token,
        BRIGHTNESS_THROTTLE,
        [brightness.watch()],
        |out| {
            let _ = out.send(BrightnessCmd::BrightnessChanged);
        }
    );
}
