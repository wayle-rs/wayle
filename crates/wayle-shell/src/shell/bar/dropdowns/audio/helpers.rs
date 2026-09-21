use std::collections::HashMap;

use wayle_config::schemas::modules::AppIconSource;

use crate::shell::bar::icons::{lookup_app_icon, symbolic_desktop_icon};

const PA_PROP_STREAM_RESTORE_ID: &str = "module-stream-restore.id";
const PA_ROLE_EVENT: &str = "sink-input-by-media-role:event";
const PA_PROP_APP_ICON_NAME: &str = "application.icon_name";
const PA_PROP_APP_NAME: &str = "application.name";
const PA_PROP_APP_PROCESS_BINARY: &str = "application.process.binary";

pub(crate) fn is_event_stream(props: &HashMap<String, String>) -> bool {
    props
        .get(PA_PROP_STREAM_RESTORE_ID)
        .is_some_and(|id| id == PA_ROLE_EVENT)
}

pub(crate) fn stream_icon(
    props: &HashMap<String, String>,
    icon_source: AppIconSource,
) -> Option<String> {
    if icon_source == AppIconSource::Mapped {
        let candidates = [
            PA_PROP_APP_NAME,
            PA_PROP_APP_PROCESS_BINARY,
            PA_PROP_APP_ICON_NAME,
        ];

        // Theme resolution first: try each property as a desktop-entry id.
        for key in candidates {
            if let Some(value) = props.get(key)
                && let Some(icon) = symbolic_desktop_icon(value)
            {
                return Some(icon);
            }
        }

        // Built-in mappings are a last resort after theme resolution.
        for key in candidates {
            if let Some(value) = props.get(key)
                && let Some(icon) = lookup_app_icon(value)
            {
                return Some(icon.to_string());
            }
        }
    }

    match icon_source {
        AppIconSource::Native => props
            .get(PA_PROP_APP_ICON_NAME)
            .or_else(|| props.get(PA_PROP_APP_PROCESS_BINARY))
            .cloned(),
        AppIconSource::Mapped => props.get(PA_PROP_APP_ICON_NAME).map(|name| {
            if name.ends_with("-symbolic") {
                name.clone()
            } else {
                format!("{name}-symbolic")
            }
        }),
    }
}

pub(crate) fn volume_icon(percentage: f64, muted: bool) -> &'static str {
    if muted || percentage <= 0.0 {
        "ld-volume-x-symbolic"
    } else if percentage < 34.0 {
        "ld-volume-symbolic"
    } else if percentage < 67.0 {
        "ld-volume-1-symbolic"
    } else {
        "ld-volume-2-symbolic"
    }
}

pub(crate) fn input_icon(muted: bool) -> &'static str {
    if muted {
        "ld-mic-off-symbolic"
    } else {
        "ld-mic-symbolic"
    }
}

pub(crate) fn app_display_name(application_name: &Option<String>, stream_name: &str) -> String {
    let name = application_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .unwrap_or(stream_name);

    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

const PA_FORM_FACTOR: &str = "device.form_factor";
const DEFAULT_OUTPUT_ICON: &str = "tb-device-speaker-symbolic";
const DEFAULT_INPUT_ICON: &str = "tb-microphone-symbolic";

pub(crate) fn output_device_icon(
    name: &str,
    description: &str,
    properties: &HashMap<String, String>,
) -> &'static str {
    output_icon_from_form_factor(properties)
        .unwrap_or_else(|| output_icon_from_name(name, description))
}

pub(crate) fn input_device_icon(
    name: &str,
    description: &str,
    properties: &HashMap<String, String>,
) -> &'static str {
    input_icon_from_form_factor(properties)
        .unwrap_or_else(|| input_icon_from_name(name, description))
}

fn output_icon_from_form_factor(properties: &HashMap<String, String>) -> Option<&'static str> {
    match properties.get(PA_FORM_FACTOR)?.as_str() {
        "internal" | "speaker" | "hifi" => Some("tb-device-speaker-symbolic"),
        "headphone" => Some("tb-headphones-symbolic"),
        "headset" => Some("tb-headset-symbolic"),
        "phone" => Some("tb-device-mobile-symbolic"),
        "portable" => Some("tb-radio-symbolic"),
        "car" => Some("tb-car-symbolic"),
        "computer" => Some("tb-device-desktop-symbolic"),
        _ => None,
    }
}

fn output_icon_from_name(name: &str, description: &str) -> &'static str {
    let haystack = format!("{name} {description}").to_lowercase();

    if haystack.contains("hdmi") || haystack.contains("displayport") || haystack.contains("monitor")
    {
        "tb-device-tv-symbolic"
    } else if haystack.contains("headset") || haystack.contains("airpods") {
        "tb-headset-symbolic"
    } else if haystack.contains("headphone") || haystack.contains("bluetooth") {
        "tb-headphones-symbolic"
    } else {
        DEFAULT_OUTPUT_ICON
    }
}

fn input_icon_from_form_factor(properties: &HashMap<String, String>) -> Option<&'static str> {
    match properties.get(PA_FORM_FACTOR)?.as_str() {
        "internal" | "microphone" => Some("tb-microphone-symbolic"),
        "headset" => Some("tb-headset-symbolic"),
        "webcam" => Some("tb-device-computer-camera-symbolic"),
        "phone" => Some("tb-device-mobile-symbolic"),
        _ => None,
    }
}

fn input_icon_from_name(name: &str, description: &str) -> &'static str {
    let haystack = format!("{name} {description}").to_lowercase();

    if haystack.contains("headset") || haystack.contains("airpods") {
        "tb-headset-symbolic"
    } else if haystack.contains("bluetooth") {
        "tb-headphones-symbolic"
    } else if haystack.contains("webcam") || haystack.contains("camera") {
        "tb-device-computer-camera-symbolic"
    } else {
        DEFAULT_INPUT_ICON
    }
}

pub(crate) fn active_port_description(
    active_port: &Option<String>,
    ports: &[wayle_audio::types::device::DevicePort],
) -> Option<String> {
    let active_port = active_port.as_deref()?;
    ports
        .iter()
        .find(|port| port.name == active_port)
        .map(|port| port.description.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_icon_muted() {
        assert_eq!(volume_icon(50.0, true), "ld-volume-x-symbolic");
    }

    #[test]
    fn volume_icon_zero() {
        assert_eq!(volume_icon(0.0, false), "ld-volume-x-symbolic");
    }

    #[test]
    fn volume_icon_low() {
        assert_eq!(volume_icon(20.0, false), "ld-volume-symbolic");
    }

    #[test]
    fn volume_icon_medium() {
        assert_eq!(volume_icon(50.0, false), "ld-volume-1-symbolic");
    }

    #[test]
    fn volume_icon_high() {
        assert_eq!(volume_icon(80.0, false), "ld-volume-2-symbolic");
    }

    #[test]
    fn input_icon_muted_state() {
        assert_eq!(input_icon(true), "ld-mic-off-symbolic");
    }

    #[test]
    fn input_icon_active_state() {
        assert_eq!(input_icon(false), "ld-mic-symbolic");
    }

    #[test]
    fn app_name_prefers_application_name() {
        let name = app_display_name(&Some("Firefox".into()), "AudioStream");
        assert_eq!(name, "Firefox");
    }

    #[test]
    fn app_name_falls_back_to_stream_name() {
        let name = app_display_name(&None, "AudioStream");
        assert_eq!(name, "AudioStream");
    }

    #[test]
    fn app_name_ignores_empty_application_name() {
        let name = app_display_name(&Some(String::new()), "Fallback");
        assert_eq!(name, "Fallback");
    }

    #[test]
    fn app_name_capitalizes_first_letter() {
        let name = app_display_name(&Some("spotify".into()), "AudioStream");
        assert_eq!(name, "Spotify");
    }

    #[test]
    fn event_stream_detected() {
        let mut props = HashMap::new();
        props.insert(
            "module-stream-restore.id".into(),
            "sink-input-by-media-role:event".into(),
        );
        assert!(is_event_stream(&props));
    }

    #[test]
    fn non_event_stream_passes() {
        let props = HashMap::new();
        assert!(!is_event_stream(&props));
    }

    #[test]
    fn stream_icon_mapped_from_props() {
        let mut props = HashMap::new();
        props.insert("application.icon_name".into(), "firefox".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Mapped),
            Some("si-firefox-symbolic".into())
        );
    }

    #[test]
    fn stream_icon_missing() {
        let props = HashMap::new();
        assert_eq!(stream_icon(&props, AppIconSource::Mapped), None);
    }

    #[test]
    fn stream_icon_mapped_matches_icon_name_with_hyphen() {
        let mut props = HashMap::new();
        props.insert("application.name".into(), "Microsoft Edge".into());
        props.insert("application.icon_name".into(), "microsoft-edge".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Mapped),
            Some("tb-brand-edge-symbolic".into())
        );
    }

    #[test]
    fn stream_icon_mapped_falls_back_to_symbolic_pa_icon() {
        let mut props = HashMap::new();
        props.insert("application.icon_name".into(), "xyzzy-unknown".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Mapped),
            Some("xyzzy-unknown-symbolic".into())
        );
    }

    #[test]
    fn stream_icon_preserves_existing_symbolic_suffix() {
        let mut props = HashMap::new();
        props.insert("application.icon_name".into(), "my-app-symbolic".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Mapped),
            Some("my-app-symbolic".into())
        );
    }

    #[test]
    fn stream_icon_native_uses_icon_name() {
        let mut props = HashMap::new();
        props.insert("application.icon_name".into(), "firefox".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Native),
            Some("firefox".into())
        );
    }

    #[test]
    fn stream_icon_native_falls_back_to_binary() {
        let mut props = HashMap::new();
        props.insert("application.process.binary".into(), "spotify".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Native),
            Some("spotify".into())
        );
    }

    #[test]
    fn stream_icon_native_prefers_icon_name_over_binary() {
        let mut props = HashMap::new();
        props.insert("application.icon_name".into(), "custom-icon".into());
        props.insert("application.process.binary".into(), "myapp".into());
        assert_eq!(
            stream_icon(&props, AppIconSource::Native),
            Some("custom-icon".into())
        );
    }

    #[test]
    fn stream_icon_native_missing() {
        let props = HashMap::new();
        assert_eq!(stream_icon(&props, AppIconSource::Native), None);
    }

    #[test]
    fn output_icon_from_form_factor_speaker() {
        let mut props = HashMap::new();
        props.insert("device.form_factor".into(), "speaker".into());
        assert_eq!(
            output_device_icon("alsa_output.pci", "Built-in Audio", &props),
            "tb-device-speaker-symbolic"
        );
    }

    #[test]
    fn output_icon_from_form_factor_headphone() {
        let mut props = HashMap::new();
        props.insert("device.form_factor".into(), "headphone".into());
        assert_eq!(
            output_device_icon("alsa_output.pci", "Something", &props),
            "tb-headphones-symbolic"
        );
    }

    #[test]
    fn output_icon_hdmi_fallback() {
        let props = HashMap::new();
        assert_eq!(
            output_device_icon("alsa_output.hdmi", "HDMI", &props),
            "tb-device-tv-symbolic"
        );
    }

    #[test]
    fn output_icon_bluetooth_fallback() {
        let props = HashMap::new();
        assert_eq!(
            output_device_icon("bluez_output", "AirPods Pro", &props),
            "tb-headset-symbolic"
        );
    }

    #[test]
    fn output_icon_default_speaker() {
        let props = HashMap::new();
        assert_eq!(
            output_device_icon("alsa_output.pci", "Built-in Audio", &props),
            DEFAULT_OUTPUT_ICON
        );
    }

    #[test]
    fn input_icon_from_form_factor_headset() {
        let mut props = HashMap::new();
        props.insert("device.form_factor".into(), "headset".into());
        assert_eq!(
            input_device_icon("bluez_input", "Headset", &props),
            "tb-headset-symbolic"
        );
    }

    #[test]
    fn input_icon_webcam_fallback() {
        let props = HashMap::new();
        assert_eq!(
            input_device_icon("v4l2_input", "USB Webcam", &props),
            "tb-device-computer-camera-symbolic"
        );
    }

    #[test]
    fn input_icon_default_mic() {
        let props = HashMap::new();
        assert_eq!(
            input_device_icon("alsa_input.pci", "Built-in Audio", &props),
            DEFAULT_INPUT_ICON
        );
    }
}
