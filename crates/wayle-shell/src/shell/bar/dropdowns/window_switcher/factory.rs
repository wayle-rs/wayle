use relm4::prelude::*;

use super::{WindowSwitcherDropdown, messages::WindowSwitcherDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let service = require_service(
            "window-switcher",
            "wlr-toplevel",
            services.wlr_toplevel.clone(),
        )?;

        let init = WindowSwitcherDropdownInit {
            service,
            config: services.config.clone(),
        };
        let controller = WindowSwitcherDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
