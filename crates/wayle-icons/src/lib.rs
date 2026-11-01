//! Icon management for Wayle with CDN fetching and GTK integration.
//!
//! Features:
//! - [`IconSource`] trait and implementations for different icon CDNs
//! - [`IconManager`] for installing and removing icons
//! - [`IconRegistry`] for registering icons with GTK's IconTheme
//!
//! # Icon Sources
//!
//! | Source | Prefix | Use Case |
//! |--------|--------|----------|
//! | Tabler | `tb-` | UI icons (home, settings, bell) |
//! | TablerFilled | `tbf-` | Solid UI icons |
//! | Simple Icons | `si-` | Brand logos (firefox, spotify) |
//! | Lucide | `ld-` | Alternative UI icons |
//!
//! # Example
//!
//! ```rust,no_run
//! use wayle_icons::{IconManager, IconRegistry, sources};
//!
//! # async fn example() -> wayle_icons::Result<()> {
//! // At app startup, register icon directory with GTK
//! let registry = IconRegistry::new()?;
//! registry.init()?;
//!
//! // Install icons from CDN
//! let manager = IconManager::new()?;
//! manager.install(&sources::Tabler, &["home", "settings"]).await?;
//!
//! // Icons are now available via set_icon_name("tb-home")
//! # Ok(())
//! # }
//! ```

/// Error types for icon operations.
pub mod error;

/// Icon manager for install, remove, and validation operations.
pub mod manager;

/// On-disk icon format migration.
pub mod migrate;

/// GTK IconTheme integration.
pub mod registry;

/// Icon source definitions (Tabler, Simple Icons, Lucide).
pub mod sources;

/// Extract icon references from config and install what's missing.
pub mod sync;

/// SVG transformation for GTK symbolic icon compatibility.
pub mod transform;

pub use error::{Error, Result};
pub use manager::{IconManager, InstallFailure, InstallResult};
pub use registry::IconRegistry;
pub use sources::IconSource;
pub use sync::{IconOrigin, MissingIcon, SyncFailure, SyncSummary};
