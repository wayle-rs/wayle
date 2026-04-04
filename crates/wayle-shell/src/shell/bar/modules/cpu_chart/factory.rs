use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{CpuChartInit, CpuChartModule};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::registry::{ModuleFactory, ModuleInstance, dynamic_controller},
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
        let init = CpuChartInit {
            config: services.config.clone(),
            sysinfo: services.sysinfo.clone(),
            settings: settings.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(CpuChartModule::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
