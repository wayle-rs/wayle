use wayle_brightness::types::BacklightType;

pub(crate) fn device_subtitle(
    device_name: &str,
    kind: BacklightType,
    multi: bool,
) -> Option<String> {
    if !multi {
        return None;
    }

    Some(format!(
        "{device_name} \u{00b7} {}",
        backlight_type_label(kind)
    ))
}

pub(crate) fn backlight_type_label(kind: BacklightType) -> &'static str {
    match kind {
        BacklightType::Raw => "raw",
        BacklightType::Platform => "platform",
        BacklightType::Firmware => "firmware",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_device_has_no_subtitle() {
        assert_eq!(
            device_subtitle("intel_backlight", BacklightType::Firmware, false),
            None
        );
    }

    #[test]
    fn multi_device_subtitle_includes_name_and_type() {
        assert_eq!(
            device_subtitle("intel_backlight", BacklightType::Firmware, true),
            Some(String::from("intel_backlight \u{00b7} firmware"))
        );
    }

    #[test]
    fn backlight_type_labels() {
        assert_eq!(backlight_type_label(BacklightType::Raw), "raw");
        assert_eq!(backlight_type_label(BacklightType::Platform), "platform");
        assert_eq!(backlight_type_label(BacklightType::Firmware), "firmware");
    }
}
