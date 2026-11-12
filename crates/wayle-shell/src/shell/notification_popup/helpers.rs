use std::sync::Arc;

use chrono::{DateTime, Utc};
use relm4::gtk::{gdk, gio, glib, pango, prelude::*};
use wayle_config::schemas::modules::notification::{IconSource, UrgencyBarThreshold};
use wayle_notification::{
    core::{notification::Notification, types::Image},
    types::Priority,
};

use crate::shell::bar::icons::lookup_app_icon;

const FALLBACK_ICON: &str = "ld-bell-symbolic";
const MINUTES_PER_HOUR: i64 = 60;

/// Some apps send notification bodies with bare `&` or other broken XML
/// that Pango chokes on. If the text parses cleanly we leave it alone,
/// otherwise we escape the whole thing so the label at least shows
/// something instead of blowing up.
///
/// `<a href>` hyperlinks are a `GtkLabel` markup extension: the label strips the links itself
/// before handing the remaining markup to Pango, so bare `pango::parse_markup` does NOT know
/// the `<a>` tag and would reject (and thus escape) a perfectly valid link body. We therefore
/// validate the *link-stripped* form — matching what `GtkLabel` actually renders — so a body
/// with `<a href>` links is kept and rendered as clickable links rather than shown raw.
pub(crate) fn sanitize_markup(text: &str) -> String {
    if pango::parse_markup(&strip_anchor_tags(text), '\0').is_ok() {
        return text.to_owned();
    }

    glib::markup_escape_text(text).into()
}

/// Removes `<a ...>` / `</a>` hyperlink tags for the markup-validation pass only (the original
/// text, links intact, is what gets rendered). `GtkLabel` parses `<a>` itself and passes the
/// rest to Pango, so this mirrors its validation; other tags (`<b>`, `<span>`, …) are left for
/// Pango to check. A malformed/unterminated `<a` is left in place so validation still fails and
/// the body is escaped.
fn strip_anchor_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find('<') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let is_open = tail
            .strip_prefix("<a")
            .is_some_and(|after| after.starts_with(|c: char| c == '>' || c.is_whitespace()));
        let is_close = tail.starts_with("</a>");
        if is_open || is_close {
            match tail.find('>') {
                Some(gt) => rest = &tail[gt + 1..],
                None => {
                    // Unterminated `<a`: keep it so Pango validation fails → whole body escaped.
                    out.push_str(tail);
                    return out;
                }
            }
        } else {
            // A different tag (e.g. `<b>`) or a literal `<`: keep it and keep scanning.
            out.push('<');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

/// Prepares a notification body for a markup-enabled label. When the body is markup
/// (freedesktop, portal `markup-body`) it is sanitized (kept if valid, else escaped);
/// when it is plain text (GNotification, portal plain `body`) it is escaped so any `<`/`&`
/// render literally instead of being parsed as markup.
pub(crate) fn render_body(text: &str, markup: bool) -> String {
    if markup {
        sanitize_markup(text)
    } else {
        glib::markup_escape_text(text).into()
    }
}

/// Mints an `xdg-activation-v1` token for the current click, so an app invoked by a
/// notification action may raise its window past the compositor's focus-stealing prevention.
///
/// Uses GDK's `AppLaunchContext`, which performs the Wayland `xdg_activation_token_v1` dance
/// internally (the headless notification daemon can't — it has no Wayland connection, so the
/// shell mints and passes the token in). Returns `None` if there's no display or the compositor
/// declines (e.g. the shell's layer-shell surface doesn't currently hold focus); the action
/// still dispatches, the window just may not auto-raise.
pub(crate) fn mint_activation_token() -> Option<String> {
    let context = gdk::Display::default()?.app_launch_context();
    context
        .startup_notify_id(None::<&gio::AppInfo>, &[])
        .map(|token| token.to_string())
}

/// Handles a click on an `<a href>` link in a notification body. Wire to a body label's
/// `connect_activate_link`: it opens the URI through the notification's XDG portal connection
/// (via [`Notification::open_uri`](wayle_notification::core::notification::Notification::open_uri))
/// and returns [`glib::Propagation::Stop`] so GTK's default handler — which calls `gtk_show_uri`
/// on our layer-shell surface and crashes with a Wayland protocol error — never runs.
pub(crate) fn open_body_link(notification: &Arc<Notification>, uri: &str) -> glib::Propagation {
    let notification = notification.clone();
    let uri = uri.to_owned();
    relm4::spawn_local(async move {
        if let Err(err) = notification.open_uri(&uri).await {
            tracing::warn!(uri = %uri, error = %err, "opening notification body link failed");
        }
    });
    glib::Propagation::Stop
}

/// Resolved notification icon.
#[derive(Debug, Clone)]
pub(crate) enum ResolvedIcon {
    /// GTK icon theme name.
    Named(String),
    /// Filesystem path to an image file.
    File(String),
}

/// Returns the CSS class name for a notification's priority level.
pub(crate) fn priority_css_class(priority: Priority) -> &'static str {
    match priority {
        Priority::Low => "low",
        Priority::Normal => "normal",
        Priority::High => "high",
        Priority::Urgent => "urgent",
    }
}

/// Whether the priority bar should be visible for the given priority and threshold.
///
/// The configured [`UrgencyBarThreshold`] is the *minimum* level that shows the bar.
/// `Critical` maps to the top level (`Urgent`), so `High` shows a bar under the `Normal`
/// and `Low` thresholds but not under `Critical`.
pub(super) fn priority_bar_visible(priority: Priority, threshold: UrgencyBarThreshold) -> bool {
    match threshold {
        UrgencyBarThreshold::None => false,
        UrgencyBarThreshold::Critical => priority as u8 >= Priority::Urgent as u8,
        UrgencyBarThreshold::Normal => priority as u8 >= Priority::Normal as u8,
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

/// Flattens an [`Image`] facet back to the string form [`resolve_icon`] classifies. A themed
/// name and a file path both round-trip: the resolver re-classifies the string identically.
fn image_to_string(image: Option<Image>) -> Option<String> {
    image.map(|image| match image {
        Image::Named(name) => name,
        Image::Path(path) => path.display().to_string(),
    })
}

/// Resolves a notification's icon straight from its typed facets, so call sites never unpack
/// the `Origin`/`Image` shape themselves.
pub(crate) fn resolve_notification_icon(
    icon_source: IconSource,
    notification: &Notification,
) -> ResolvedIcon {
    let origin = notification.view.get().origin;
    let name = origin.name;
    let icon = image_to_string(origin.icon);
    let desktop_entry = origin.desktop_entry.map(|entry| entry.as_str().to_owned());
    let image = image_to_string(notification.view.get().image);
    resolve_icon(icon_source, &name, &icon, &image, &desktop_entry)
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
    fn priority_css_class_maps_all_levels() {
        assert_eq!(priority_css_class(Priority::Low), "low");
        assert_eq!(priority_css_class(Priority::Normal), "normal");
        assert_eq!(priority_css_class(Priority::High), "high");
        assert_eq!(priority_css_class(Priority::Urgent), "urgent");
    }

    #[test]
    fn priority_bar_none_always_hidden() {
        for priority in [
            Priority::Low,
            Priority::Normal,
            Priority::High,
            Priority::Urgent,
        ] {
            assert!(!priority_bar_visible(priority, UrgencyBarThreshold::None));
        }
    }

    #[test]
    fn priority_bar_low_always_visible() {
        for priority in [
            Priority::Low,
            Priority::Normal,
            Priority::High,
            Priority::Urgent,
        ] {
            assert!(priority_bar_visible(priority, UrgencyBarThreshold::Low));
        }
    }

    #[test]
    fn priority_bar_normal_hides_low() {
        assert!(!priority_bar_visible(
            Priority::Low,
            UrgencyBarThreshold::Normal
        ));
        assert!(priority_bar_visible(
            Priority::Normal,
            UrgencyBarThreshold::Normal
        ));
        assert!(priority_bar_visible(
            Priority::High,
            UrgencyBarThreshold::Normal
        ));
        assert!(priority_bar_visible(
            Priority::Urgent,
            UrgencyBarThreshold::Normal
        ));
    }

    #[test]
    fn priority_bar_critical_shows_only_urgent() {
        assert!(!priority_bar_visible(
            Priority::Low,
            UrgencyBarThreshold::Critical
        ));
        assert!(!priority_bar_visible(
            Priority::Normal,
            UrgencyBarThreshold::Critical
        ));
        assert!(!priority_bar_visible(
            Priority::High,
            UrgencyBarThreshold::Critical
        ));
        assert!(priority_bar_visible(
            Priority::Urgent,
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

    #[test]
    fn sanitize_markup_keeps_hyperlink() {
        // `<a href>` is a GtkLabel extension Pango doesn't know; it must NOT be escaped.
        let link = r#"See the <a href="https://example.com">release notes</a>"#;
        assert_eq!(sanitize_markup(link), link);
    }

    #[test]
    fn sanitize_markup_keeps_hyperlink_mixed_with_other_tags() {
        let mixed = r#"<b>Update</b> — <a href="https://x.y">details</a>"#;
        assert_eq!(sanitize_markup(mixed), mixed);
    }

    #[test]
    fn sanitize_markup_escapes_body_with_link_and_bare_ampersand() {
        // A real link but a bare `&` elsewhere ⇒ invalid overall ⇒ escape everything,
        // including the link markup (shown literally rather than half-rendered).
        let out = sanitize_markup(r#"A & B <a href="https://x.y">link</a>"#);
        assert!(out.contains("&amp;"), "bare & escaped: {out}");
        assert!(out.contains("&lt;a href"), "link markup escaped too: {out}");
        assert!(!out.contains("<a href"), "no raw tag remains: {out}");
    }

    #[test]
    fn sanitize_markup_escapes_unterminated_anchor() {
        // A stray `<a` that isn't a real tag must still be escaped, not kept.
        assert_eq!(sanitize_markup("click <a here"), "click &lt;a here");
    }
}
