//! The core config property type backing every field in the config schema.

use std::{
    borrow::Cow,
    fmt::Debug,
    sync::{Arc, RwLock},
};

use futures::{Stream, StreamExt};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::mpsc;
use wayle_core::Property;

use super::{
    ApplyConfigLayer, ApplyRuntimeLayer, ClearAllRuntime, ClearRuntimeByPath, CommitConfigReload,
    ExtractRuntimeValues, ResetConfigLayer, ResetRuntimeLayer, SubscribeChanges,
};
use crate::diagnostic::Diagnostic;

fn format_toml_value(value: &toml::Value) -> String {
    toml::to_string_pretty(value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim()
        .to_string()
}

/// Which layer the current effective value comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    /// No config.toml or runtime value; using compiled default.
    Default,
    /// Set in config.toml with no runtime override.
    Config,
    /// Set via GUI/CLI, no config.toml entry exists for this field.
    RuntimeOnly,
    /// config.toml value exists but runtime takes precedence.
    Overridden,
}

/// A config value with three layers: default, config.toml, and runtime override.
///
/// `get()` returns whichever layer has highest precedence (runtime > config > default).
pub struct ConfigProperty<T: Clone + Send + Sync + PartialEq + 'static> {
    default: T,
    config: Arc<RwLock<Option<T>>>,
    runtime: Arc<RwLock<Option<T>>>,
    effective: Property<T>,
    i18n_key: Option<&'static str>,
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ConfigProperty<T> {
    /// Creates a property with the given default value and no i18n key.
    pub fn new(default: T) -> Self {
        let effective = Property::new(default.clone());
        Self {
            default,
            config: Arc::new(RwLock::new(None)),
            runtime: Arc::new(RwLock::new(None)),
            effective,
            i18n_key: None,
        }
    }

    /// Creates a property bound to a Fluent key for the settings GUI label
    /// and description.
    pub fn with_i18n_key(default: T, key: &'static str) -> Self {
        let effective = Property::new(default.clone());
        Self {
            default,
            config: Arc::new(RwLock::new(None)),
            runtime: Arc::new(RwLock::new(None)),
            effective,
            i18n_key: Some(key),
        }
    }

    /// Fluent message ID for the settings GUI, or `None` for internal fields.
    pub fn i18n_key(&self) -> Option<&'static str> {
        self.i18n_key
    }

    /// The effective value after layer resolution (runtime > config > default).
    pub fn get(&self) -> T {
        self.effective.get()
    }

    /// The compiled default, before any config or runtime layers.
    pub fn default(&self) -> &T {
        &self.default
    }

    /// The raw config.toml value, ignoring runtime and default layers.
    pub fn config(&self) -> Option<T> {
        self.config.read().ok().and_then(|guard| guard.clone())
    }

    /// The raw runtime override value, ignoring config and default layers.
    pub fn runtime(&self) -> Option<T> {
        self.runtime.read().ok().and_then(|guard| guard.clone())
    }

    /// Which layer is currently winning.
    pub fn source(&self) -> ValueSource {
        let has_runtime = self
            .runtime
            .read()
            .ok()
            .is_some_and(|guard| guard.is_some());
        let has_config = self.config.read().ok().is_some_and(|guard| guard.is_some());

        match (has_runtime, has_config) {
            (true, true) => ValueSource::Overridden,
            (true, false) => ValueSource::RuntimeOnly,
            (false, true) => ValueSource::Config,
            (false, false) => ValueSource::Default,
        }
    }

    /// Sets a runtime override and immediately notifies watchers.
    pub fn set(&self, value: T) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = Some(value);
        }
        self.flush();
    }

    /// Removes the runtime override and always notifies watchers, even when
    /// the effective value is unchanged.
    pub fn clear_runtime(&self) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = None;
        }
        self.flush_forced();
    }

    /// Writes to the config layer without notifying anyone.
    /// Part of the batch reload cycle; `commit_config_reload` flushes later.
    pub fn stage_config(&self, value: T) {
        if let Ok(mut guard) = self.config.write() {
            *guard = Some(value);
        }
    }

    /// Writes to the runtime layer without notifying anyone.
    /// Part of the batch reload cycle; `commit_config_reload` flushes later.
    pub fn stage_runtime(&self, value: T) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = Some(value);
        }
    }

    /// Stream of effective value changes. Yields the current value
    /// immediately on subscribe, then on every subsequent change.
    pub fn watch(&self) -> impl Stream<Item = T> + Send + 'static {
        self.effective.watch()
    }

    /// Resolves layers and notifies watchers if the result changed.
    fn flush(&self) {
        let effective = self.resolve();
        self.effective.set(effective);
    }

    /// Resolves layers and always notifies watchers, even if unchanged.
    /// Used by `clear_runtime` so PersistenceWatcher saves even when
    /// the runtime value happened to match the config value.
    fn flush_forced(&self) {
        let effective = self.resolve();
        self.effective.replace(effective);
    }

    /// Pure layer resolution: runtime > config > default. No side effects.
    fn resolve(&self) -> T {
        let runtime_value = self.runtime.read().ok().and_then(|guard| guard.clone());
        let config_value = self.config.read().ok().and_then(|guard| guard.clone());

        runtime_value
            .or(config_value)
            .unwrap_or_else(|| self.default.clone())
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> Clone for ConfigProperty<T> {
    fn clone(&self) -> Self {
        Self {
            default: self.default.clone(),
            config: Arc::clone(&self.config),
            runtime: Arc::clone(&self.runtime),
            effective: self.effective.clone(),
            i18n_key: self.i18n_key,
        }
    }
}

impl<T: Clone + Send + Sync + PartialEq + Debug + 'static> Debug for ConfigProperty<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config_value = self.config.read().ok().and_then(|guard| guard.clone());
        let runtime_value = self.runtime.read().ok().and_then(|guard| guard.clone());

        f.debug_struct("ConfigProperty")
            .field("effective", &self.get())
            .field("source", &self.source())
            .field("default", &self.default)
            .field("config", &config_value)
            .field("runtime", &runtime_value)
            .finish()
    }
}

impl<T: Clone + Send + Sync + PartialEq + Serialize + 'static> Serialize for ConfigProperty<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.get().serialize(serializer)
    }
}

impl<'de, T: Clone + Send + Sync + PartialEq + Deserialize<'de> + 'static> Deserialize<'de>
    for ConfigProperty<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;
        Ok(ConfigProperty::new(value))
    }
}

impl<T: Clone + Send + Sync + PartialEq + JsonSchema + 'static> JsonSchema for ConfigProperty<T> {
    fn schema_name() -> Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(gen_param: &mut SchemaGenerator) -> Schema {
        T::json_schema(gen_param)
    }
}

impl<T> ApplyConfigLayer for ConfigProperty<T>
where
    T: Clone + Send + Sync + PartialEq + for<'de> Deserialize<'de> + 'static,
{
    fn apply_config_layer(&self, value: &toml::Value, path: &str) {
        let _span = tracing::warn_span!("config", field = path).entered();
        match T::deserialize(value.clone()) {
            Ok(new_value) => {
                let has_runtime_override = self.runtime().is_some();
                self.stage_config(new_value);

                if has_runtime_override {
                    let diag = Diagnostic::warning("config.toml change ignored")
                        .field("Field", path)
                        .field("Reason", "runtime override active")
                        .hint(format!("wayle config reset {path}"));
                    diag.emit();
                    tracing::info!("{}", diag.to_plain());
                }
            }
            Err(e) => {
                let diag = Diagnostic::error("invalid config value")
                    .field("Field", path)
                    .field("Error", e.to_string().trim())
                    .field("Value", format_toml_value(value));
                diag.emit();
                tracing::info!("{}", diag.to_plain());
            }
        }
    }
}

impl<T> ApplyRuntimeLayer for ConfigProperty<T>
where
    T: Clone + Send + Sync + PartialEq + for<'de> Deserialize<'de> + 'static,
{
    fn apply_runtime_layer(&self, value: &toml::Value, path: &str) -> Result<(), String> {
        let _span = tracing::warn_span!("runtime_config", field = path).entered();
        match T::deserialize(value.clone()) {
            Ok(new_value) => {
                self.stage_runtime(new_value);
                Ok(())
            }
            Err(e) => {
                let diag = Diagnostic::error("invalid runtime value")
                    .field("Field", path)
                    .field("Error", e.to_string().trim())
                    .field("Value", format_toml_value(value));
                diag.emit();
                tracing::info!("{}", diag.to_plain());
                Err(format!("invalid value for '{path}': {e}"))
            }
        }
    }
}

impl<T> ExtractRuntimeValues for ConfigProperty<T>
where
    T: Clone + Send + Sync + PartialEq + Serialize + 'static,
{
    fn extract_runtime_values(&self) -> Option<toml::Value> {
        self.runtime().map(|value| {
            toml::Value::try_from(value).unwrap_or_else(|e| {
                tracing::warn!(error = %e, "cannot serialize runtime value");
                toml::Value::String(String::from("<serialization error>"))
            })
        })
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> SubscribeChanges for ConfigProperty<T> {
    fn subscribe_changes(&self, tx: mpsc::UnboundedSender<()>) {
        let mut watch_stream = self.watch();

        tokio::spawn(async move {
            watch_stream.next().await;

            while watch_stream.next().await.is_some() {
                if tx.send(()).is_err() {
                    break;
                }
            }
        });
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ResetConfigLayer for ConfigProperty<T> {
    fn reset_config_layer(&self) {
        if let Ok(mut guard) = self.config.write() {
            *guard = None;
        }
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ResetRuntimeLayer for ConfigProperty<T> {
    fn reset_runtime_layer(&self) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = None;
        }
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ClearAllRuntime for ConfigProperty<T> {
    fn clear_all_runtime(&self) {
        self.clear_runtime();
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> CommitConfigReload for ConfigProperty<T> {
    fn commit_config_reload(&self) {
        self.flush();
    }
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ClearRuntimeByPath for ConfigProperty<T> {
    fn clear_runtime_by_path(&self, path: &str) -> Result<bool, String> {
        if !path.is_empty() {
            return Err(format!("no nested field at '{path}'"));
        }

        let had_runtime = self.runtime().is_some();
        self.clear_runtime();
        Ok(had_runtime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_default_value() {
        let prop = ConfigProperty::new(42);

        assert_eq!(prop.get(), 42);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn set_config_changes_effective_value() {
        let prop = ConfigProperty::new(10);

        prop.stage_config(20);

        assert_eq!(prop.get(), 20);
        assert_eq!(prop.source(), ValueSource::Config);
        assert_eq!(prop.config(), Some(20));
    }

    #[test]
    fn set_runtime_overrides_config() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);

        prop.set(30);

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Overridden);
        assert_eq!(prop.config(), Some(20));
        assert_eq!(prop.runtime(), Some(30));
    }

    #[test]
    fn set_runtime_without_config_is_custom() {
        let prop = ConfigProperty::new(10);

        prop.set(30);

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::RuntimeOnly);
    }

    #[test]
    fn clear_runtime_falls_back_to_config() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        prop.set(30);

        prop.clear_runtime();

        assert_eq!(prop.get(), 20);
        assert_eq!(prop.source(), ValueSource::Config);
    }

    #[test]
    fn clear_runtime_falls_back_to_default() {
        let prop = ConfigProperty::new(10);
        prop.set(30);

        prop.clear_runtime();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn default_returns_original_default() {
        let prop = ConfigProperty::new(42);
        prop.stage_config(100);
        prop.set(200);

        assert_eq!(*prop.default(), 42);
    }

    #[test]
    fn reset_config_layer_clears_without_recomputing() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        assert_eq!(prop.get(), 20);

        prop.reset_config_layer();

        assert_eq!(prop.config(), None);
        assert_eq!(
            prop.get(),
            20,
            "effective value should NOT change until commit"
        );
    }

    #[test]
    fn commit_config_reload_recomputes_effective() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        prop.reset_config_layer();
        assert_eq!(prop.get(), 20);

        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn reset_apply_commit_reverts_removed_config_to_default() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        assert_eq!(prop.get(), 20);

        prop.reset_config_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn reset_apply_commit_preserves_runtime_override() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        prop.set(30);
        assert_eq!(prop.source(), ValueSource::Overridden);

        prop.reset_config_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::RuntimeOnly);
        assert_eq!(prop.runtime(), Some(30));
        assert_eq!(prop.config(), None);
    }

    #[test]
    fn reset_apply_commit_with_new_config_value() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);

        prop.reset_config_layer();
        prop.stage_config(50);
        prop.commit_config_reload();

        assert_eq!(prop.get(), 50);
        assert_eq!(prop.source(), ValueSource::Config);
    }

    #[test]
    fn reset_runtime_layer_clears_without_recomputing() {
        let prop = ConfigProperty::new(10);
        prop.set(30);
        assert_eq!(prop.get(), 30);

        prop.reset_runtime_layer();

        assert_eq!(prop.runtime(), None);
        assert_eq!(
            prop.get(),
            30,
            "effective value should NOT change until commit"
        );
    }

    #[test]
    fn reset_runtime_then_commit_falls_back_to_config() {
        let prop = ConfigProperty::new(10);
        prop.stage_config(20);
        prop.set(30);
        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Overridden);

        prop.reset_runtime_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 20);
        assert_eq!(prop.source(), ValueSource::Config);
    }

    #[test]
    fn reset_runtime_then_commit_falls_back_to_default() {
        let prop = ConfigProperty::new(10);
        prop.set(30);
        assert_eq!(prop.source(), ValueSource::RuntimeOnly);

        prop.reset_runtime_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }
}
