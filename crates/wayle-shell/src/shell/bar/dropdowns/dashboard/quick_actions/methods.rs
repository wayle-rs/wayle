use std::sync::Arc;

use relm4::ComponentSender;
use tracing::warn;
use wayle_core::Property;
use wayle_iwd::IwdService;
use wayle_network::NetworkService;
use wayle_power_profiles::types::profile::PowerProfile;

use super::{QuickActionsSection, messages::QuickActionsCmd};

/// The WiFi enabled/powered `Property` from whichever backend is in use, or
/// `None` when no WiFi device is present.
///
/// NetworkManager takes precedence; IWD is only consulted when the Network
/// service is entirely absent.
pub(super) fn wifi_enabled_property(
    network: &Option<Arc<NetworkService>>,
    iwd: &Option<Arc<IwdService>>,
) -> Option<Property<bool>> {
    if let Some(network) = network {
        network.wifi.get().map(|wifi| wifi.enabled.clone())
    } else {
        iwd.as_ref()
            .and_then(|iwd| iwd.station.get())
            .map(|station| station.powered.clone())
    }
}

impl QuickActionsSection {
    pub(super) fn toggle_wifi(&self, sender: &ComponentSender<Self>) {
        let target = !self.wifi_active;

        // NetworkManager takes precedence; IWD is the fallback — the same ordering
        // as `wifi_enabled_property`. The toggle is hand-rolled rather than sharing
        // that helper because the two backends expose different setters
        // (`set_enabled` vs `set_powered`) with different error types.
        if let Some(network) = self.network.clone() {
            sender.oneshot_command(async move {
                if let Some(wifi) = network.wifi.get()
                    && let Err(err) = wifi.set_enabled(target).await
                {
                    warn!(error = %err, "wifi toggle failed");
                }
                QuickActionsCmd::WifiChanged(target)
            });
        } else if let Some(iwd) = self.iwd.clone() {
            sender.oneshot_command(async move {
                if let Some(station) = iwd.station.get()
                    && let Err(err) = station.set_powered(target).await
                {
                    warn!(error = %err, "wifi toggle failed");
                }
                QuickActionsCmd::WifiChanged(target)
            });
        }
    }

    pub(super) fn toggle_bluetooth(&self, sender: &ComponentSender<Self>) {
        let Some(bluetooth) = self.bluetooth.get() else {
            return;
        };

        let target = !self.bluetooth_active;

        sender.oneshot_command(async move {
            let result = if target {
                bluetooth.enable().await
            } else {
                bluetooth.disable().await
            };
            if let Err(err) = result {
                warn!(error = %err, "bluetooth toggle failed");
            }
            QuickActionsCmd::BluetoothChanged(target)
        });
    }

    pub(super) fn toggle_airplane(&mut self, sender: &ComponentSender<Self>) {
        let target = !self.airplane_active;

        if target {
            self.pre_airplane_wifi = self.wifi_active;
            self.pre_airplane_bt = self.bluetooth_active;

            if self.wifi_active {
                self.toggle_wifi(sender);
            }
            if self.bluetooth_active {
                self.toggle_bluetooth(sender);
            }
        } else {
            if self.pre_airplane_wifi {
                self.toggle_wifi(sender);
            }
            if self.pre_airplane_bt {
                self.toggle_bluetooth(sender);
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
