//! Lets `SettingRow` work with any config property without knowing its type.
//! Wraps source queries, reset, display formatting, and change watching
//! behind boxed closures so the row doesn't need generics.

use std::sync::Arc;

use wayle_config::{ConfigProperty, ValueSource};

use crate::editors::{WatcherHandle, spawn_property_watcher};

pub(crate) type WatchCallback = Box<dyn FnOnce(Box<dyn Fn() -> bool + 'static>) -> WatcherHandle>;

pub(crate) struct PropertyHandle {
    pub(crate) source: Box<dyn Fn() -> ValueSource>,
    pub(crate) clear_runtime: Box<dyn Fn()>,
    pub(crate) config_display: Box<dyn Fn() -> Option<String>>,
    pub(crate) default_display: Box<dyn Fn() -> String>,
    pub(crate) watch_changes: Option<WatchCallback>,
}

impl PropertyHandle {
    pub(crate) fn new<T, D>(property: &ConfigProperty<T>, display_fn: D) -> Self
    where
        T: Clone + Send + Sync + PartialEq + 'static,
        D: Fn(&T) -> String + 'static,
    {
        let prop = Arc::new(property.clone());
        let display_fn = Arc::new(display_fn);

        let source_prop = Arc::clone(&prop);
        let clear_prop = Arc::clone(&prop);
        let config_prop = Arc::clone(&prop);
        let default_prop = Arc::clone(&prop);
        let config_display_fn = Arc::clone(&display_fn);
        let default_display_fn = Arc::clone(&display_fn);
        let watch_prop = Arc::clone(&prop);

        Self {
            source: Box::new(move || source_prop.source()),
            clear_runtime: Box::new(move || clear_prop.clear_runtime()),

            config_display: Box::new(move || {
                config_prop.config().map(|value| config_display_fn(&value))
            }),

            default_display: Box::new(move || default_display_fn(default_prop.default())),

            watch_changes: Some(Box::new(move |callback| {
                spawn_property_watcher(&watch_prop, callback)
            })),
        }
    }

    pub(crate) fn source(&self) -> ValueSource {
        (self.source)()
    }

    pub(crate) fn clear_runtime(&self) {
        (self.clear_runtime)()
    }

    pub(crate) fn config_display(&self) -> Option<String> {
        (self.config_display)()
    }

    pub(crate) fn default_display(&self) -> String {
        (self.default_display)()
    }
}
