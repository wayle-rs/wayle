use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{WindowSwitcherInit, WindowSwitcherModule};
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
        let service = require_service(
            "window-switcher",
            "wlr-toplevel",
            services.wlr_toplevel.clone(),
        )?;

        let init = WindowSwitcherInit {
            settings: settings.clone(),
            service,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
            ipc_state: services.shell_ipc.state(),
        };
        let controller = dynamic_controller(WindowSwitcherModule::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
