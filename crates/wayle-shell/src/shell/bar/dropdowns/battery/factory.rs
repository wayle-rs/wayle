use relm4::prelude::*;

use super::{BatteryDropdown, messages::BatteryDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let battery = require_service("battery", "battery", services.battery.clone())?;
        let power_profiles = services.power_profiles.clone();
        let config = services.config.clone();

        let init = BatteryDropdownInit {
            battery,
            power_profiles,
            config,
        };
        let controller = BatteryDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
