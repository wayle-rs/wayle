use chrono::{DateTime, Utc};
use relm4::gtk::{gdk, glib, pango};
use wayle_config::schemas::modules::notification::{IconSource, UrgencyBarThreshold};
use wayle_notification::types::Urgency;

use crate::shell::bar::icons::lookup_app_icon;

const FALLBACK_ICON: &str = "ld-bell-symbolic";
const MINUTES_PER_HOUR: i64 = 60;

/// Some apps send notification bodies with bare `&` or other broken XML
/// that Pango chokes on. If the text parses cleanly we leave it alone,
/// otherwise we escape the whole thing so the label at least shows
/// something instead of blowing up.
pub(crate) fn sanitize_markup(text: &str) -> String {
    if pango::parse_markup(text, '\0').is_ok() {
        return text.to_owned();
    }

    glib::markup_escape_text(text).into()
}

/// Resolved notification icon.
#[derive(Debug, Clone)]
pub(crate) enum ResolvedIcon {
    /// GTK icon theme name.
    Named(String),
    /// Filesystem path to an image file.
    File(String),
}

/// Returns the CSS class name for a notification's urgency level.
pub(crate) fn urgency_css_class(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Low => "low",
        Urgency::Normal => "normal",
        Urgency::Critical => "critical",
    }
}

/// Whether the urgency bar should be visible for the given urgency and threshold.
pub(super) fn urgency_bar_visible(urgency: Urgency, threshold: UrgencyBarThreshold) -> bool {
    match threshold {
        UrgencyBarThreshold::None => false,
        UrgencyBarThreshold::Critical => urgency as u8 >= Urgency::Critical as u8,
        UrgencyBarThreshold::Normal => urgency as u8 >= Urgency::Normal as u8,
        UrgencyBarThreshold::Low => true,
    }
}

/// Time elapsed since a notification was created.
#[derive(Debug)]
pub(crate) enum RelativeTime {
    JustNow,
    Minutes(i64),
    Hours(i64),
}

/// Computes the relative time from a timestamp to now.
pub(crate) fn relative_time(timestamp: &DateTime<Utc>) -> RelativeTime {
    let duration = Utc::now().signed_duration_since(timestamp);
    let minutes = duration.num_minutes();

    if minutes < 1 {
        RelativeTime::JustNow
    } else if minutes < MINUTES_PER_HOUR {
        RelativeTime::Minutes(minutes)
    } else {
        RelativeTime::Hours(duration.num_hours())
    }
}

/// Resolves the notification icon based on the configured source mode.
pub(crate) fn resolve_icon(
    icon_source: IconSource,
    app_name: &Option<String>,
    app_icon: &Option<String>,
    image_path: &Option<String>,
    desktop_entry: &Option<String>,
) -> ResolvedIcon {
    match icon_source {
        IconSource::Mapped => mapped_icon(app_name),

        IconSource::Automatic => {
            if let Some(resolved) = try_icon_string(image_path) {
                return resolved;
            }

            mapped_icon(app_name)
        }

        IconSource::Application => {
            if let Some(resolved) = try_icon_string(image_path) {
                return resolved;
            }

            if let Some(resolved) = try_icon_string(app_icon) {
                return resolved;
            }

            if let Some(entry) = desktop_entry
                && !entry.is_empty()
            {
                return ResolvedIcon::Named(entry.clone());
            }

            mapped_icon(app_name)
        }
    }
}

/// Classifies a non-empty icon string as either a file path or theme icon name.
fn try_icon_string(value: &Option<String>) -> Option<ResolvedIcon> {
    let icon = value.as_deref().filter(|raw| !raw.is_empty())?;

    if let Some(path) = icon.strip_prefix("file://") {
        Some(ResolvedIcon::File(path.to_owned()))
    } else if icon.starts_with('/') {
        Some(ResolvedIcon::File(icon.to_owned()))
    } else {
        Some(ResolvedIcon::Named(icon.to_owned()))
    }
}

fn mapped_icon(app_name: &Option<String>) -> ResolvedIcon {
    let name = app_name
        .as_deref()
        .and_then(lookup_app_icon)
        .unwrap_or(FALLBACK_ICON);

    ResolvedIcon::Named(String::from(name))
}

/// Loads a file-based icon as a scaled texture to avoid keeping oversized
/// image allocations alive when notifications provide large images.
pub(crate) fn load_scaled_file_icon(path: &str, target_px: i32) -> Option<gdk::Texture> {
    if path.is_empty() {
        return None;
    }

    let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(path, target_px, target_px, true).ok()?;
    Some(gdk::Texture::for_pixbuf(&pixbuf))
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn urgency_css_class_maps_all_levels() {
        assert_eq!(urgency_css_class(Urgency::Low), "low");
        assert_eq!(urgency_css_class(Urgency::Normal), "normal");
        assert_eq!(urgency_css_class(Urgency::Critical), "critical");
    }

    #[test]
    fn urgency_bar_none_always_hidden() {
        assert!(!urgency_bar_visible(
            Urgency::Low,
            UrgencyBarThreshold::None
        ));
        assert!(!urgency_bar_visible(
            Urgency::Normal,
            UrgencyBarThreshold::None
        ));
        assert!(!urgency_bar_visible(
            Urgency::Critical,
            UrgencyBarThreshold::None
        ));
    }

    #[test]
    fn urgency_bar_low_always_visible() {
        assert!(urgency_bar_visible(Urgency::Low, UrgencyBarThreshold::Low));
        assert!(urgency_bar_visible(
            Urgency::Normal,
            UrgencyBarThreshold::Low
        ));
        assert!(urgency_bar_visible(
            Urgency::Critical,
            UrgencyBarThreshold::Low
        ));
    }

    #[test]
    fn urgency_bar_normal_hides_low() {
        assert!(!urgency_bar_visible(
            Urgency::Low,
            UrgencyBarThreshold::Normal
        ));
        assert!(urgency_bar_visible(
            Urgency::Normal,
            UrgencyBarThreshold::Normal
        ));
        assert!(urgency_bar_visible(
            Urgency::Critical,
            UrgencyBarThreshold::Normal
        ));
    }

    #[test]
    fn urgency_bar_critical_only_shows_critical() {
        assert!(!urgency_bar_visible(
            Urgency::Low,
            UrgencyBarThreshold::Critical
        ));
        assert!(!urgency_bar_visible(
            Urgency::Normal,
            UrgencyBarThreshold::Critical
        ));
        assert!(urgency_bar_visible(
            Urgency::Critical,
            UrgencyBarThreshold::Critical
        ));
    }

    #[test]
    fn relative_time_just_now() {
        let now = Utc::now();
        assert!(matches!(relative_time(&now), RelativeTime::JustNow));
    }

    #[test]
    fn relative_time_minutes() {
        let thirty_min_ago = Utc::now() - chrono::Duration::minutes(30);
        let result = relative_time(&thirty_min_ago);

        let RelativeTime::Minutes(minutes) = result else {
            panic!("expected Minutes, got {result:?}");
        };
        assert!((29..=31).contains(&minutes));
    }

    #[test]
    fn relative_time_hours() {
        let two_hours_ago = Utc::now() - chrono::Duration::hours(2);
        let result = relative_time(&two_hours_ago);

        let RelativeTime::Hours(hours) = result else {
            panic!("expected Hours, got {result:?}");
        };
        assert_eq!(hours, 2);
    }

    #[test]
    fn try_icon_string_none_returns_none() {
        assert!(try_icon_string(&None).is_none());
    }

    #[test]
    fn try_icon_string_empty_returns_none() {
        assert!(try_icon_string(&Some(String::new())).is_none());
    }

    #[test]
    fn try_icon_string_file_uri() {
        let result = try_icon_string(&Some("file:///usr/share/icon.png".into()));
        assert!(matches!(result, Some(ResolvedIcon::File(path)) if path == "/usr/share/icon.png"));
    }

    #[test]
    fn try_icon_string_absolute_path() {
        let result = try_icon_string(&Some("/usr/share/icon.png".into()));
        assert!(matches!(result, Some(ResolvedIcon::File(path)) if path == "/usr/share/icon.png"));
    }

    #[test]
    fn try_icon_string_theme_name() {
        let result = try_icon_string(&Some("firefox".into()));
        assert!(matches!(result, Some(ResolvedIcon::Named(name)) if name == "firefox"));
    }

    #[test]
    fn sanitize_markup_preserves_valid_markup() {
        let valid = "<b>bold</b> and <i>italic</i>";
        assert_eq!(sanitize_markup(valid), valid);
    }

    #[test]
    fn sanitize_markup_escapes_bare_ampersand() {
        let raw = "NixOS Package & Module";
        assert_eq!(sanitize_markup(raw), "NixOS Package &amp; Module");
    }

    #[test]
    fn sanitize_markup_passes_plain_text() {
        let plain = "Hello world";
        assert_eq!(sanitize_markup(plain), plain);
    }

    #[test]
    fn sanitize_markup_escapes_mixed_invalid() {
        let raw = "<b>bold</b> & more";
        assert_eq!(sanitize_markup(raw), "&lt;b&gt;bold&lt;/b&gt; &amp; more");
    }
}
