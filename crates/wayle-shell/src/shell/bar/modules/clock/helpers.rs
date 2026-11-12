use gtk4::glib::DateTime;
use tracing::error;

pub(super) fn format_time(format: &str) -> String {
    DateTime::now_local()
        .and_then(|dt| dt.format(format))
        .map(|gstring| gstring.to_string())
        .inspect_err(|e| error!(error = %e, "cannot format time"))
        .unwrap_or_else(|_| String::from("--"))
}

#[cfg(test)]
mod tests {
    use gtk4::glib::DateTime;

    #[test]
    fn unpadded_hour_drops_leading_zero() {
        // GLib's `%-` modifier suppresses zero-padding; minutes stay padded.
        let dt = DateTime::from_local(2024, 1, 15, 9, 5, 0.0).expect("valid datetime");
        assert_eq!(dt.format("%-I:%M").expect("format").to_string(), "9:05");
        assert_eq!(dt.format("%-H:%M").expect("format").to_string(), "9:05");
    }
}
