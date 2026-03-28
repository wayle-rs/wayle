//! Network management via NetworkManager D-Bus.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use wayle_network::NetworkService;
//!
//! # async fn example() -> Result<(), wayle_network::Error> {
//! let net = NetworkService::new().await?;
//!
//! // Check WiFi state (wifi is reactive for hot-plug support)
//! if let Some(wifi) = net.wifi.get() {
//!     println!("WiFi enabled: {}", wifi.enabled.get());
//!     for ap in wifi.access_points.get().iter() {
//!         println!("  {} ({}%)", ap.ssid.get(), ap.strength.get());
//!     }
//! }
//!
//! // Check wired state
//! if let Some(wired) = net.wired.get() {
//!     println!("Ethernet status: {:?}", wired.connectivity.get());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Watching for Changes
//!
//! ```rust,no_run
//! use wayle_network::NetworkService;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), wayle_network::Error> {
//! # let net = NetworkService::new().await?;
//! if let Some(wifi) = net.wifi.get() {
//!     let mut stream = wifi.access_points.watch();
//!     while let Some(aps) = stream.next().await {
//!         println!("{} networks visible", aps.len());
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # WiFi Control
//!
//! ```rust,no_run
//! # use wayle_network::NetworkService;
//! # async fn example() -> Result<(), wayle_network::Error> {
//! # let net = NetworkService::new().await?;
//! if let Some(wifi) = net.wifi.get() {
//!     // Enable WiFi
//!     wifi.set_enabled(true).await?;
//!
//!     // List available networks
//!     for ap in wifi.access_points.get().iter() {
//!         println!("{}: {:?} ({}%)",
//!             ap.ssid.get(),
//!             ap.security.get(),
//!             ap.strength.get()
//!         );
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Reactive Properties
//!
//! All fields are [`Property<T>`](wayle_core::Property):
//! - `.get()` - Current value snapshot
//! - `.watch()` - Stream yielding on changes
//!
//! # Service Fields
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `wifi` | `Property<Option<Arc<Wifi>>>` | WiFi device (reactive for hot-plug) |
//! | `wired` | `Property<Option<Arc<Wired>>>` | Ethernet device (reactive for hot-plug) |
//! | `settings` | `Settings` | Connection profile management |
//! | `primary` | `Property<ConnectionType>` | Active connection type |

/// Core network domain models.
pub mod core;
mod discovery;
mod error;
mod monitoring;
mod proxy;
mod service;
/// Network type definitions
pub mod types;
/// WiFi device functionality
pub mod wifi;
/// Wired device functionality
pub mod wired;
/// WireGuard VPN functionality
pub mod wireguard;

pub use error::Error;
pub use service::NetworkService;
