use std::sync::Arc;

use relm4::ComponentSender;
use tokio_util::sync::CancellationToken;
use wayle_bluetooth::BluetoothService;
use wayle_core::DeferredService;
use wayle_network::{NetworkService, wifi::Wifi};
use wayle_notification::NotificationService;
use wayle_power_profiles::{PowerProfilesService, types::profile::PowerProfile};
use wayle_widgets::{watch, watch_cancellable, watch_deferred};

use super::{QuickActionsSection, messages::QuickActionsCmd};
use crate::services::IdleInhibitService;

pub(super) fn spawn(
    sender: &ComponentSender<QuickActionsSection>,
    network: &Option<Arc<NetworkService>>,
    bluetooth: &DeferredService<BluetoothService>,
    notification: &Option<Arc<NotificationService>>,
    power_profiles: &DeferredService<PowerProfilesService>,
    idle_inhibit: &Arc<IdleInhibitService>,
) {
    if let Some(network) = network {
        let wifi_prop = network.wifi.clone();

        watch!(sender, [wifi_prop.watch()], |out| {
            let has_wifi = wifi_prop.get().is_some();
            let _ = out.send(QuickActionsCmd::WifiAvailabilityChanged(has_wifi));
        });
    }

    spawn_bluetooth_availability(sender, bluetooth);

    if let Some(notification) = notification {
        let dnd = notification.dnd.clone();

        watch!(sender, [dnd.watch()], |out| {
            let _ = out.send(QuickActionsCmd::DndChanged(dnd.get()));
        });
    }

    let active = idle_inhibit.state().active.clone();

    watch!(sender, [active.watch()], |out| {
        let _ = out.send(QuickActionsCmd::IdleInhibitChanged(active.get()));
    });

    spawn_power_profile_availability(sender, power_profiles);
}

pub(super) fn spawn_bluetooth_watchers(
    sender: &ComponentSender<QuickActionsSection>,
    service: &Arc<BluetoothService>,
) {
    let enabled = service.enabled.clone();
    let available = service.available.clone();
    let radio_block = service.radio_block.clone();

    watch!(
        sender,
        [enabled.watch(), available.watch(), radio_block.watch()],
        |out| {
            let _ = out.send(QuickActionsCmd::BluetoothChanged);
        }
    );
}

pub(super) fn spawn_bluetooth_availability(
    sender: &ComponentSender<QuickActionsSection>,
    bluetooth: &DeferredService<BluetoothService>,
) {
    watch_deferred!(sender, bluetooth, QuickActionsCmd::BluetoothReady);
}

pub(super) fn spawn_power_profile_availability(
    sender: &ComponentSender<QuickActionsSection>,
    power_profiles: &DeferredService<PowerProfilesService>,
) {
    watch_deferred!(sender, power_profiles, QuickActionsCmd::PowerProfilesReady);
}

pub(super) fn spawn_power_profile_watcher(
    sender: &ComponentSender<QuickActionsSection>,
    service: &Arc<PowerProfilesService>,
    token: CancellationToken,
) {
    let profile = service.power_profiles.active_profile.clone();

    watch_cancellable!(sender, token, [profile.watch()], |out| {
        let is_saver = profile.get() == PowerProfile::PowerSaver;
        let _ = out.send(QuickActionsCmd::PowerSaverChanged(is_saver));
    });
}

pub(super) fn spawn_wifi_enabled_watcher(
    sender: &ComponentSender<QuickActionsSection>,
    wifi: &Arc<Wifi>,
    token: CancellationToken,
) {
    let enabled = wifi.enabled.clone();

    watch_cancellable!(sender, token, [enabled.watch()], |out| {
        let _ = out.send(QuickActionsCmd::WifiChanged(enabled.get()));
    });
}
