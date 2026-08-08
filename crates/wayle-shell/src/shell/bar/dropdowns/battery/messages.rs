use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_core::Property;
use wayle_power_profiles::PowerProfilesService;

use crate::services::BatteryService;

pub(crate) struct BatteryDropdownInit {
    pub battery: Arc<BatteryService>,
    pub power_profiles: Property<Option<Arc<PowerProfilesService>>>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum BatteryDropdownCmd {
    ScaleChanged(f32),
}
