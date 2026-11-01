//! Factory entry: gate on the Niri compositor + service availability,
//! then launch the [`NiriWorkspaces`] component.

use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{NiriWorkspaces, NiriWorkspacesInit};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller, require_niri},
    },
    services::ShellServices,
};

/// Module factory that launches [`NiriWorkspaces`] when niri is the
/// active compositor and the [`NiriService`] is available.
///
/// [`NiriWorkspaces`]: super::NiriWorkspaces
/// [`NiriService`]: wayle_niri::NiriService
pub(crate) struct Factory;

impl ModuleFactory for Factory {
    fn create(
        settings: &BarSettings,
        services: &ShellServices,
        dropdowns: &Rc<DropdownRegistry>,
        class: Option<String>,
    ) -> Option<ModuleInstance> {
        if !require_niri("niri-workspaces") {
            return None;
        }
        let niri = services.niri.clone()?;

        let init = NiriWorkspacesInit {
            settings: settings.clone(),
            niri,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(NiriWorkspaces::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
