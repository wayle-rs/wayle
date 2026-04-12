//! CSS provider setup and rebuild for the settings window.

use relm4::gtk::{
    CssProvider, STYLE_PROVIDER_PRIORITY_USER, gdk::Display, style_context_add_provider_for_display,
};
use tracing::warn;
use wayle_config::ConfigService;
use wayle_styling::{STATIC_CSS, theme_css};

pub(super) fn load_css(config_service: &ConfigService) -> CssProvider {
    let Some(display) = Display::default() else {
        warn!("no display available, skipping CSS load");
        return CssProvider::new();
    };

    let provider = CssProvider::new();
    let css = build_css(config_service);

    provider.load_from_string(&css);
    style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER);

    provider
}

pub(super) fn build_css(config_service: &ConfigService) -> String {
    let config = config_service.config();
    let palette = config.styling.palette();
    let theme = theme_css(&palette, &config.general, &config.bar, &config.styling);

    format!("{STATIC_CSS}\n{theme}")
}
