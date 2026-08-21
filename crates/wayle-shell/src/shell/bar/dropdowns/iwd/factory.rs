use relm4::prelude::*;

use super::{IwdDropdown, messages::IwdDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let iwd = require_service("iwd", "iwd", services.iwd.clone())?;
        let config = services.config.clone();

        let init = IwdDropdownInit { iwd, config };
        let controller = IwdDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
