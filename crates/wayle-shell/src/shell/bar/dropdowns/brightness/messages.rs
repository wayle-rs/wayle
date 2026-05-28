use std::sync::Arc;

use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::ConfigService;

pub(crate) struct BrightnessDropdownInit {
    pub brightness: Arc<BrightnessService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum BrightnessDropdownInput {
    DeviceBrightnessChanged(String, f64),
}

#[derive(Debug)]
pub(crate) enum BrightnessDropdownCmd {
    DevicesChanged(Vec<Arc<BacklightDevice>>),
    DeviceBrightnessUpdated(String),
    ScaleChanged(f32),
}
