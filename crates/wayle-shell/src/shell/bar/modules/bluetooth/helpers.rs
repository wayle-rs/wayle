use std::sync::Arc;

use wayle_bluetooth::core::device::Device;
use wayle_config::schemas::modules::BluetoothConfig;

use crate::i18n::t;

pub(crate) struct BluetoothContext<'a> {
    pub(crate) available: bool,
    pub(crate) enabled: bool,
    pub(crate) discovering: bool,
    pub(crate) connected_devices: &'a [Arc<Device>],
}

pub(crate) fn select_icon(config: &BluetoothConfig, ctx: &BluetoothContext<'_>) -> String {
    if !ctx.available || !ctx.enabled {
        return config.disabled_icon.get().clone();
    }

    if ctx.discovering {
        return config.searching_icon.get().clone();
    }

    if ctx.connected_devices.is_empty() {
        config.disconnected_icon.get().clone()
    } else {
        config.connected_icon.get().clone()
    }
}

pub(crate) fn format_label(ctx: &BluetoothContext<'_>) -> String {
    if !ctx.available || !ctx.enabled {
        return t!("bar-bluetooth-disabled");
    }

    match ctx.connected_devices.len() {
        0 => t!("bar-bluetooth-disconnected"),
        1 => ctx.connected_devices[0].info.get().alias.clone(),
        n => t!("bar-bluetooth-connected-count", count = n),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_config() -> BluetoothConfig {
        BluetoothConfig::default()
    }

    #[test]
    fn icon_disabled_when_unavailable() {
        let config = mock_config();
        let ctx = BluetoothContext {
            available: false,
            enabled: true,
            discovering: false,
            connected_devices: &[],
        };
        assert_eq!(select_icon(&config, &ctx), config.disabled_icon.get());
    }

    #[test]
    fn icon_disabled_when_powered_off() {
        let config = mock_config();
        let ctx = BluetoothContext {
            available: true,
            enabled: false,
            discovering: false,
            connected_devices: &[],
        };
        assert_eq!(select_icon(&config, &ctx), config.disabled_icon.get());
    }

    #[test]
    fn icon_searching_when_discovering() {
        let config = mock_config();
        let ctx = BluetoothContext {
            available: true,
            enabled: true,
            discovering: true,
            connected_devices: &[],
        };
        assert_eq!(select_icon(&config, &ctx), config.searching_icon.get());
    }

    #[test]
    fn icon_disconnected_when_no_devices() {
        let config = mock_config();
        let ctx = BluetoothContext {
            available: true,
            enabled: true,
            discovering: false,
            connected_devices: &[],
        };
        assert_eq!(select_icon(&config, &ctx), config.disconnected_icon.get());
    }

    #[test]
    fn label_shows_disabled_when_powered_off() {
        let ctx = BluetoothContext {
            available: true,
            enabled: false,
            discovering: false,
            connected_devices: &[],
        };
        assert!(!format_label(&ctx).is_empty());
    }

    #[test]
    fn label_disconnected_when_no_devices() {
        let ctx = BluetoothContext {
            available: true,
            enabled: true,
            discovering: false,
            connected_devices: &[],
        };
        assert!(!format_label(&ctx).is_empty());
    }
}
