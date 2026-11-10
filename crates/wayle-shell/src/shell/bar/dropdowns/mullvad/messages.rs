use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_mullvad::MullvadService;

use super::{country_item::CountryItemOutput, current_connection::CurrentConnectionOutput};

pub(crate) struct MullvadDropdownInit {
    pub mullvad: DeferredService<MullvadService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum MullvadDropdownMsg {
    Country(CountryItemOutput),
    Current(CurrentConnectionOutput),
}

#[derive(Debug)]
pub(crate) enum MullvadDropdownCmd {
    ServiceReady(Arc<MullvadService>),
    ScaleChanged(f32),
    RelaysChanged,
    TunnelChanged,
}
