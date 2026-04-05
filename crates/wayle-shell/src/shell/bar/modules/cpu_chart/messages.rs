use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_sysinfo::SysinfoService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct CpuChartInit {
    pub config: Arc<ConfigService>,
    pub sysinfo: Arc<SysinfoService>,
    pub settings: BarSettings,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum CpuChartMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum CpuChartCmd {
    Update(Vec<f64>),
    Resize,
}
