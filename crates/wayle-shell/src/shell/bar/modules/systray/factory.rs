use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{SystrayInit, SystrayModule};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller, require_service},
    },
    services::ShellServices,
};

pub(crate) struct Factory;

impl ModuleFactory for Factory {
    fn create(
        settings: &BarSettings,
        services: &ShellServices,
        dropdowns: &Rc<DropdownRegistry>,
        class: Option<String>,
    ) -> Option<ModuleInstance> {
        let systray = require_service("systray", "systray", services.systray.clone())?;

        let init = SystrayInit {
            is_vertical: settings.is_vertical.clone(),
            systray,
            config: services.config.clone(),
            coordinator: dropdowns.coordinator(),
            shell_ipc: services.shell_ipc.state(),
            monitor: settings.monitor_name.clone(),
        };
        let controller = dynamic_controller(SystrayModule::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
