use serde_json::json;

pub(crate) struct IconContext<'a> {
    pub(crate) percentage: f64,
    pub(crate) level_icons: &'a [String],
}

pub(crate) fn select_icon(ctx: &IconContext<'_>) -> String {
    if ctx.level_icons.is_empty() {
        return String::new();
    }

    let index = ((ctx.percentage / 100.0) * ctx.level_icons.len() as f64)
        .floor()
        .min((ctx.level_icons.len() - 1) as f64) as usize;

    ctx.level_icons
        .get(index)
        .cloned()
        .unwrap_or_else(|| ctx.level_icons.last().cloned().unwrap_or_default())
}

pub(crate) fn format_label(format: &str, percentage: f64) -> String {
    let ctx = json!({
        "percent": percentage.round() as u32,
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level_icons() -> Vec<String> {
        vec![
            String::from("sun-dim"),
            String::from("sun-medium"),
            String::from("sun"),
        ]
    }

    #[test]
    fn empty_icons_returns_empty_string() {
        let result = select_icon(&IconContext {
            percentage: 50.0,
            level_icons: &[],
        });
        assert_eq!(result, "");
    }

    #[test]
    fn zero_percent_returns_first_icon() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 0.0,
            level_icons: &icons,
        });
        assert_eq!(result, "sun-dim");
    }

    #[test]
    fn low_brightness_returns_first_icon() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 25.0,
            level_icons: &icons,
        });
        assert_eq!(result, "sun-dim");
    }

    #[test]
    fn mid_brightness_returns_second_icon() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 50.0,
            level_icons: &icons,
        });
        assert_eq!(result, "sun-medium");
    }

    #[test]
    fn high_brightness_returns_last_icon() {
        let icons = level_icons();
        let result = select_icon(&IconContext {
            percentage: 100.0,
            level_icons: &icons,
        });
        assert_eq!(result, "sun");
    }

    #[test]
    fn format_label_default() {
        assert_eq!(format_label("{{ percent }}%", 65.0), "65%");
        assert_eq!(format_label("{{ percent }}", 100.0), "100");
        assert_eq!(
            format_label("Brightness: {{ percent }}", 50.0),
            "Brightness: 50"
        );
    }

    #[test]
    fn format_label_rounds_to_nearest_integer() {
        assert_eq!(format_label("{{ percent }}%", 49.6), "50%");
        assert_eq!(format_label("{{ percent }}%", 49.4), "49%");
    }
}
