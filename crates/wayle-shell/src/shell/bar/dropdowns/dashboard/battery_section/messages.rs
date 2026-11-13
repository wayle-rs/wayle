use std::sync::Arc;

use wayle_battery::types::DeviceState;
use wayle_core::Property;
use wayle_power_profiles::{PowerProfilesService, types::profile::PowerProfile};

use crate::services::BatteryService;

pub(crate) struct BatterySectionInit {
    pub battery: Option<Arc<BatteryService>>,
    pub power_profiles: Property<Option<Arc<PowerProfilesService>>>,
}

#[derive(Debug)]
pub(crate) enum BatterySectionCmd {
    StateChanged {
        percentage: f64,
        state: DeviceState,
        time_remaining_secs: i64,
    },
    PowerProfileChanged(PowerProfile),
    PowerProfilesAvailable(Arc<PowerProfilesService>),
    PowerProfilesUnavailable,
}
