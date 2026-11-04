use relm4::ComponentSender;
use tracing::warn;
use wayle_bluetooth::types::RadioBlock;
use wayle_power_profiles::types::profile::PowerProfile;

use super::{QuickActionsSection, messages::QuickActionsCmd};

impl QuickActionsSection {
    pub(super) fn toggle_wifi(&self, sender: &ComponentSender<Self>) {
        let Some(network) = self.network.clone() else {
            return;
        };

        let target = !self.wifi_active;

        sender.oneshot_command(async move {
            if let Some(wifi) = network.wifi.get()
                && let Err(err) = wifi.set_enabled(target).await
            {
                warn!(error = %err, "wifi toggle failed");
            }
            QuickActionsCmd::WifiChanged(target)
        });
    }

    /// Whether Bluetooth is on or turning on: the service's `enabled`, which
    /// BlueZ updates as soon as it accepts a power request, and reverts if it
    /// fails.
    pub(super) fn bluetooth_active(&self) -> bool {
        self.bluetooth
            .get()
            .is_some_and(|bluetooth| bluetooth.enabled.get())
    }

    /// Whether a Bluetooth adapter is present.
    pub(super) fn has_bluetooth(&self) -> bool {
        self.bluetooth
            .get()
            .is_some_and(|bluetooth| bluetooth.available.get())
    }

    /// Whether a hardware switch blocks a Bluetooth radio, which keeps
    /// Bluetooth off (as KDE reckons it) whatever the tile does.
    pub(super) fn bluetooth_hardware_blocked(&self) -> bool {
        self.bluetooth
            .get()
            .is_some_and(|bluetooth| bluetooth.radio_block.get() == RadioBlock::Hardware)
    }

    /// Powers Bluetooth on or off.
    pub(super) fn toggle_bluetooth(&self) {
        self.set_bluetooth(!self.bluetooth_active());
    }

    /// Requests Bluetooth on or off. Explicit rather than a toggle: the shown
    /// state follows BlueZ and can lag a request that is still in flight.
    fn set_bluetooth(&self, enabled: bool) {
        let Some(bluetooth) = self.bluetooth.get() else {
            return;
        };

        if enabled {
            bluetooth.enable();
        } else {
            bluetooth.disable();
        }
    }

    pub(super) fn toggle_airplane(&mut self, sender: &ComponentSender<Self>) {
        let target = !self.airplane_active;

        if target {
            self.pre_airplane_wifi = self.wifi_active;
            self.pre_airplane_bt = self.bluetooth_active();

            if self.wifi_active {
                self.toggle_wifi(sender);
            }
            if self.pre_airplane_bt {
                self.set_bluetooth(false);
            }
        } else {
            if self.pre_airplane_wifi {
                self.toggle_wifi(sender);
            }
            if self.pre_airplane_bt {
                self.set_bluetooth(true);
            }
        }

        self.airplane_active = target;
    }

    pub(super) fn toggle_dnd(&self, sender: &ComponentSender<Self>) {
        let Some(notification) = self.notification.clone() else {
            return;
        };

        let target = !self.dnd_active;

        sender.oneshot_command(async move {
            notification.set_dnd(target);
            QuickActionsCmd::DndChanged(target)
        });
    }

    pub(super) fn toggle_idle_inhibit(&self) {
        let state = self.idle_inhibit.state();
        if state.active.get() {
            state.disable();
        } else {
            state.enable(false);
        }
    }

    pub(super) fn toggle_power_saver(&self, sender: &ComponentSender<Self>) {
        let Some(power_profiles) = self.power_profiles.get() else {
            return;
        };

        let target = if self.power_saver_active {
            PowerProfile::Balanced
        } else {
            PowerProfile::PowerSaver
        };

        sender.oneshot_command(async move {
            if let Err(err) = power_profiles
                .power_profiles
                .set_active_profile(target)
                .await
            {
                warn!(error = %err, "power profile toggle failed");
            }
            QuickActionsCmd::PowerSaverChanged(target == PowerProfile::PowerSaver)
        });
    }
}
