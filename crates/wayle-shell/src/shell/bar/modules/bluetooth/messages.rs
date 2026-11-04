use std::{rc::Rc, sync::Arc};

use wayle_bluetooth::BluetoothService;
use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct BluetoothInit {
    pub settings: BarSettings,
    pub bluetooth: DeferredService<BluetoothService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum BluetoothMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum BluetoothCmd {
    ServiceReady(Arc<BluetoothService>),
    StateChanged,
    IconConfigChanged,
}
