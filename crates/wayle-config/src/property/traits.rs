use tokio::sync::mpsc;

/// Applies TOML values to the config layer of ConfigProperty fields.
///
/// Used when loading or hot-reloading config.toml. The config layer sits
/// between defaults and runtime overrides in precedence.
pub trait ApplyConfigLayer {
    /// Apply TOML values to the config layer.
    ///
    /// The `path` parameter indicates the config key path (e.g., "general.bar.layout")
    /// for error messages.
    ///
    /// Missing fields are skipped. Deserialization failures are logged
    /// and skipped, allowing partial updates to succeed.
    fn apply_config_layer(&self, value: &toml::Value, path: &str);
}

/// Applies TOML values to the runtime layer of ConfigProperty fields.
///
/// Used when loading runtime.toml (GUI overrides). The runtime layer
/// has highest precedence, overriding both config and default values.
pub trait ApplyRuntimeLayer {
    /// Apply TOML values to the runtime layer.
    ///
    /// The `path` parameter indicates the config key path (e.g., "general.bar.layout")
    /// for error messages. Missing fields in nested structs are skipped.
    ///
    /// # Errors
    ///
    /// Returns error description if the value cannot be deserialized.
    fn apply_runtime_layer(&self, value: &toml::Value, path: &str) -> Result<(), String>;
}

/// Extracts runtime layer values for persistence to runtime.toml.
///
/// Walks the config tree and collects only values that have been set
/// in the runtime layer (GUI overrides). Returns None if no runtime
/// value exists, allowing sparse serialization.
pub trait ExtractRuntimeValues {
    /// Extract runtime values as TOML.
    ///
    /// Returns Some(Value) if this field or any nested field has a runtime
    /// override, None otherwise. For structs, returns a Table containing
    /// only fields with runtime values.
    fn extract_runtime_values(&self) -> Option<toml::Value>;
}

/// Trait for subscribing to changes in config structures.
///
/// Enables automatic persistence by watching all fields for changes.
pub trait SubscribeChanges {
    /// Subscribe to changes by sending notifications to the provided channel.
    ///
    /// Spawns background tasks that watch for changes and send () to the channel.
    fn subscribe_changes(&self, tx: mpsc::UnboundedSender<()>);
}

/// Resets the config layer to None without notifying watchers.
///
/// Part of the reload cycle: reset -> apply -> commit.
/// Clearing the config layer allows fields removed from TOML to
/// fall back to their default values after commit.
pub trait ResetConfigLayer {
    /// Clears the config layer value without triggering notifications.
    ///
    /// After calling this, the effective value is NOT recomputed.
    /// Call `commit_config_reload` after applying new values to
    /// recompute and notify watchers.
    fn reset_config_layer(&self);
}

/// Resets the runtime layer to None without notifying watchers.
///
/// Part of the runtime reload cycle: reset -> apply -> commit.
/// Clearing the runtime layer allows externally removed overrides
/// to fall back to config or default values after commit.
pub trait ResetRuntimeLayer {
    /// Clears the runtime layer value without triggering notifications.
    ///
    /// After calling this, the effective value is NOT recomputed.
    /// Call `commit_config_reload` after applying new values to
    /// recompute and notify watchers.
    fn reset_runtime_layer(&self);
}

/// Commits a config reload by recomputing effective values.
///
/// Part of the reload cycle: reset -> apply -> commit.
/// Recomputes effective values and notifies watchers only for
/// fields whose effective value actually changed.
pub trait CommitConfigReload {
    /// Recomputes effective values and notifies watchers of changes.
    ///
    /// Thanks to change detection in Property::set, watchers are only
    /// notified if the effective value actually changed.
    fn commit_config_reload(&self);
}

/// Drops every runtime override in the tree, notifying watchers even when
/// the effective value is unchanged.
pub trait ClearAllRuntime {
    /// Drops every runtime override and fires watcher notifications
    /// unconditionally, even for fields whose effective value is unchanged.
    fn clear_all_runtime(&self);
}

/// Clears the runtime override at a specific path.
///
/// Used by CLI reset commands to remove runtime overrides by string path.
/// For direct struct access (GUI), use `ConfigProperty::clear_runtime()` instead.
pub trait ClearRuntimeByPath {
    /// Clears the runtime value at the given dot-separated path.
    ///
    /// Returns `Ok(true)` if a value was cleared, `Ok(false)` if no runtime
    /// value existed at that path.
    ///
    /// # Errors
    ///
    /// Returns error if the path doesn't match any field.
    fn clear_runtime_by_path(&self, path: &str) -> Result<bool, String>;
}
