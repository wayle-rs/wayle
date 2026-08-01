//! Layered configuration property with provenance tracking.
//!
//! Provides `ConfigProperty<T>` and configuration layer traits for
//! three-layer config management (default, config, runtime).

mod config;
mod removed;
mod traits;

pub use config::{ConfigProperty, ValueSource};
pub use removed::Removed;
pub use traits::{
    ApplyConfigLayer, ApplyRuntimeLayer, ClearAllRuntime, ClearRuntimeByPath, CommitConfigReload,
    ExtractRuntimeValues, ResetConfigLayer, ResetRuntimeLayer, SubscribeChanges,
};
