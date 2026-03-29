//! Reactive state for Wayle services. Wrap a value in [`Property<T>`]
//! and consumers can `.get()` it or `.watch()` for a stream of changes.
//!
//! ```rust,no_run
//! use wayle_core::Property;
//! use futures::stream::StreamExt;
//!
//! # async fn example() {
//! let brightness = Property::new(75u32);
//! brightness.set(100);
//!
//! let mut changes = brightness.watch();
//! while let Some(level) = changes.next().await {
//!     println!("{level}");
//! }
//! # }
//! ```
//!
//! Also includes D-Bus macros (`unwrap_*!`, `watch_all!`) for extracting
//! properties with type-safe defaults.
//!
//! Enable `schema` for [`schemars::JsonSchema`] support on `Property<T>`.

#[macro_use]
mod macros;
/// XDG Base Directory path resolution.
pub mod paths;
mod property;

use std::sync::Arc;

pub use property::{Property, PropertyStream};

/// A [`Property`] holding a service that initializes in the background.
/// Starts `None`, becomes `Some` once the service is ready.
pub type DeferredService<T> = Property<Option<Arc<T>>>;

/// D-Bus root object path.
pub const ROOT_PATH: &str = "/";

/// D-Bus null object path (no associated object).
pub const NULL_PATH: &str = "/";
