use relm4::prelude::*;

use super::{SysinfoDropdown, messages::SysinfoDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let sysinfo = services.sysinfo.clone();
        let config = services.config.clone();

        let init = SysinfoDropdownInit { sysinfo, config };
        let controller = SysinfoDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
