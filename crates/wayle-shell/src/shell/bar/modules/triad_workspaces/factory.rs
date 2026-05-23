//! Factory entry: gate on the Triad compositor + service availability,
//! then launch the [`TriadWorkspaces`] component.

use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{TriadWorkspaces, TriadWorkspacesInit};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller, require_triad},
    },
    services::ShellServices,
};

/// Module factory that launches [`TriadWorkspaces`] when triad is the
/// active compositor and the [`TriadService`] is available.
///
/// [`TriadWorkspaces`]: super::TriadWorkspaces
/// [`TriadService`]: wayle_triad::TriadService
pub(crate) struct Factory;

impl ModuleFactory for Factory {
    fn create(
        settings: &BarSettings,
        services: &ShellServices,
        dropdowns: &Rc<DropdownRegistry>,
        class: Option<String>,
    ) -> Option<ModuleInstance> {
        if !require_triad("triad-workspaces") {
            return None;
        }
        let triad = services.triad.clone()?;

        let init = TriadWorkspacesInit {
            settings: settings.clone(),
            triad,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(TriadWorkspaces::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
