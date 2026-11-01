use relm4::prelude::*;

use super::{BrightnessDropdown, messages::BrightnessDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let brightness = require_service("brightness", "brightness", services.brightness.clone())?;
        let config = services.config.clone();

        let init = BrightnessDropdownInit { brightness, config };
        let controller = BrightnessDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
