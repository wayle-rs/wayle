use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_mullvad::MullvadService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct MullvadInit {
    pub settings: BarSettings,
    pub mullvad: DeferredService<MullvadService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum MullvadMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum MullvadCmd {
    ServiceReady(Arc<MullvadService>),
    StateChanged,
    IconConfigChanged,
}
