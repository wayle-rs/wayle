use std::time::Duration;

use wayle_media::core::player::Player;

use crate::shell::bar::icons;

pub(super) fn resolve_source_icon(player: &Player) -> String {
    let identity = player.identity.get();
    if let Some(icon) = icons::lookup_app_icon(&identity) {
        return icon.to_string();
    }

    let bus_name = player.id.bus_name();
    if let Some(icon) = icons::lookup_app_icon(bus_name) {
        return icon.to_string();
    }

    let Some(desktop_entry) = player.desktop_entry.get() else {
        return String::from("ld-music-symbolic");
    };

    icons::lookup_app_icon(&desktop_entry)
        .map(|icon| icon.to_string())
        .unwrap_or_else(|| format!("{desktop_entry}-symbolic"))
}

pub(super) struct DurationParts {
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
}

pub(super) fn duration_parts(duration: Duration) -> DurationParts {
    let total_secs = duration.as_secs();
    DurationParts {
        hours: total_secs / 3600,
        minutes: (total_secs % 3600) / 60,
        seconds: total_secs % 60,
    }
}

pub(super) fn format_duration(duration: Duration) -> String {
    let parts = duration_parts(duration);
    if parts.hours > 0 {
        return format!("{}:{:02}:{:02}", parts.hours, parts.minutes, parts.seconds);
    }
    format!("{}:{:02}", parts.minutes, parts.seconds)
}

pub(super) fn progress_fraction(position: Duration, length: Duration) -> f64 {
    let length_secs = length.as_secs_f64();
    if length_secs <= 0.0 {
        return 0.0;
    }
    (position.as_secs_f64() / length_secs).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_parts_zero() {
        let parts = duration_parts(Duration::ZERO);
        assert_eq!(parts.minutes, 0);
        assert_eq!(parts.seconds, 0);
    }

    #[test]
    fn duration_parts_mixed() {
        let parts = duration_parts(Duration::from_secs(142));
        assert_eq!(parts.minutes, 2);
        assert_eq!(parts.seconds, 22);
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(Duration::ZERO), "0:00");
    }

    #[test]
    fn format_duration_seconds_padded() {
        assert_eq!(format_duration(Duration::from_secs(65)), "1:05");
    }

    #[test]
    fn format_duration_long() {
        assert_eq!(format_duration(Duration::from_secs(354)), "5:54");
    }

    #[test]
    fn format_duration_very_long() {
        assert_eq!(
            format_duration(
                Duration::from_hours(2) + Duration::from_mins(35) + Duration::from_secs(49)
            ),
            "2:35:49"
        );
    }

    #[test]
    fn progress_fraction_zero_length() {
        assert_eq!(
            progress_fraction(Duration::from_secs(10), Duration::ZERO),
            0.0
        );
    }

    #[test]
    fn progress_fraction_midway() {
        let frac = progress_fraction(Duration::from_secs(50), Duration::from_secs(100));
        assert!((frac - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_fraction_clamped() {
        let frac = progress_fraction(Duration::from_secs(200), Duration::from_secs(100));
        assert_eq!(frac, 1.0);
    }
}
