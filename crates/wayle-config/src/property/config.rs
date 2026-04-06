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
    ApplyConfigLayer, ApplyRuntimeLayer, ClearRuntimeByPath, CommitConfigReload,
    ExtractRuntimeValues, ResetConfigLayer, ResetRuntimeLayer, SubscribeChanges,
};
use crate::diagnostic::Diagnostic;

fn format_toml_value(value: &toml::Value) -> String {
    toml::to_string_pretty(value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim()
        .to_string()
}

/// Indicates where a configuration value originates from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSource {
    /// Using the compiled default value.
    Default,
    /// Set in config.toml (user's base configuration).
    Config,
    /// Changed via GUI, not present in config.toml.
    Custom,
    /// GUI override of a config.toml value.
    Override,
}

/// A layered configuration property with provenance tracking.
///
/// Wraps a reactive `Property<T>` and adds three-layer configuration support:
/// - **Default**: Compiled-in default value
/// - **Config**: Value from config.toml (user's base configuration)
/// - **Runtime**: GUI overrides (stored in runtime.toml)
///
/// The effective value follows precedence: runtime > config > default.
pub struct ConfigProperty<T: Clone + Send + Sync + PartialEq + 'static> {
    default: T,
    config: Arc<RwLock<Option<T>>>,
    runtime: Arc<RwLock<Option<T>>>,
    effective: Property<T>,
    i18n_key: Option<&'static str>,
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ConfigProperty<T> {
    /// Creates a new ConfigProperty with the given default value.
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

    /// Creates a ConfigProperty with a fluent i18n key for the settings GUI.
    /// The key resolves to a label and `.description` in the FTL bundle.
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

    /// The fluent message ID for this property's settings label, if any.
    /// Returns `None` for properties created with `new()` or marked
    /// `#[i18n(skip)]`.
    pub fn i18n_key(&self) -> Option<&'static str> {
        self.i18n_key
    }

    /// Returns the effective (currently active) value.
    ///
    /// Precedence: runtime > config > default.
    pub fn get(&self) -> T {
        self.effective.get()
    }

    /// Returns the compiled default value.
    pub fn default(&self) -> &T {
        &self.default
    }

    /// Returns the config.toml value, if set.
    pub fn config(&self) -> Option<T> {
        self.config.read().ok().and_then(|guard| guard.clone())
    }

    /// Returns the runtime override value, if set.
    pub fn runtime(&self) -> Option<T> {
        self.runtime.read().ok().and_then(|guard| guard.clone())
    }

    /// Returns where the current effective value originates from.
    pub fn source(&self) -> ValueSource {
        let has_runtime = self
            .runtime
            .read()
            .ok()
            .is_some_and(|guard| guard.is_some());
        let has_config = self.config.read().ok().is_some_and(|guard| guard.is_some());

        match (has_runtime, has_config) {
            (true, true) => ValueSource::Override,
            (true, false) => ValueSource::Custom,
            (false, true) => ValueSource::Config,
            (false, false) => ValueSource::Default,
        }
    }

    /// Sets a runtime override value (used by GUI).
    ///
    /// This value takes precedence over config.toml and default values.
    pub fn set(&self, value: T) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = Some(value);
        }
        self.recompute_effective();
    }

    /// Clears the runtime override, falling back to config or default.
    pub fn clear_runtime(&self) {
        if let Ok(mut guard) = self.runtime.write() {
            *guard = None;
        }
        self.recompute_effective();
    }

    /// Sets the config.toml value (used during config loading).
    pub fn set_config(&self, value: T) {
        if let Ok(mut guard) = self.config.write() {
            *guard = Some(value);
        }
        self.recompute_effective();
    }

    /// Clears the config value (rarely needed).
    pub fn clear_config(&self) {
        if let Ok(mut guard) = self.config.write() {
            *guard = None;
        }
        self.recompute_effective();
    }

    /// Watch for changes to the effective value.
    ///
    /// The stream immediately yields the current value, then yields
    /// whenever the effective value changes.
    pub fn watch(&self) -> impl Stream<Item = T> + Send + 'static {
        self.effective.watch()
    }

    fn recompute_effective(&self) {
        let runtime_value = self.runtime.read().ok().and_then(|guard| guard.clone());
        let config_value = self.config.read().ok().and_then(|guard| guard.clone());

        let effective = runtime_value
            .or(config_value)
            .unwrap_or_else(|| self.default.clone());

        self.effective.set(effective);
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
                self.set_config(new_value);

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
                self.set(new_value);
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

impl<T: Clone + Send + Sync + PartialEq + 'static> CommitConfigReload for ConfigProperty<T> {
    fn commit_config_reload(&self) {
        self.recompute_effective();
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

        prop.set_config(20);

        assert_eq!(prop.get(), 20);
        assert_eq!(prop.source(), ValueSource::Config);
        assert_eq!(prop.config(), Some(20));
    }

    #[test]
    fn set_runtime_overrides_config() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);

        prop.set(30);

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Override);
        assert_eq!(prop.config(), Some(20));
        assert_eq!(prop.runtime(), Some(30));
    }

    #[test]
    fn set_runtime_without_config_is_custom() {
        let prop = ConfigProperty::new(10);

        prop.set(30);

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Custom);
    }

    #[test]
    fn clear_runtime_falls_back_to_config() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);
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
        prop.set_config(100);
        prop.set(200);

        assert_eq!(*prop.default(), 42);
    }

    #[test]
    fn reset_config_layer_clears_without_recomputing() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);
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
        prop.set_config(20);
        prop.reset_config_layer();
        assert_eq!(prop.get(), 20);

        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn reset_apply_commit_reverts_removed_config_to_default() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);
        assert_eq!(prop.get(), 20);

        prop.reset_config_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }

    #[test]
    fn reset_apply_commit_preserves_runtime_override() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);
        prop.set(30);
        assert_eq!(prop.source(), ValueSource::Override);

        prop.reset_config_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Custom);
        assert_eq!(prop.runtime(), Some(30));
        assert_eq!(prop.config(), None);
    }

    #[test]
    fn reset_apply_commit_with_new_config_value() {
        let prop = ConfigProperty::new(10);
        prop.set_config(20);

        prop.reset_config_layer();
        prop.set_config(50);
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
        prop.set_config(20);
        prop.set(30);
        assert_eq!(prop.get(), 30);
        assert_eq!(prop.source(), ValueSource::Override);

        prop.reset_runtime_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 20);
        assert_eq!(prop.source(), ValueSource::Config);
    }

    #[test]
    fn reset_runtime_then_commit_falls_back_to_default() {
        let prop = ConfigProperty::new(10);
        prop.set(30);
        assert_eq!(prop.source(), ValueSource::Custom);

        prop.reset_runtime_layer();
        prop.commit_config_reload();

        assert_eq!(prop.get(), 10);
        assert_eq!(prop.source(), ValueSource::Default);
    }
}
