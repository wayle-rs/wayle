use serde_json::json;
use wayle_battery::types::DeviceState;

use crate::i18n::t;

pub(crate) struct IconContext<'a> {
    pub(crate) percentage: f64,
    pub(crate) state: DeviceState,
    pub(crate) is_present: bool,
    pub(crate) level_icons: &'a [String],
    pub(crate) charging_icon: &'a str,
    pub(crate) alert_icon: &'a str,
}

pub(crate) fn select_icon(ctx: &IconContext<'_>) -> String {
    if !ctx.is_present || matches!(ctx.state, DeviceState::Unknown) {
        return ctx.alert_icon.to_string();
    }

    if matches!(
        ctx.state,
        DeviceState::Charging | DeviceState::PendingCharge
    ) {
        return ctx.charging_icon.to_string();
    }

    if ctx.level_icons.is_empty() {
        return ctx.alert_icon.to_string();
    }

    let index = ((ctx.percentage / 100.0) * ctx.level_icons.len() as f64)
        .floor()
        .min((ctx.level_icons.len() - 1) as f64) as usize;

    ctx.level_icons
        .get(index)
        .cloned()
        .unwrap_or_else(|| ctx.level_icons.last().cloned().unwrap_or_default())
}

pub(crate) fn format_label(format: &str, percentage: f64, is_present: bool) -> String {
    if is_present {
        let ctx = json!({
            "percent": percentage.round() as u32,
        });
        crate::template::render(format, ctx).unwrap_or_default()
    } else {
        t!("bar-battery-unavailable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level_icons() -> Vec<String> {
        vec![
            "battery-empty".into(),
            "battery-low".into(),
            "battery-medium".into(),
            "battery-high".into(),
            "battery-full".into(),
        ]
    }

    #[test]
    fn select_icon_not_present() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 50.0,
            state: DeviceState::Discharging,
            is_present: false,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "alert");
    }

    #[test]
    fn select_icon_unknown_state() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 100.0,
            state: DeviceState::Unknown,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "alert");
    }

    #[test]
    fn select_icon_charging() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 25.0,
            state: DeviceState::Charging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "charging");
    }

    #[test]
    fn select_icon_pending_charge() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 80.0,
            state: DeviceState::PendingCharge,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "charging");
    }

    #[test]
    fn select_icon_empty_level_icons_uses_alert() {
        let result = select_icon(&IconContext {
            percentage: 50.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &[],
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "alert");
    }

    #[test]
    fn select_icon_level_0_percent() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 0.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "battery-empty");
    }

    #[test]
    fn select_icon_level_19_percent() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 19.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "battery-empty");
    }

    #[test]
    fn select_icon_level_20_percent() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 20.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "battery-low");
    }

    #[test]
    fn select_icon_level_50_percent() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 50.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "battery-medium");
    }

    #[test]
    fn select_icon_level_100_percent() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 100.0,
            state: DeviceState::Discharging,
            is_present: true,
            level_icons: &icons,
            charging_icon: "charging",
            alert_icon: "alert",
        });
        assert_eq!(result, "battery-full");
    }

    #[test]
    fn format_label_present() {
        assert_eq!(format_label("{{ percent }}%", 75.0, true), "75%");
        assert_eq!(format_label("{{ percent }}", 100.0, true), "100");
        assert_eq!(format_label("Bat: {{ percent }}", 0.0, true), "Bat: 0");
    }

    #[test]
    fn format_label_not_present() {
        assert_eq!(format_label("{{ percent }}%", 50.0, false), "N/A");
    }

    #[test]
    fn format_label_no_spaces() {
        assert_eq!(format_label("{{percent}}", 75.0, true), "75");
        assert_eq!(format_label("{{percent}}%", 75.0, true), "75%");
    }
}
