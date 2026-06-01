use std::{sync::Arc, time::Duration};

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::ConfigService;
use wayle_widgets::{watch, watch_cancellable_throttled};

use super::{BrightnessDropdown, messages::BrightnessDropdownCmd};

const BRIGHTNESS_THROTTLE: Duration = Duration::from_millis(30);

pub(super) fn spawn_top_level(
    sender: &ComponentSender<BrightnessDropdown>,
    brightness: &Arc<BrightnessService>,
    config: &Arc<ConfigService>,
) {
    let devices = brightness.devices.clone();
    watch!(sender, [devices.watch()], |out| {
        let _ = out.send(BrightnessDropdownCmd::DevicesChanged(devices.get()));
    });

    let scale = config.config().styling.scale.clone();
    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(BrightnessDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

pub(super) fn spawn_per_device(
    sender: &ComponentSender<BrightnessDropdown>,
    devices: &[Arc<BacklightDevice>],
    token: CancellationToken,
) {
    for device in devices {
        let device_name = device.name.to_string();
        let brightness = device.brightness.clone();
        watch_cancellable_throttled!(
            sender,
            token.clone(),
            BRIGHTNESS_THROTTLE,
            [brightness.watch()],
            |out| {
                let _ = out.send(BrightnessDropdownCmd::DeviceBrightnessUpdated(
                    device_name.clone(),
                ));
            }
        );
    }
}
