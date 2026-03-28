mod battery;
mod bluetooth;
mod cava;
mod clock;
mod compositor;
mod cpu;
mod cpuchart;
mod custom;
mod dashboard;
mod hyprland_workspaces;
mod hyprsunset;
mod idle_inhibit;
mod keybind_mode;
mod keyboard_input;
mod media;
mod microphone;
mod netstat;
mod network;
mod notification;
mod power;
mod ram;
mod registry;
mod separator;
mod shared;
mod storage;
mod systray;
mod volume;
pub(crate) mod weather;
mod window_title;
mod world_clock;

use std::rc::Rc;

use tracing::warn;
use wayle_config::schemas::bar::{BarModule, ModuleRef};
use wayle_widgets::prelude::BarSettings;

pub(crate) use self::registry::{ModuleFactory, ModuleInstance};
use crate::shell::{bar::dropdowns::DropdownRegistry, services::ShellServices};

macro_rules! register_modules {
    ($($variant:ident => $factory:ty),+ $(,)?) => {
        fn create_from_variant(
            module: BarModule,
            settings: &BarSettings,
            services: &ShellServices,
            dropdowns: &Rc<DropdownRegistry>,
            class: Option<String>,
        ) -> Option<ModuleInstance> {
            match module {
                $(BarModule::$variant => <$factory as ModuleFactory>::create(settings, services, dropdowns, class),)+
                _ => {
                    warn!(?module, "module not implemented");
                    None
                }
            }
        }
    };
}

register_modules! {
    Battery => battery::Factory,
    Bluetooth => bluetooth::Factory,
    Cava => cava::Factory,
    Clock => clock::Factory,
    Cpu => cpu::Factory,
    CpuChart => cpuchart::Factory,
    Dashboard => dashboard::Factory,
    HyprlandWorkspaces => hyprland_workspaces::Factory,
    Hyprsunset => hyprsunset::Factory,
    IdleInhibit => idle_inhibit::Factory,
    KeybindMode => keybind_mode::Factory,
    KeyboardInput => keyboard_input::Factory,
    Media => media::Factory,
    Microphone => microphone::Factory,
    Netstat => netstat::Factory,
    Network => network::Factory,
    Notifications => notification::Factory,
    Power => power::Factory,
    Ram => ram::Factory,
    Separator => separator::Factory,
    Storage => storage::Factory,
    Systray => systray::Factory,
    Volume => volume::Factory,
    Weather => weather::Factory,
    WindowTitle => window_title::Factory,
    WorldClock => world_clock::Factory,
}

pub(crate) fn create_module(
    module_ref: &ModuleRef,
    settings: &BarSettings,
    services: &ShellServices,
    dropdowns: &Rc<DropdownRegistry>,
) -> Option<ModuleInstance> {
    let module = module_ref.module();
    let class = module_ref.class().map(String::from);

    if let Some(id) = module.custom_id() {
        return custom::Factory::create_for_id(id, settings, services, dropdowns, class);
    }

    create_from_variant(module.clone(), settings, services, dropdowns, class)
}
