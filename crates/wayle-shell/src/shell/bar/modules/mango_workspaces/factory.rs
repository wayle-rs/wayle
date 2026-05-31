//! Factory entry: gate on the Mango compositor + service availability,
//! then launch the [`MangoWorkspaces`] component.

use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{MangoWorkspaces, MangoWorkspacesInit};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller, require_mango},
    },
    services::ShellServices,
};

/// Module factory that launches [`MangoWorkspaces`] when Mango is the active
/// compositor and the [`MangoService`] is available.
///
/// [`MangoWorkspaces`]: super::MangoWorkspaces
/// [`MangoService`]: wayle_mango::MangoService
pub(crate) struct Factory;

impl ModuleFactory for Factory {
    fn create(
        settings: &BarSettings,
        services: &ShellServices,
        dropdowns: &Rc<DropdownRegistry>,
        class: Option<String>,
    ) -> Option<ModuleInstance> {
        if !require_mango("mango-workspaces") {
            return None;
        }
        let mango = services.mango.clone()?;

        let init = MangoWorkspacesInit {
            settings: settings.clone(),
            mango,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(MangoWorkspaces::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
