//! One control per config type: toggle, dropdown, slider, spin button, font picker.
//! Each owns its `ConfigProperty` and writes back on user interaction.

use futures::StreamExt;
use wayle_config::ConfigProperty;

pub mod enum_select;
pub mod font;
pub mod number;
pub mod slider;
pub mod toggle;

/// Messages emitted by control components to their parent.
#[derive(Debug)]
pub enum ControlOutput {
    ValueChanged,
}

pub(super) fn spawn_property_watcher<T: Clone + Send + Sync + PartialEq + 'static>(
    property: &ConfigProperty<T>,
    callback: impl Fn() + 'static,
) {
    let mut stream = property.watch();

    gtk4::glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            callback();
        }
    });
}
