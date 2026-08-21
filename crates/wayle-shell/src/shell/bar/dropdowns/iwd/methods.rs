use relm4::prelude::*;
use tracing::warn;

use super::{IwdDropdown, watchers};

impl IwdDropdown {
    pub(super) fn reset_station_watchers(&mut self, sender: &ComponentSender<Self>) {
        let token = self.station_watcher.reset();
        watchers::spawn_station_watchers(sender, &self.iwd, token);
    }

    pub(super) fn toggle_powered(&mut self, active: bool, sender: &ComponentSender<Self>) {
        self.powered = active;

        let iwd = self.iwd.clone();

        sender.command(move |_out, _shutdown| async move {
            if let Some(station) = iwd.station.get()
                && let Err(err) = station.set_powered(active).await
            {
                warn!(error = %err, "wifi toggle failed");
            }
        });
    }

    /// Initiate a single scan. Results arrive reactively via IWD's ObjectManager
    /// and `Scanning` signals, so the list updates without further action.
    pub(super) fn request_scan(&self, sender: &ComponentSender<Self>) {
        let iwd = self.iwd.clone();

        sender.command(move |_out, _shutdown| async move {
            if let Some(station) = iwd.station.get() {
                // Ignored if already scanning or the device is powered off.
                let _ = station.scan().await;
            }
        });
    }
}
