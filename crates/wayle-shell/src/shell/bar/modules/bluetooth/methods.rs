use std::sync::Arc;

use relm4::ComponentController;
use wayle_bluetooth::BluetoothService;
use wayle_config::schemas::modules::BluetoothConfig;
use wayle_widgets::prelude::BarButtonInput;

use super::{
    BluetoothModule,
    helpers::{BluetoothContext, format_label, select_icon},
};

impl BluetoothModule {
    pub(super) fn compute_display(
        config: &BluetoothConfig,
        bt: &Option<Arc<BluetoothService>>,
    ) -> (String, String) {
        let Some(bt) = bt else {
            let ctx = BluetoothContext {
                available: false,
                enabled: false,
                discovering: false,
                connected_devices: &[],
            };
            return (select_icon(config, &ctx), format_label(&ctx));
        };

        let available = bt.available.get();
        let enabled = bt.enabled.get();
        let discovering = bt.discovering.get();
        let connected_devices = bt.connected.get();

        let ctx = BluetoothContext {
            available,
            enabled,
            discovering,
            connected_devices: &connected_devices,
        };

        (select_icon(config, &ctx), format_label(&ctx))
    }

    pub(super) fn update_display(
        &self,
        config: &BluetoothConfig,
        bt: &Option<Arc<BluetoothService>>,
    ) {
        let (icon, label) = Self::compute_display(config, bt);
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
        self.bar_button.emit(BarButtonInput::SetLabel(label));
    }
}
