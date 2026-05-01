//! Background tasks that watch for config changes and external signals,
//! forwarding CSS reload requests to the app.

mod palette;
mod scss_dev;
mod theme;

pub(crate) use palette::spawn as spawn_palette_watcher;
pub(crate) use scss_dev::spawn as spawn_scss_dev_watcher;
pub(crate) use theme::spawn as spawn_theme_watcher;
