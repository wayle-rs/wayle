use std::{rc::Rc, sync::Arc};

use wayle_config::{ConfigService, schemas::styling::ThresholdColors};
use wayle_sysinfo::SysinfoService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct RamInit {
    pub settings: BarSettings,
    pub sysinfo: Arc<SysinfoService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum RamMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum RamCmd {
    UpdateLabel(String),
    UpdateIcon(String),
    UpdateThresholdColors(ThresholdColors),
}
