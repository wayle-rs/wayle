use std::{rc::Rc, sync::Arc};

use wayle_config::{ConfigService, schemas::styling::ThresholdColors};
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct UpdatesInit {
    pub settings: BarSettings,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum UpdatesMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum UpdatesCmd {
    UpdateLabel(String),
    UpdateIcon(String),
    UpdateThresholdColors(ThresholdColors),
    UpdateVisibility(bool),
}
