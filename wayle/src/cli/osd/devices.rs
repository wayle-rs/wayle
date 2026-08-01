//! Lists the devices `wayle osd` can target.

use wayle_ipc::shell_ipc::OsdDeviceInfo;

use super::proxy::{connect, format_error};
use crate::cli::CliAction;

/// Device kinds in display order, paired with their section heading.
const SECTIONS: [(&str, &str); 3] = [
    ("speaker", "Speakers"),
    ("microphone", "Microphones"),
    ("brightness", "Displays"),
];

/// Prints every targetable device grouped by kind, marking defaults with `*`.
///
/// # Errors
/// Returns error if D-Bus communication fails.
pub async fn execute() -> CliAction {
    let (_connection, proxy) = connect().await?;

    let devices = proxy
        .osd_devices()
        .await
        .map_err(|err| format_error("list OSD devices", err))?;

    if devices.is_empty() {
        println!("No OSD devices found");
        return Ok(());
    }

    for (kind, heading) in SECTIONS {
        let matching: Vec<&OsdDeviceInfo> =
            devices.iter().filter(|device| device.kind == kind).collect();

        if matching.is_empty() {
            continue;
        }

        println!("{heading}:");

        for device in matching {
            let marker = if device.is_default { "*" } else { " " };
            let label = &device.label;

            if device.id == device.label {
                println!("  {marker} {label}");
            } else {
                println!("  {marker} {label} [{}]", device.id);
            }
        }
    }

    Ok(())
}
