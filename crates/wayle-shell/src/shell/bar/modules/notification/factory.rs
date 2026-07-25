use std::rc::Rc;

use relm4::prelude::*;
use wayle_widgets::prelude::BarSettings;

use super::{NotificationInit, NotificationModule};
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
        let notification_enabled = services.config.config().modules.notifications.enabled.get();
        let notification = require_service(
            "notification",
            "notification",
            services.notification.clone(),
        )?;

        if !notification_enabled {
            return None;
        }

        let init = NotificationInit {
            settings: settings.clone(),
            notification,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(NotificationModule::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}
