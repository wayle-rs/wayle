use relm4::prelude::*;

use super::{PomodoroDropdown, messages::PomodoroDropdownInit};
use crate::shell::{
    bar::dropdowns::{DropdownFactory, DropdownInstance},
    services::ShellServices,
};

pub(crate) struct Factory;

impl DropdownFactory for Factory {
    fn create(services: &ShellServices) -> Option<DropdownInstance> {
        let init = PomodoroDropdownInit {
            config: services.config.clone(),
            state: services.pomodoro_state.clone(),
        };
        let controller = PomodoroDropdown::builder().launch(init).detach();

        let popover = controller.widget().clone();
        Some(DropdownInstance::new(popover, Box::new(controller)))
    }
}
