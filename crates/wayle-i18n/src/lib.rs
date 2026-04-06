//! i18n lookups for Wayle, backed by embedded Fluent (FTL) files.
//!
//! ```ignore
//! use wayle_i18n::{t, t_attr};
//!
//! let text = t("app-name");
//! let desc = t_attr("app-name", "description");
//! ```

use std::sync::OnceLock;

use i18n_embed::{
    DesktopLanguageRequester, LanguageLoader,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "locales/"]
struct Localizations;

static LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();

/// Lazy-initialized Fluent loader. Detects system locale on first call, falls back to en-US.
///
/// # Panics
///
/// Panics if embedded FTL resources fail to load.
#[allow(clippy::expect_used)]
pub fn loader() -> &'static FluentLanguageLoader {
    LOADER.get_or_init(|| {
        let loader = fluent_language_loader!();
        loader
            .load_fallback_language(&Localizations)
            .expect("embedded FTL resources are valid");

        let requested = DesktopLanguageRequester::requested_languages();
        let _ = i18n_embed::select(&loader, &Localizations, &requested);

        loader
    })
}

/// Returns the localized string for `key`, or the key itself if no FTL entry exists.
pub fn t(key: &str) -> String {
    loader().get(key)
}

/// Returns a localized attribute value, or the attribute path if no FTL entry exists.
pub fn t_attr(key: &str, attr: &str) -> String {
    loader().get_attr(key, attr)
}

#[cfg(test)]
mod tests {
    use super::t;

    #[test]
    fn keys_from_both_files_work() {
        let _ = t("app-name");
        let _ = t("settings-bar-scale");
    }
}
