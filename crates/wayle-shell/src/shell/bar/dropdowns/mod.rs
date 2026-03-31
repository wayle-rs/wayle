mod audio;
mod battery;
mod bluetooth;
mod calendar;
mod custom;
mod dashboard;
mod media;
mod network;
mod notification;
mod registry;
mod weather;

pub(crate) use self::registry::{
    DropdownFactory, DropdownInstance, DropdownRegistry, dispatch_click, dispatch_click_widget,
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
                    if let Some(custom_name) = name.strip_prefix("custom:") {
                        custom::create(custom_name, services)
                    } else {
                        tracing::warn!(dropdown = name, "unknown dropdown type");
                        None
                    }
                }
            }
        }
    };
}

register_dropdowns! {
    "audio" => audio::Factory,
    "battery" => battery::Factory,
    "bluetooth" => bluetooth::Factory,
    "calendar" => calendar::Factory,
    "dashboard" => dashboard::Factory,
    "media" => media::Factory,
    "network" => network::Factory,
    "notification" => notification::Factory,
    "weather" => weather::Factory,
}
