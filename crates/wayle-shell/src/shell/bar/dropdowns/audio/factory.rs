use relm4::prelude::*;

use super::{AudioDropdown, messages::AudioDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance, require_service},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let audio = require_service("audio", "audio", services.audio.clone())?;
        let config = services.config.clone();

        let init = AudioDropdownInit { audio, config };
        let controller = AudioDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
