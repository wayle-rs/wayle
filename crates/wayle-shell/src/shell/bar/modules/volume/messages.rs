use std::{rc::Rc, sync::Arc};

use wayle_audio::{AudioService, core::device::output::OutputDevice};
use wayle_config::{ConfigService, schemas::styling::ThresholdColors};
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct VolumeInit {
    pub settings: BarSettings,
    pub audio: Arc<AudioService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum VolumeMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum VolumeCmd {
    DeviceChanged(Option<Arc<OutputDevice>>),
    VolumeOrMuteChanged,
    IconConfigChanged,
    UpdateThresholdColors(ThresholdColors),
}
