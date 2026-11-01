//! CSS provider setup and rebuild for the settings window.

use relm4::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk::Display, style_context_add_provider_for_display,
};
use tracing::warn;
use wayle_config::{ConfigService, infrastructure::paths::ConfigPaths};
use wayle_styling::{STATIC_CSS, ensure_user_styles_scaffold, theme_css, user_css};

pub(super) fn load_css(config_service: &ConfigService) -> CssProvider {
    let Some(display) = Display::default() else {
        warn!("no display available, skipping CSS load");
        return CssProvider::new();
    };

    if let Ok(dir) = ConfigPaths::config_dir() {
        ensure_user_styles_scaffold(&dir);
    } else {
        warn!("cannot resolve config dir; user styles disabled");
    }

    let provider = CssProvider::new();
    let css = build_css(config_service);

    provider.load_from_string(&css);
    style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER + 100);

    provider
}

pub(super) fn build_css(config_service: &ConfigService) -> String {
    let config = config_service.config();
    let palette = config.styling.palette();
    let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);

    let user = match ConfigPaths::config_dir() {
        Ok(dir) => user_css(&dir),
        Err(err) => {
            warn!(error = %err, "cannot resolve config dir; user styles disabled");
            String::new()
        }
    };

    format!("{STATIC_CSS}\n{theme}\n{user}")
}
