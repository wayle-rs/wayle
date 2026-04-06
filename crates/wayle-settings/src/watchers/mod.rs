//! Background tasks that watch for config changes and external signals,
//! forwarding CSS reload requests to the app.

mod palette;
mod scss_dev;
mod theme;

pub use palette::spawn_palette_watcher;
pub use scss_dev::spawn as spawn_scss_dev_watcher;
pub use theme::spawn as spawn_theme_watcher;
