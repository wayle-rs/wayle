//! A schema marker for a config field that has been REMOVED.
//!
//! A `Removed<T>` field in a config struct — annotated with
//! `#[wayle(deprecated("why it's gone"))]` — keeps the old TOML key recognized so a
//! stale value still present in a user's config is WARNED about at load (rather than
//! silently ignored), while the field itself stores nothing, applies nothing, and is
//! absent from both the JSON schema and serialized output. It replaces hand-maintaining
//! a separate list of removed dotted-paths (which risked typos): the key is derived from
//! the field, and the removal is visible right in the struct.
//!
//! The warning text comes from the field's `#[wayle(deprecated("..."))]` attribute and
//! is emitted by the `ApplyConfigLayer` derive; this type itself is inert.

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, de::IgnoredAny};
use tokio::sync::mpsc;

use super::traits::{
    ApplyConfigLayer, ApplyRuntimeLayer, ClearAllRuntime, ClearRuntimeByPath, CommitConfigReload,
    ExtractRuntimeValues, ResetConfigLayer, ResetRuntimeLayer, SubscribeChanges,
};

/// A removed config field of former type `T`. See the [module docs](self). Give the
/// field a `#[wayle(deprecated("..."))]` attribute (and a `#[serde(rename = "...")]` if
/// the removed key wasn't the field name); exclude it from the schema and serialized
/// output with `#[schemars(skip)]` and `#[serde(skip_serializing)]`.
pub struct Removed<T>(PhantomData<T>);

impl<T> std::fmt::Debug for Removed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Removed")
    }
}

impl<T> Clone for Removed<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Removed<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> Deserialize<'de> for Removed<T> {
    /// Accept and discard whatever value is still present, so deserializing a config
    /// that still carries the removed key never fails. (Config is normally loaded via
    /// [`ApplyConfigLayer`], which is where the deprecation warning fires.)
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        IgnoredAny::deserialize(deserializer)?;
        Ok(Self(PhantomData))
    }
}

// Every config-layer operation is a no-op: a removed field holds no value, applies
// nothing, and persists nothing. `ApplyConfigLayer`'s deprecation warning is generated
// by the derive (from the field's attribute + derived key), not here.

impl<T> ApplyConfigLayer for Removed<T> {
    fn apply_config_layer(&self, _value: &toml::Value, _path: &str) {}
}

impl<T> ApplyRuntimeLayer for Removed<T> {
    fn apply_runtime_layer(&self, _value: &toml::Value, _path: &str) -> Result<(), String> {
        Ok(())
    }
}

impl<T> ExtractRuntimeValues for Removed<T> {
    fn extract_runtime_values(&self) -> Option<toml::Value> {
        None
    }
}

impl<T> SubscribeChanges for Removed<T> {
    fn subscribe_changes(&self, _tx: mpsc::UnboundedSender<()>) {}
}

impl<T> ResetConfigLayer for Removed<T> {
    fn reset_config_layer(&self) {}
}

impl<T> ResetRuntimeLayer for Removed<T> {
    fn reset_runtime_layer(&self) {}
}

impl<T> CommitConfigReload for Removed<T> {
    fn commit_config_reload(&self) {}
}

impl<T> ClearAllRuntime for Removed<T> {
    fn clear_all_runtime(&self) {}
}

impl<T> ClearRuntimeByPath for Removed<T> {
    fn clear_runtime_by_path(&self, _path: &str) -> Result<bool, String> {
        Ok(false)
    }
}
