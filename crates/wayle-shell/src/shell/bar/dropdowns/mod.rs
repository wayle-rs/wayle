mod audio;
mod battery;
mod bluetooth;
mod brightness;
mod calendar;
mod coordinator;
mod dashboard;
mod media;
mod network;
mod notification;
mod registry;
mod scrim;
mod weather;

pub(crate) use self::coordinator::{
    DismissFn, OPENER_CSS_CLASS, OpenSurfaceCoordinator, SECONDARY_OPENER_CSS_CLASS,
};
pub(crate) use self::registry::{
    DropdownFactory, DropdownInstance, DropdownOpener, DropdownRegistry, require_service,
};
use crate::shell::services::ShellServices;

pub(crate) fn scaled_dimension(base: f32, scale: f32) -> i32 {
    (base * scale).round() as i32
}

macro_rules! register_dropdowns {
    ($($name:literal => $factory:ty),+ $(,)?) => {
        pub(crate) const DROPDOWN_NAMES: &[&str] = &[$($name),+];

        pub(crate) fn create(
            name: &str,
            services: &ShellServices,
        ) -> Option<DropdownInstance> {
            match name {
                $($name => <$factory as DropdownFactory>::create(services),)+
                _ => {
                    tracing::warn!(dropdown = name, "unknown dropdown type");
                    None
                }
            }
        }
    };
}

register_dropdowns! {
    "audio" => audio::Factory,
    "battery" => battery::Factory,
    "bluetooth" => bluetooth::Factory,
    "brightness" => brightness::Factory,
    "calendar" => calendar::Factory,
    "dashboard" => dashboard::Factory,
    "media" => media::Factory,
    "network" => network::Factory,
    "notification" => notification::Factory,
    "weather" => weather::Factory,
}
