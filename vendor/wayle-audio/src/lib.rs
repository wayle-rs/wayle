//! PulseAudio integration for managing audio devices and streams.
//!
//! # Overview
//!
//! Provides reactive access to PulseAudio through [`AudioService`].
//! All state is exposed via [`Property`] fields that automatically update when
//! PulseAudio state changes.
//!
//! # Reactive Properties
//!
//! Every field on [`AudioService`], [`OutputDevice`], [`InputDevice`], and
//! [`AudioStream`] is a [`Property<T>`] with two access patterns:
//!
//! - **Snapshot**: Call `.get()` for the current value
//! - **Stream**: Call `.watch()` for a `Stream<Item = T>` that yields on changes
//!
//! ```rust,no_run
//! use wayle_audio::AudioService;
//! use futures::StreamExt;
//!
//! # async fn example() -> Result<(), wayle_audio::Error> {
//! let audio = AudioService::new().await?;
//!
//! // Snapshot: get current default output device
//! if let Some(device) = audio.default_output.get() {
//!     println!("Default output: {}", device.description.get());
//!     println!("Volume: {:?}", device.volume.get());
//!     println!("Muted: {}", device.muted.get());
//! }
//!
//! // Stream: react to default output changes
//! let mut stream = audio.default_output.watch();
//! while let Some(maybe_device) = stream.next().await {
//!     match maybe_device {
//!         Some(device) => println!("Default changed to: {}", device.description.get()),
//!         None => println!("No default output"),
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Live vs Snapshot Instances
//!
//! Devices from [`AudioService`] fields (`output_devices`, `default_output`, etc.)
//! are **live**: their properties update when PulseAudio state changes.
//!
//! The explicit lookup methods differ:
//!
//! | Method | Returns | Properties Update? |
//! |--------|---------|-------------------|
//! | `output_device()` | `OutputDevice` | No (snapshot) |
//! | `output_device_monitored()` | `Arc<OutputDevice>` | Yes (live) |
//!
//! ```rust,no_run
//! # use wayle_audio::{AudioService, types::device::{DeviceKey, DeviceType}};
//! # async fn example() -> Result<(), wayle_audio::Error> {
//! # let audio = AudioService::new().await?;
//! # let key = DeviceKey::new(0, DeviceType::Output);
//! // Snapshot: properties won't update
//! let snapshot = audio.output_device(key).await?;
//! let vol_at_query_time = snapshot.volume.get();
//!
//! // Live: properties update automatically
//! let live = audio.output_device_monitored(key).await?;
//! let mut vol_stream = live.volume.watch();
//! // vol_stream yields whenever volume changes in PulseAudio
//! # Ok(())
//! # }
//! ```
//!
//! # Controlling Devices
//!
//! [`OutputDevice`] and [`InputDevice`] have control methods:
//!
//! ```rust,no_run
//! # use wayle_audio::{AudioService, volume::types::Volume};
//! # async fn example() -> Result<(), wayle_audio::Error> {
//! # let audio = AudioService::new().await?;
//! if let Some(device) = audio.default_output.get() {
//!     // Mute/unmute
//!     device.set_mute(true).await?;
//!
//!     // Set volume (0.0 to 1.0 per channel)
//!     device.set_volume(Volume::stereo(0.5, 0.5)).await?;
//!
//!     // Make this device the default
//!     device.set_as_default().await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! | Method | Effect |
//! |--------|--------|
//! | `with_daemon()` | Control audio from scripts or other processes |
//!
//! ```rust,no_run
//! use wayle_audio::AudioService;
//!
//! # async fn example() -> Result<(), wayle_audio::Error> {
//! let audio = AudioService::builder()
//!     .with_daemon()
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # D-Bus Interface
//!
//! When `with_daemon()` is enabled, the service registers on the session bus.
//!
//! - **Service:** `com.wayle.Audio1`
//! - **Path:** `/com/wayle/Audio`
//! - **Interface:** `com.wayle.Audio1`
//!
//! See [`dbus.md`](https://github.com/wayle-rs/wayle/blob/master/crates/wayle-audio/dbus.md) for the full interface specification.
//!
//! # Service Fields
//!
//! [`AudioService`] exposes these reactive properties:
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | [`output_devices`] | `Vec<Arc<OutputDevice>>` | All sinks (speakers, headphones) |
//! | [`input_devices`] | `Vec<Arc<InputDevice>>` | All sources (microphones) |
//! | [`default_output`] | `Option<Arc<OutputDevice>>` | Current default sink |
//! | [`default_input`] | `Option<Arc<InputDevice>>` | Current default source |
//! | [`playback_streams`] | `Vec<Arc<AudioStream>>` | Active playback (apps playing audio) |
//! | [`recording_streams`] | `Vec<Arc<AudioStream>>` | Active recording (apps capturing audio) |
//!
//! [`output_devices`]: AudioService::output_devices
//! [`input_devices`]: AudioService::input_devices
//! [`default_output`]: AudioService::default_output
//! [`default_input`]: AudioService::default_input
//! [`playback_streams`]: AudioService::playback_streams
//! [`recording_streams`]: AudioService::recording_streams
//! [`Property`]: wayle_core::Property
//! [`Property<T>`]: wayle_core::Property
//! [`OutputDevice`]: core::device::output::OutputDevice
//! [`InputDevice`]: core::device::input::InputDevice
//! [`AudioStream`]: core::stream::AudioStream

#![cfg_attr(test, allow(clippy::panic))]

mod backend;
mod builder;
/// Core domain models
pub mod core;
/// D-Bus interface for external control
pub mod dbus;
mod error;
mod events;
mod monitoring;
mod service;
mod tokio_mainloop;
/// Types for the audio service
pub mod types;
/// Volume control domain
pub mod volume;

pub use builder::AudioServiceBuilder;
pub use error::Error;
pub use service::AudioService;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDocTests;
