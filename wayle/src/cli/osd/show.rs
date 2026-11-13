//! Manual OSD display commands.
//!
//! The three targets differ only in which proxy method they call, so they
//! share one round-trip helper. An empty device string means "the default
//! device", matching the convention `bar_hide` already uses for monitors.

use super::proxy::{connect, format_error};
use crate::cli::CliAction;

/// Shows the speaker OSD for `device`, or the default sink when `None`.
///
/// # Errors
/// Returns error if D-Bus communication fails or the device is unknown.
pub async fn speaker(device: Option<&str>) -> CliAction {
    let (_connection, proxy) = connect().await?;

    let label = proxy
        .osd_show_speaker(device.unwrap_or_default())
        .await
        .map_err(|err| format_error("show speaker OSD", err))?;

    println!("Speaker OSD: {label}");

    Ok(())
}

/// Shows the microphone OSD for `device`, or the default source when `None`.
///
/// # Errors
/// Returns error if D-Bus communication fails or the device is unknown.
pub async fn microphone(device: Option<&str>) -> CliAction {
    let (_connection, proxy) = connect().await?;

    let label = proxy
        .osd_show_microphone(device.unwrap_or_default())
        .await
        .map_err(|err| format_error("show microphone OSD", err))?;

    println!("Microphone OSD: {label}");

    Ok(())
}

/// Shows the brightness OSD for `device`, or the primary backlight when `None`.
///
/// # Errors
/// Returns error if D-Bus communication fails or the device is unknown.
pub async fn brightness(device: Option<&str>) -> CliAction {
    let (_connection, proxy) = connect().await?;

    let label = proxy
        .osd_show_brightness(device.unwrap_or_default())
        .await
        .map_err(|err| format_error("show brightness OSD", err))?;

    println!("Brightness OSD: {label}");

    Ok(())
}
