//! Factory entry: gate on `ExtWorkspaceService` availability (capability
//! detection, not compositor-name gating - this module works on any
//! compositor advertising `ext_workspace_manager_v1`), then launch the
//! [`ExtWorkspaces`] component.

use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{ExtWorkspaces, ExtWorkspacesInit};
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
            "ext-workspaces",
            "ext-workspace",
            services.ext_workspaces.clone(),
        )?;

        let init = ExtWorkspacesInit {
            settings: settings.clone(),
            service,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(ExtWorkspaces::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
