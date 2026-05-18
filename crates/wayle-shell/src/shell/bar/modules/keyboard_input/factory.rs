//! Factory entry point: detect the compositor, build the matching
//! [`KeyboardLayoutSource`], and launch the [`KeyboardInput`] component.

use std::{rc::Rc, sync::Arc};

use relm4::prelude::*;
use tracing::warn;
use wayle_widgets::prelude::BarSettings;

use super::{
    KeyboardInput, KeyboardInputInit,
    sources::{HyprlandKeyboardLayoutSource, KeyboardLayoutSource, NiriKeyboardLayoutSource},
};
use crate::shell::{
    bar::{
        dropdowns::DropdownRegistry,
        modules::{
            compositor::Compositor,
            registry::{ModuleFactory, ModuleInstance, dynamic_controller, require_service},
        },
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
        let source = build_source(services)?;

        let init = KeyboardInputInit {
            settings: settings.clone(),
            source,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(KeyboardInput::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}

fn build_source(services: &ShellServices) -> Option<Arc<dyn KeyboardLayoutSource>> {
    match Compositor::detect() {
        Compositor::Hyprland => {
            let hyprland =
                require_service("keyboard-input", "hyprland", services.hyprland.clone())?;
            Some(Arc::new(HyprlandKeyboardLayoutSource::new(hyprland)))
        }
        Compositor::Niri => {
            let niri = require_service("keyboard-input", "niri", services.niri.clone())?;
            Some(Arc::new(NiriKeyboardLayoutSource::new(niri)))
        }
        Compositor::Unknown(name) => {
            warn!(module = "keyboard-input", compositor = %name, "unsupported compositor");
            None
        }
    }
}
