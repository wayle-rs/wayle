mod audio;
mod battery;
mod bluetooth;
mod brightness;
mod calendar;
mod dashboard;
mod iwd;
mod media;
mod network;
mod notification;
mod registry;
mod weather;

use wayle_iwd::SignalStrength;

pub(crate) use self::registry::{
    DropdownFactory, DropdownInstance, DropdownRegistry, dispatch_click, dispatch_click_widget,
    require_service,
};
use crate::shell::services::ShellServices;

pub(crate) fn scaled_dimension(base: f32, scale: f32) -> i32 {
    (base * scale).round() as i32
}

/// Maps a WiFi channel frequency (MHz) to its band label, shared by the
/// NetworkManager and IWD dropdowns.
pub(crate) fn frequency_to_band(freq_mhz: u32) -> Option<&'static str> {
    match freq_mhz {
        2400..=2500 => Some("2.4 GHz"),
        5000..=5900 => Some("5 GHz"),
        5901..=7125 => Some("6 GHz"),
        57000..=71000 => Some("60 GHz"),
        _ => None,
    }
}

/// Picks the configured signal-strength icon for a bucket, scaling the bucket
/// onto the configured icon list; `fallback` (the configured "connected, strength
/// unknown" icon) is used when the list is empty. Shared by the IWD bar module
/// and dropdown.
pub(crate) fn signal_strength_icon(
    strength: SignalStrength,
    icons: &[String],
    fallback: &str,
) -> String {
    strength
        .icon_index(icons.len())
        .and_then(|idx| icons.get(idx))
        .cloned()
        .unwrap_or_else(|| fallback.to_string())
}

/// Icon for a connected link's signal, treating unknown strength (`None`) as the
/// weakest [`SignalStrength::None`] bucket — so it shows the configured `icons[0]`
/// (e.g. `cm-wireless-signal-none-symbolic`) rather than a distinct placeholder.
/// `fallback` (the "connected, strength unknown" icon) applies only when the icon
/// list is empty. Shared by the IWD bar module and the dropdown's
/// active-connection card.
pub(crate) fn connected_signal_icon(
    strength: Option<SignalStrength>,
    icons: &[String],
    fallback: &str,
) -> String {
    signal_strength_icon(strength.unwrap_or(SignalStrength::None), icons, fallback)
}

#[cfg(test)]
mod tests {
    use wayle_iwd::SignalStrength;

    use super::{connected_signal_icon, frequency_to_band, signal_strength_icon};

    #[test]
    fn frequency_2ghz_band() {
        assert_eq!(frequency_to_band(2412), Some("2.4 GHz"));
        assert_eq!(frequency_to_band(2437), Some("2.4 GHz"));
        assert_eq!(frequency_to_band(2484), Some("2.4 GHz"));
    }

    #[test]
    fn frequency_5ghz_band() {
        assert_eq!(frequency_to_band(5180), Some("5 GHz"));
        assert_eq!(frequency_to_band(5745), Some("5 GHz"));
        assert_eq!(frequency_to_band(5825), Some("5 GHz"));
    }

    #[test]
    fn frequency_6ghz_band() {
        assert_eq!(frequency_to_band(5955), Some("6 GHz"));
        assert_eq!(frequency_to_band(6115), Some("6 GHz"));
        assert_eq!(frequency_to_band(7115), Some("6 GHz"));
    }

    #[test]
    fn frequency_60ghz_band() {
        assert_eq!(frequency_to_band(60000), Some("60 GHz"));
    }

    #[test]
    fn frequency_unknown_band() {
        assert_eq!(frequency_to_band(0), None);
        assert_eq!(frequency_to_band(900), None);
    }

    #[test]
    fn signal_icon_buckets() {
        // A 4-icon list (no "none" entry) maps both None and Weak to the weakest
        // icon. (The shipped iwd default is a 5-icon list that includes "none".)
        let icons = vec![
            String::from("weak"),
            String::from("ok"),
            String::from("good"),
            String::from("excellent"),
        ];
        assert_eq!(signal_strength_icon(SignalStrength::None, &icons, "connected"), "weak");
        assert_eq!(signal_strength_icon(SignalStrength::Weak, &icons, "connected"), "weak");
        assert_eq!(signal_strength_icon(SignalStrength::Ok, &icons, "connected"), "ok");
        assert_eq!(signal_strength_icon(SignalStrength::Good, &icons, "connected"), "good");
        assert_eq!(
            signal_strength_icon(SignalStrength::Excellent, &icons, "connected"),
            "excellent"
        );
        // Empty list falls back to the configured connected icon.
        assert_eq!(signal_strength_icon(SignalStrength::Good, &[], "connected"), "connected");
    }

    #[test]
    fn connected_signal_icon_maps_unknown_to_none_bucket() {
        let icons = vec![
            String::from("none"),
            String::from("weak"),
            String::from("ok"),
            String::from("good"),
            String::from("excellent"),
        ];
        // Unknown strength renders the weakest/"none" icon, not the fallback.
        assert_eq!(connected_signal_icon(None, &icons, "connected"), "none");
        assert_eq!(connected_signal_icon(Some(SignalStrength::Good), &icons, "connected"), "good");
        // Fallback applies only when the list is empty.
        assert_eq!(connected_signal_icon(None, &[], "connected"), "connected");
    }
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
    "iwd" => iwd::Factory,
    "media" => media::Factory,
    "network" => network::Factory,
    "notification" => notification::Factory,
    "weather" => weather::Factory,
}
