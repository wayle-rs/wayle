//! Factory entry point: detect the compositor, build the matching
//! [`FocusedWindowSource`], and launch the [`WindowTitle`] component.

use std::{rc::Rc, sync::Arc};

use relm4::prelude::*;
use tracing::warn;
use wayle_widgets::prelude::BarSettings;

use super::{
    WindowTitle, WindowTitleInit,
    sources::{
        FocusedWindowSource, HyprlandFocusedWindowSource, MangoFocusedWindowSource,
        NiriFocusedWindowSource,
    },
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

        let init = WindowTitleInit {
            settings: settings.clone(),
            source,
            config: services.config.clone(),
            dropdowns: dropdowns.clone(),
        };
        let controller = dynamic_controller(WindowTitle::builder().launch(init).detach());
        Some(ModuleInstance { controller, class })
    }
}

fn build_source(services: &ShellServices) -> Option<Arc<dyn FocusedWindowSource>> {
    match Compositor::detect() {
        Compositor::Hyprland => {
            let hyprland = require_service("window-title", "hyprland", services.hyprland.clone())?;
            Some(Arc::new(HyprlandFocusedWindowSource::new(hyprland)))
        }
        Compositor::Niri => {
            let niri = require_service("window-title", "niri", services.niri.clone())?;
            Some(Arc::new(NiriFocusedWindowSource::new(niri)))
        }
        Compositor::Mango => {
            let mango = require_service("window-title", "mango", services.mango.clone())?;
            Some(Arc::new(MangoFocusedWindowSource::new(mango)))
        }
        Compositor::Unknown(name) => {
            warn!(module = "window-title", compositor = %name, "unsupported compositor");
            None
        }
    }
}
