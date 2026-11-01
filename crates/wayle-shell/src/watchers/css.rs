//! CSS hot-reload watcher.

use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use relm4::ComponentSender;
use tracing::warn;
use wayle_config::{infrastructure::paths::ConfigPaths, schemas::styling::ThemeProvider};
use wayle_styling::{STATIC_CSS, theme_css, user_css};
use wayle_widgets::{watch, watchers::changes_stream};

use crate::shell::{Shell, ShellCmd, ShellInput, ShellServices};

/// Spawns the CSS hot-reload watcher.
///
/// Watches styling config properties and color extraction events. Regenerates
/// theme CSS when config changes or after color extraction completes.
pub fn spawn(sender: &ComponentSender<Shell>, services: &ShellServices) {
    let config = services.config.config().clone();

    let css = build_css(&config);
    sender.input_sender().send(ShellInput::ReloadCss(css)).ok();

    let palette_stream = changes_stream(&config.styling.palette);
    let general_stream = changes_stream(&config.general);
    let bar_stream = changes_stream(&config.bar);
    let global_scale_stream = config.styling.scale.watch();
    let global_rounding_stream = config.styling.rounding.watch();

    let theme_provider_stream = config
        .styling
        .theme_provider
        .watch()
        .filter(|provider| std::future::ready(*provider == ThemeProvider::Wayle));

    let extraction_stream: BoxStream<'static, ()> = match &services.wallpaper {
        Some(ws) => ws.watch_extraction().boxed(),
        None => stream::pending().boxed(),
    };

    let config_clone = config.clone();
    watch!(sender,
        [
            palette_stream,
            general_stream,
            bar_stream,
            global_scale_stream,
            global_rounding_stream,
            theme_provider_stream,
            extraction_stream,
        ],
        move || Ok::<_, std::convert::Infallible>(build_css(&config_clone)) => ShellCmd::CssRecompiled
    );
}

pub(super) fn build_css(config: &wayle_config::Config) -> String {
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
