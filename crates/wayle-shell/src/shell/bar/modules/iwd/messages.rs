use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_iwd::IwdService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct IwdInit {
    pub settings: BarSettings,
    pub iwd: Arc<IwdService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum IwdMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum IwdCmd {
    StateChanged,
    IconConfigChanged,
    StationDeviceChanged,
}
