use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_weather::WeatherService; // placeholder, not used directly
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct AuroraInit {
    pub settings: BarSettings,
    pub weather: Arc<WeatherService>, // reusing weather service as placeholder
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum AuroraMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum AuroraCmd {
    UpdateLabel(String),
    UpdateIcon(String),
}
