use relm4::prelude::*;

use super::{NetworkDropdown, messages::NetworkDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let network = require_service("network", "network", services.network.clone())?;
        let config = services.config.clone();

        let init = NetworkDropdownInit { network, config };
        let controller = NetworkDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
