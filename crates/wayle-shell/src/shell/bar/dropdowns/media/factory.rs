use relm4::prelude::*;

use super::{MediaDropdown, messages::MediaDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let media = require_service("media", "media", services.media.clone())?;
        let config = services.config.clone();

        let init = MediaDropdownInit { media, config };
        let controller = MediaDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
