use relm4::prelude::*;

use super::{MullvadDropdown, messages::MullvadDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let init = MullvadDropdownInit {
            mullvad: services.mullvad.clone(),
            config: services.config.clone(),
        };
        let controller = MullvadDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
