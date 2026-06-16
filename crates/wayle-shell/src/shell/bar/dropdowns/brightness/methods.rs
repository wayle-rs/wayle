use std::sync::Arc;

use relm4::ComponentSender;
use tracing::warn;
use wayle_brightness::{BacklightDevice, Percentage};

use super::{
    BrightnessDropdown,
    device_item::messages::{BrightnessDeviceInit, BrightnessDeviceItemMsg},
    helpers::device_subtitle,
};

const DEVICE_ICON: &str = "ld-sun-symbolic";

impl BrightnessDropdown {
    pub(super) fn sync_devices(&mut self) {
        let multi = self.devices.len() > 1;

        let mut guard = self.device_items.guard();
        guard.clear();

        for device in &self.devices {
            guard.push_back(BrightnessDeviceInit {
                name: device.name.to_string(),
                subtitle: device_subtitle(device.name.as_str(), device.backlight_type, multi),
                icon: DEVICE_ICON,
                percentage: device.percentage().value(),
            });
        }
    }

    pub(super) fn sync_single_device(&mut self, device_name: &str) {
        let item_index = {
            let guard = self.device_items.guard();
            guard
                .iter()
                .position(|item| item.name.as_str() == device_name)
        };

        let Some(item_index) = item_index else {
            return;
        };

        let Some(device) = self.find_device(device_name) else {
            return;
        };

        self.device_items.send(
            item_index,
            BrightnessDeviceItemMsg::SetBackendBrightness(device.percentage().value()),
        );
    }

    pub(super) fn commit_brightness(
        &self,
        device_name: &str,
        percentage: f64,
        sender: &ComponentSender<Self>,
    ) {
        let Some(device) = self.find_device(device_name) else {
            return;
        };

        let device = device.clone();
        let target = Percentage::new(percentage);

        sender.command(move |_out, _shutdown| async move {
            if let Err(err) = device.set_percentage(target).await {
                warn!(error = %err, "failed to set brightness");
            }
        });
    }

    fn find_device(&self, device_name: &str) -> Option<&Arc<BacklightDevice>> {
        self.devices
            .iter()
            .find(|device| device.name.as_str() == device_name)
    }
}
