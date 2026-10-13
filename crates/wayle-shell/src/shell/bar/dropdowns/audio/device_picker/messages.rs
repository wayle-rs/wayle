use std::sync::Arc;

use wayle_audio::AudioService;
use wayle_config::ConfigService;

use crate::shell::bar::dropdowns::audio::VolumeSectionKind;

#[derive(Debug)]
pub(crate) struct DeviceInfo {
    pub description: String,
    pub subtitle: Option<String>,
    pub icon: &'static str,
    pub is_active: bool,
}

pub(crate) struct DevicePickerInit {
    pub audio: Arc<AudioService>,
    pub config: Arc<ConfigService>,
    pub kind: VolumeSectionKind,
    pub title: String,
}

#[derive(Debug)]
pub(crate) enum DevicePickerInput {
    DeviceSelected(usize),
    BackClicked,
}

#[derive(Debug)]
pub(crate) enum DevicePickerCmd {
    DevicesChanged(Vec<DeviceInfo>),
}

#[derive(Debug)]
pub(crate) enum DevicePickerOutput {
    NavigateBack,
}
