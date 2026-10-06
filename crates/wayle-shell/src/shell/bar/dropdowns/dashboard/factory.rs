use relm4::prelude::*;

use super::{DashboardDropdown, messages::DashboardDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let init = DashboardDropdownInit {
            audio: services.audio.clone(),
            battery: services.battery.clone(),
            bluetooth: services.bluetooth.clone(),
            config: services.config.clone(),
            media: services.media.clone(),
            network: services.network.clone(),
            iwd: services.iwd.clone(),
            notification: services.notification.clone(),
            power_profiles: services.power_profiles.clone(),
            sysinfo: services.sysinfo.clone(),
            idle_inhibit: services.idle_inhibit.clone(),
        };

        let controller = DashboardDropdown::builder().launch(init).detach();
        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
