use relm4::prelude::*;

use super::{CustomDropdown, CustomDropdownInit};
use crate::shell::{bar::dropdowns::DropdownInstance, services::ShellServices};

/// Creates a custom dropdown from config, or `None` if the definition is missing.
pub(crate) fn create(name: &str, services: &ShellServices) -> Option<DropdownInstance> {
    let config = services.config.config();
    let dropdowns = config.dropdowns.get();
    let Some(definition) = dropdowns.custom.get(name).cloned() else {
        tracing::warn!(name, "custom dropdown definition not found in config");
        return None;
    };

    tracing::info!(name, "creating custom dropdown");

    let init = CustomDropdownInit {
        definition,
        config: services.config.clone(),
    };
    let controller = CustomDropdown::builder().launch(init).detach();

    let popover = controller.widget().clone();
    Some(DropdownInstance::new(popover, Box::new(controller)))
}
