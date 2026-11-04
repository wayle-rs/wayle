use std::{collections::HashMap, sync::Arc};

use wayle_bluetooth::{
    core::device::{Device, DeviceInfo},
    types::device::{DeviceActivity, DeviceError},
};
use zbus::zvariant::OwnedObjectPath;

const MAJOR_COMPUTER: u32 = 0x01;
const MAJOR_PHONE: u32 = 0x02;
const MAJOR_NETWORK: u32 = 0x03;
const MAJOR_AV: u32 = 0x04;
const MAJOR_PERIPHERAL: u32 = 0x05;
const MAJOR_IMAGING: u32 = 0x06;
const MAJOR_WEARABLE: u32 = 0x07;
const MAJOR_TOY: u32 = 0x08;
const MAJOR_HEALTH: u32 = 0x09;

fn major_class(raw: u32) -> u32 {
    (raw >> 8) & 0x1F
}

fn minor_class(raw: u32) -> u32 {
    (raw >> 2) & 0x3F
}

pub(crate) fn device_icon(icon_hint: Option<&str>, class: Option<u32>) -> &'static str {
    if let Some(hint) = icon_hint {
        match hint {
            "audio-headphones" | "audio-headset" => return "ld-headphones-symbolic",
            "audio-speakers" => return "ld-speaker-symbolic",
            "input-keyboard" => return "ld-keyboard-symbolic",
            "input-mouse" | "input-tablet" => return "ld-mouse-symbolic",
            "input-gaming" => return "ld-gamepad-2-symbolic",
            "video-display" | "computer" => return "ld-monitor-symbolic",
            "phone" => return "ld-smartphone-symbolic",
            _ => {}
        }
    }

    let Some(raw) = class else {
        return "ld-bluetooth-symbolic";
    };

    match major_class(raw) {
        MAJOR_COMPUTER => computer_icon(minor_class(raw)),
        MAJOR_PHONE => "ld-smartphone-symbolic",
        MAJOR_AV => av_icon(minor_class(raw)),
        MAJOR_PERIPHERAL => peripheral_icon(minor_class(raw)),
        MAJOR_IMAGING => imaging_icon(minor_class(raw)),
        MAJOR_WEARABLE => "ld-watch-symbolic",
        _ => "ld-bluetooth-symbolic",
    }
}

fn computer_icon(minor: u32) -> &'static str {
    match minor {
        0x03 => "ld-laptop-symbolic",
        0x07 => "ld-tablet-symbolic",
        _ => "ld-monitor-symbolic",
    }
}

fn av_icon(minor: u32) -> &'static str {
    match minor {
        0x01 | 0x02 => "ld-headphones-symbolic",
        0x04 => "ld-mic-symbolic",
        0x05 | 0x07 | 0x08 | 0x0A => "ld-speaker-symbolic",
        0x06 => "ld-headphones-symbolic",
        0x09 => "tb-device-tv-symbolic",
        0x0B => "tb-device-tv-symbolic",
        0x0C | 0x0D => "ld-camera-symbolic",
        0x0E | 0x0F => "ld-monitor-symbolic",
        0x10 => "ld-monitor-symbolic",
        0x12 => "ld-gamepad-2-symbolic",
        _ => "ld-headphones-symbolic",
    }
}

fn peripheral_icon(minor: u32) -> &'static str {
    let kb_pointing = minor >> 4;
    let subtype = minor & 0x0F;
    match kb_pointing {
        0x01 | 0x03 => "ld-keyboard-symbolic",
        0x02 => "ld-mouse-symbolic",
        _ => match subtype {
            0x01 | 0x02 => "ld-gamepad-2-symbolic",
            0x05 => "ld-mouse-symbolic",
            _ => "ld-bluetooth-symbolic",
        },
    }
}

// Imaging minor classes are bit flags (in the minor-class field, after its
// two reserved low bits).
const IMAGING_DISPLAY: u32 = 0x04;
const IMAGING_CAMERA: u32 = 0x08;
const IMAGING_SCANNER: u32 = 0x10;
const IMAGING_PRINTER: u32 = 0x20;

fn imaging_icon(minor: u32) -> &'static str {
    if minor & IMAGING_PRINTER != 0 {
        return "ld-printer-symbolic";
    }
    if minor & IMAGING_CAMERA != 0 {
        return "ld-camera-symbolic";
    }
    if minor & IMAGING_DISPLAY != 0 {
        return "ld-monitor-symbolic";
    }
    "ld-printer-symbolic"
}

pub(super) fn battery_level_icon(percent: u8) -> &'static str {
    match percent {
        0..=5 => "tb-battery-vertical-symbolic",
        6..=25 => "tb-battery-vertical-1-symbolic",
        26..=50 => "tb-battery-vertical-2-symbolic",
        51..=75 => "tb-battery-vertical-3-symbolic",
        _ => "tb-battery-vertical-4-symbolic",
    }
}

pub(crate) fn device_type_key(icon_hint: Option<&str>, class: Option<u32>) -> &'static str {
    if let Some(hint) = icon_hint {
        match hint {
            "audio-headphones" => return "dropdown-bluetooth-type-headphones",
            "audio-headset" => return "dropdown-bluetooth-type-headset",
            "audio-speakers" => return "dropdown-bluetooth-type-loudspeaker",
            "input-keyboard" => return "dropdown-bluetooth-type-keyboard",
            "input-mouse" | "input-tablet" => return "dropdown-bluetooth-type-mouse",
            "input-gaming" => return "dropdown-bluetooth-type-gamepad",
            "video-display" => return "dropdown-bluetooth-type-video-display",
            "computer" => return "dropdown-bluetooth-type-computer",
            "phone" => return "dropdown-bluetooth-type-phone",
            _ => {}
        }
    }

    let Some(raw) = class else {
        return "dropdown-bluetooth-type-unknown";
    };

    match major_class(raw) {
        MAJOR_COMPUTER => computer_type_key(minor_class(raw)),
        MAJOR_PHONE => phone_type_key(minor_class(raw)),
        MAJOR_NETWORK => "dropdown-bluetooth-type-network",
        MAJOR_AV => av_type_key(minor_class(raw)),
        MAJOR_PERIPHERAL => peripheral_type_key(minor_class(raw)),
        MAJOR_IMAGING => imaging_type_key(minor_class(raw)),
        MAJOR_WEARABLE => wearable_type_key(minor_class(raw)),
        MAJOR_TOY => toy_type_key(minor_class(raw)),
        MAJOR_HEALTH => "dropdown-bluetooth-type-health",
        _ => "dropdown-bluetooth-type-unknown",
    }
}

fn av_type_key(minor: u32) -> &'static str {
    match minor {
        0x01 => "dropdown-bluetooth-type-headset",
        0x02 => "dropdown-bluetooth-type-handsfree",
        0x04 => "dropdown-bluetooth-type-microphone",
        0x05 => "dropdown-bluetooth-type-loudspeaker",
        0x06 => "dropdown-bluetooth-type-headphones",
        0x07 => "dropdown-bluetooth-type-portable-audio",
        0x08 => "dropdown-bluetooth-type-car-audio",
        0x09 => "dropdown-bluetooth-type-set-top-box",
        0x0A => "dropdown-bluetooth-type-hifi",
        0x0B => "dropdown-bluetooth-type-vcr",
        0x0C => "dropdown-bluetooth-type-video-camera",
        0x0D => "dropdown-bluetooth-type-camcorder",
        0x0E => "dropdown-bluetooth-type-video-monitor",
        0x0F => "dropdown-bluetooth-type-video-display",
        0x10 => "dropdown-bluetooth-type-video-conferencing",
        0x12 => "dropdown-bluetooth-type-gaming",
        _ => "dropdown-bluetooth-type-audio-video",
    }
}

fn computer_type_key(minor: u32) -> &'static str {
    match minor {
        0x01 => "dropdown-bluetooth-type-desktop",
        0x02 => "dropdown-bluetooth-type-server",
        0x03 => "dropdown-bluetooth-type-laptop",
        0x04 => "dropdown-bluetooth-type-handheld",
        0x05 => "dropdown-bluetooth-type-palm",
        0x06 => "dropdown-bluetooth-type-wearable-computer",
        0x07 => "dropdown-bluetooth-type-computer-tablet",
        _ => "dropdown-bluetooth-type-computer",
    }
}

fn phone_type_key(minor: u32) -> &'static str {
    match minor {
        0x01 => "dropdown-bluetooth-type-cellular",
        0x02 => "dropdown-bluetooth-type-cordless",
        0x03 => "dropdown-bluetooth-type-smartphone",
        0x04 => "dropdown-bluetooth-type-modem",
        _ => "dropdown-bluetooth-type-phone",
    }
}

fn wearable_type_key(minor: u32) -> &'static str {
    match minor {
        0x01 => "dropdown-bluetooth-type-wrist-watch",
        0x02 => "dropdown-bluetooth-type-pager",
        0x03 => "dropdown-bluetooth-type-jacket",
        0x04 => "dropdown-bluetooth-type-helmet",
        0x05 => "dropdown-bluetooth-type-glasses",
        _ => "dropdown-bluetooth-type-wearable",
    }
}

fn toy_type_key(minor: u32) -> &'static str {
    match minor {
        0x01 => "dropdown-bluetooth-type-robot",
        0x02 => "dropdown-bluetooth-type-vehicle",
        0x03 => "dropdown-bluetooth-type-doll",
        0x04 => "dropdown-bluetooth-type-controller",
        0x05 => "dropdown-bluetooth-type-game",
        _ => "dropdown-bluetooth-type-toy",
    }
}

fn imaging_type_key(minor: u32) -> &'static str {
    if minor & IMAGING_PRINTER != 0 {
        return "dropdown-bluetooth-type-printer";
    }
    if minor & IMAGING_SCANNER != 0 {
        return "dropdown-bluetooth-type-scanner";
    }
    if minor & IMAGING_CAMERA != 0 {
        return "dropdown-bluetooth-type-camera";
    }
    if minor & IMAGING_DISPLAY != 0 {
        return "dropdown-bluetooth-type-display";
    }
    "dropdown-bluetooth-type-imaging"
}

fn peripheral_type_key(minor: u32) -> &'static str {
    let kb_pointing = minor >> 4;
    let subtype = minor & 0x0F;
    match kb_pointing {
        0x01 => "dropdown-bluetooth-type-keyboard",
        0x02 => "dropdown-bluetooth-type-mouse",
        0x03 => "dropdown-bluetooth-type-combo-keyboard",
        _ => match subtype {
            0x01 => "dropdown-bluetooth-type-joystick",
            0x02 => "dropdown-bluetooth-type-gamepad",
            0x03 => "dropdown-bluetooth-type-remote",
            0x04 => "dropdown-bluetooth-type-sensing",
            0x05 => "dropdown-bluetooth-type-tablet",
            0x06 => "dropdown-bluetooth-type-card-reader",
            _ => "dropdown-bluetooth-type-peripheral",
        },
    }
}

pub(crate) fn format_passkey(passkey: u32) -> String {
    format!("{passkey:06}")
}

const BT_BASE_UUID_SUFFIX: &str = "-0000-1000-8000-00805f9b34fb";

fn extract_short_uuid(uuid: &str) -> Option<u16> {
    let lower = uuid.to_ascii_lowercase();
    let hex_part = lower
        .strip_prefix("0000")?
        .strip_suffix(BT_BASE_UUID_SUFFIX)?;
    u16::from_str_radix(hex_part, 16).ok()
}

/// Maps a Bluetooth service UUID to an FTL key for display.
///
/// Gotten from the Bluetooth SIG Assigned Numbers:
/// <https://www.bluetooth.com/specifications/assigned-numbers/>
pub(crate) fn service_name_key(uuid: &str) -> &'static str {
    let Some(short) = extract_short_uuid(uuid) else {
        return "dropdown-bluetooth-service-proprietary";
    };

    match short {
        0x1101 => "dropdown-bluetooth-service-serial-port",
        0x1102 => "dropdown-bluetooth-service-lan-access",
        0x1103 => "dropdown-bluetooth-service-dialup-networking",
        0x1105 => "dropdown-bluetooth-service-object-push",
        0x1106 => "dropdown-bluetooth-service-file-transfer",
        0x1108 | 0x1112 => "dropdown-bluetooth-service-headset",
        0x110a => "dropdown-bluetooth-service-audio-source",
        0x110b => "dropdown-bluetooth-service-audio-sink",
        0x110c | 0x110e | 0x110f => "dropdown-bluetooth-service-remote-control",
        0x110d => "dropdown-bluetooth-service-audio-distribution",
        0x111e | 0x111f => "dropdown-bluetooth-service-handsfree",
        0x1115..=0x1117 => "dropdown-bluetooth-service-network-access",
        0x1124 => "dropdown-bluetooth-service-input-device",
        0x112d => "dropdown-bluetooth-service-sim-access",
        0x112f | 0x1130 => "dropdown-bluetooth-service-phonebook",
        0x1132..=0x1134 => "dropdown-bluetooth-service-messaging",
        _ => "dropdown-bluetooth-service-unknown",
    }
}

pub(super) struct DeviceDisplayInfo {
    pub name: String,
    pub icon: &'static str,
    pub device_type_key: &'static str,
}

impl Default for DeviceDisplayInfo {
    fn default() -> Self {
        Self {
            name: "-".into(),
            icon: "ld-bluetooth-symbolic",
            device_type_key: "dropdown-bluetooth-type-unknown",
        }
    }
}

/// How a device is shown: its name, icon and type.
pub(super) fn resolve_device_display(info: &DeviceInfo) -> DeviceDisplayInfo {
    DeviceDisplayInfo {
        name: info.alias.clone(),
        icon: device_icon(info.icon.as_deref(), info.class),
        device_type_key: device_type_key(info.icon.as_deref(), info.class),
    }
}

/// What a device's row shows, and the device it acts on. Equal snapshots
/// look and act the same, so a row whose snapshot is unchanged is left alone.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeviceSnapshot {
    /// BlueZ's alias, which falls back to the device's name, then its address.
    pub name: String,
    pub icon: &'static str,
    /// Localization key of the device type, translated when displayed.
    pub device_type_key: &'static str,
    pub battery: Option<u8>,
    pub connected: bool,
    pub paired: bool,
    pub activity: DeviceActivity,
    /// The most recent failed action on the device, if any.
    pub error: Option<DeviceError>,
    pub device: Arc<Device>,
}

impl DeviceSnapshot {
    /// Whether it's one of the user's devices (see [`is_mine`]).
    pub(crate) fn is_mine(&self) -> bool {
        mine(self.connected, self.paired)
    }
}

/// Whether a device is one of the user's, listed under "My devices": it is
/// connected or paired. The others are available to connect to.
pub(crate) fn is_mine(info: &DeviceInfo) -> bool {
    mine(info.connected, info.paired)
}

fn mine(connected: bool, paired: bool) -> bool {
    connected || paired
}

/// Whether the dropdown lists a device: it is one of the user's, or
/// advertises a name. Most devices seen while scanning somewhere crowded
/// advertise no name; this is checked before any other work on them.
pub(crate) fn is_listed(info: &DeviceInfo) -> bool {
    info.name.is_some() || is_mine(info)
}

/// What `device`'s row shows given its `info`, if it has a row.
pub(crate) fn snapshot_of(device: &Arc<Device>, info: &DeviceInfo) -> Option<DeviceSnapshot> {
    is_listed(info).then(|| {
        let DeviceDisplayInfo {
            name,
            icon,
            device_type_key,
        } = resolve_device_display(info);
        DeviceSnapshot {
            name,
            icon,
            device_type_key,
            battery: info.battery_percentage,
            connected: info.connected,
            paired: info.paired,
            activity: info.activity,
            error: info.last_error.clone(),
            device: Arc::clone(device),
        }
    })
}

pub(crate) struct SplitDeviceLists {
    pub my_devices: Vec<DeviceSnapshot>,
    pub available_devices: Vec<DeviceSnapshot>,
}

/// Splits `devices` into the two dropdown lists.
///
/// "My devices" are ordered connected first, then by name. Available devices
/// are ordered strongest signal first, with devices not currently seen last.
/// Ties keep the order in `shown_available` (each device's current row), so
/// equal readings never swap rows; new devices among them are ordered by name.
///
/// This sorts on the RSSI BlueZ reports, which already has per-device
/// hysteresis: BlueZ only reports a change once a device has moved at least 8
/// dBm (`RSSI_THRESHOLD`) from its last report. Jitter below that never
/// reorders the list, and devices more than 16 dBm apart are always in order.
/// (BlueZ 5.87 and earlier drop the threshold while any client's discovery
/// filter is active, and report every change.)
pub(crate) fn build_split_device_lists(
    devices: &[Arc<Device>],
    shown_available: &HashMap<OwnedObjectPath, usize>,
) -> SplitDeviceLists {
    let mut my_devices = Vec::new();
    // With each device's RSSI, `None` while it isn't being seen.
    let mut available_devices = Vec::new();

    for device in devices {
        let Some(snapshot) = snapshot_of(device, &device.info.get()) else {
            continue;
        };
        if snapshot.is_mine() {
            my_devices.push(snapshot);
        } else {
            available_devices.push((device.signal.get().rssi, snapshot));
        }
    }

    my_devices.sort_by(|left, right| {
        right
            .connected
            .cmp(&left.connected)
            .then_with(|| left.name.cmp(&right.name))
    });
    let shown_row = |snapshot: &DeviceSnapshot| {
        shown_available
            .get(&snapshot.device.object_path)
            .copied()
            .unwrap_or(usize::MAX)
    };
    available_devices.sort_by(|(left_rssi, left), (right_rssi, right)| {
        right_rssi
            .cmp(left_rssi)
            .then_with(|| shown_row(left).cmp(&shown_row(right)))
            .then_with(|| left.name.cmp(&right.name))
    });

    SplitDeviceLists {
        my_devices,
        available_devices: available_devices
            .into_iter()
            .map(|(_, snapshot)| snapshot)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_from_headphones_hint() {
        assert_eq!(
            device_icon(Some("audio-headphones"), None),
            "ld-headphones-symbolic"
        );
        assert_eq!(
            device_icon(Some("audio-headset"), None),
            "ld-headphones-symbolic"
        );
    }

    #[test]
    fn icon_from_speaker_hint() {
        assert_eq!(
            device_icon(Some("audio-speakers"), None),
            "ld-speaker-symbolic"
        );
    }

    #[test]
    fn icon_from_keyboard_hint() {
        assert_eq!(
            device_icon(Some("input-keyboard"), None),
            "ld-keyboard-symbolic"
        );
    }

    #[test]
    fn icon_from_mouse_hint() {
        assert_eq!(device_icon(Some("input-mouse"), None), "ld-mouse-symbolic");
        assert_eq!(device_icon(Some("input-tablet"), None), "ld-mouse-symbolic");
    }

    #[test]
    fn icon_from_gaming_hint() {
        assert_eq!(
            device_icon(Some("input-gaming"), None),
            "ld-gamepad-2-symbolic"
        );
    }

    #[test]
    fn icon_from_display_hint() {
        assert_eq!(
            device_icon(Some("video-display"), None),
            "ld-monitor-symbolic"
        );
        assert_eq!(device_icon(Some("computer"), None), "ld-monitor-symbolic");
    }

    #[test]
    fn icon_from_phone_hint() {
        assert_eq!(device_icon(Some("phone"), None), "ld-smartphone-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_computer() {
        let class = MAJOR_COMPUTER << 8;
        assert_eq!(device_icon(None, Some(class)), "ld-monitor-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_phone() {
        let class = MAJOR_PHONE << 8;
        assert_eq!(device_icon(None, Some(class)), "ld-smartphone-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_av_headphones() {
        let class = (MAJOR_AV << 8) | (0x06 << 2);
        assert_eq!(device_icon(None, Some(class)), "ld-headphones-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_av_set_top_box() {
        let class = (MAJOR_AV << 8) | (0x09 << 2);
        assert_eq!(device_icon(None, Some(class)), "tb-device-tv-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_av_speaker() {
        let class = (MAJOR_AV << 8) | (0x05 << 2);
        assert_eq!(device_icon(None, Some(class)), "ld-speaker-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_av_microphone() {
        let class = (MAJOR_AV << 8) | (0x04 << 2);
        assert_eq!(device_icon(None, Some(class)), "ld-mic-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_peripheral_keyboard() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x01 << 6);
        assert_eq!(device_icon(None, Some(class)), "ld-keyboard-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_peripheral_mouse() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x02 << 6);
        assert_eq!(device_icon(None, Some(class)), "ld-mouse-symbolic");
    }

    #[test]
    fn icon_fallback_to_class_peripheral_gamepad() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x02 << 2);
        assert_eq!(device_icon(None, Some(class)), "ld-gamepad-2-symbolic");
    }

    #[test]
    fn type_peripheral_joystick_from_subtype() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x01 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-joystick"
        );
    }

    #[test]
    fn type_peripheral_keyboard_from_kb_pointing() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x01 << 6);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-keyboard"
        );
    }

    #[test]
    fn type_av_vcr() {
        let class = (MAJOR_AV << 8) | (0x0B << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-vcr"
        );
    }

    #[test]
    fn type_av_video_conferencing() {
        let class = (MAJOR_AV << 8) | (0x10 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-video-conferencing"
        );
    }

    #[test]
    fn type_av_handsfree() {
        let class = (MAJOR_AV << 8) | (0x02 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-handsfree"
        );
    }

    #[test]
    fn icon_hint_takes_precedence_over_class() {
        let class = MAJOR_COMPUTER << 8;
        assert_eq!(
            device_icon(Some("phone"), Some(class)),
            "ld-smartphone-symbolic"
        );
    }

    #[test]
    fn icon_unknown_hint_falls_back_to_class() {
        let class = MAJOR_PHONE << 8;
        assert_eq!(
            device_icon(Some("unknown-device"), Some(class)),
            "ld-smartphone-symbolic"
        );
    }

    #[test]
    fn device_type_computer_from_class() {
        let class = MAJOR_COMPUTER << 8;
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-computer"
        );
    }

    #[test]
    fn device_type_set_top_box_from_av_minor() {
        let class = (MAJOR_AV << 8) | (0x09 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-set-top-box"
        );
    }

    #[test]
    fn device_type_keyboard_from_hint() {
        assert_eq!(
            device_type_key(Some("input-keyboard"), None),
            "dropdown-bluetooth-type-keyboard"
        );
    }

    #[test]
    fn device_type_unknown_without_class() {
        assert_eq!(
            device_type_key(None, None),
            "dropdown-bluetooth-type-unknown"
        );
    }

    #[test]
    fn icon_no_hint_no_class_uses_bluetooth() {
        assert_eq!(device_icon(None, None), "ld-bluetooth-symbolic");
    }

    #[test]
    fn icon_unknown_hint_unknown_class_uses_bluetooth() {
        assert_eq!(
            device_icon(Some("alien-device"), Some(0xFF00)),
            "ld-bluetooth-symbolic"
        );
    }

    #[test]
    fn type_computer_laptop() {
        let class = (MAJOR_COMPUTER << 8) | (0x03 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-laptop"
        );
    }

    #[test]
    fn icon_computer_laptop() {
        let class = (MAJOR_COMPUTER << 8) | (0x03 << 2);
        assert_eq!(device_icon(None, Some(class)), "ld-laptop-symbolic");
    }

    #[test]
    fn type_computer_tablet() {
        let class = (MAJOR_COMPUTER << 8) | (0x07 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-computer-tablet"
        );
    }

    #[test]
    fn type_phone_smartphone() {
        let class = (MAJOR_PHONE << 8) | (0x03 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-smartphone"
        );
    }

    #[test]
    fn imaging_devices_from_their_class_of_device() {
        // As BlueZ reports them: the imaging flags are bits 4-7.
        for (class, key, icon) in [
            (
                0x00_0610,
                "dropdown-bluetooth-type-display",
                "ld-monitor-symbolic",
            ),
            (
                0x00_0620,
                "dropdown-bluetooth-type-camera",
                "ld-camera-symbolic",
            ),
            (
                0x00_0640,
                "dropdown-bluetooth-type-scanner",
                "ld-printer-symbolic",
            ),
            (
                0x04_0680,
                "dropdown-bluetooth-type-printer",
                "ld-printer-symbolic",
            ),
        ] {
            assert_eq!(device_type_key(None, Some(class)), key);
            assert_eq!(device_icon(None, Some(class)), icon);
        }
    }

    #[test]
    fn type_wearable_watch() {
        let class = (MAJOR_WEARABLE << 8) | (0x01 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-wrist-watch"
        );
    }

    #[test]
    fn type_wearable_glasses() {
        let class = (MAJOR_WEARABLE << 8) | (0x05 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-glasses"
        );
    }

    #[test]
    fn type_toy_robot() {
        let class = (MAJOR_TOY << 8) | (0x01 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-robot"
        );
    }

    #[test]
    fn type_av_loudspeaker() {
        let class = (MAJOR_AV << 8) | (0x05 << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-loudspeaker"
        );
    }

    #[test]
    fn type_av_video_camera() {
        let class = (MAJOR_AV << 8) | (0x0C << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-video-camera"
        );
    }

    #[test]
    fn type_av_camcorder() {
        let class = (MAJOR_AV << 8) | (0x0D << 2);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-camcorder"
        );
    }

    #[test]
    fn type_peripheral_combo_keyboard() {
        let class = (MAJOR_PERIPHERAL << 8) | (0x03 << 6);
        assert_eq!(
            device_type_key(None, Some(class)),
            "dropdown-bluetooth-type-combo-keyboard"
        );
    }

    #[test]
    fn passkey_zero_padded() {
        assert_eq!(format_passkey(0), "000000");
        assert_eq!(format_passkey(42), "000042");
        assert_eq!(format_passkey(123456), "123456");
        assert_eq!(format_passkey(999999), "999999");
    }

    #[test]
    fn service_name_audio_sink() {
        assert_eq!(
            service_name_key("0000110b-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-audio-sink"
        );
    }

    #[test]
    fn service_name_audio_source() {
        assert_eq!(
            service_name_key("0000110a-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-audio-source"
        );
    }

    #[test]
    fn service_name_handsfree() {
        assert_eq!(
            service_name_key("0000111e-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-handsfree"
        );
    }

    #[test]
    fn service_name_hid() {
        assert_eq!(
            service_name_key("00001124-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-input-device"
        );
    }

    #[test]
    fn service_name_uppercase_uuid() {
        assert_eq!(
            service_name_key("0000110B-0000-1000-8000-00805F9B34FB"),
            "dropdown-bluetooth-service-audio-sink"
        );
    }

    #[test]
    fn service_name_proprietary_uuid() {
        assert_eq!(
            service_name_key("a1b1c2d3-e4f5-6789-abcd-ef0123456789"),
            "dropdown-bluetooth-service-proprietary"
        );
    }

    #[test]
    fn service_name_unknown_standard_uuid() {
        assert_eq!(
            service_name_key("0000ffff-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-unknown"
        );
    }

    #[test]
    fn service_name_headset_variants() {
        assert_eq!(
            service_name_key("00001108-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-headset"
        );
        assert_eq!(
            service_name_key("00001112-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-headset"
        );
    }

    #[test]
    fn service_name_avrcp_variants() {
        assert_eq!(
            service_name_key("0000110c-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-remote-control"
        );
        assert_eq!(
            service_name_key("0000110e-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-remote-control"
        );
        assert_eq!(
            service_name_key("0000110f-0000-1000-8000-00805f9b34fb"),
            "dropdown-bluetooth-service-remote-control"
        );
    }
}
