use relm4::gtk;

/// Fallback icon used when a country flag asset is not installed.
pub(super) const FLAG_FALLBACK: &str = "ld-globe-symbolic";

/// Resolves the flag icon name for an ISO country `code` (e.g. `"se"`).
///
/// Returns the bundled `flag-{code}` icon (registered as a GResource icon path
/// at startup) when present in the icon theme, otherwise a generic globe so the
/// row still renders.
pub(super) fn flag_icon(code: &str) -> String {
    let code = code.trim().to_ascii_lowercase();
    if code.is_empty() {
        return FLAG_FALLBACK.to_string();
    }

    let name = format!("flag-{code}");
    let available = gtk::gdk::Display::default()
        .map(|display| gtk::IconTheme::for_display(&display).has_icon(&name))
        .unwrap_or(false);

    if available {
        name
    } else {
        FLAG_FALLBACK.to_string()
    }
}
