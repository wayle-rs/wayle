use gtk::prelude::*;
use relm4::{gtk, prelude::*};

use super::{IwdDropdown, watchers};

impl IwdDropdown {
    pub(super) fn reset_station_watchers(&mut self, sender: &ComponentSender<Self>) {
        let token = self.station_watcher.reset();
        watchers::spawn_station_watchers(sender, &self.iwd, token);
    }

    /// Puts the power switch's knob at IWD's state, if it isn't there.
    ///
    /// Only when it differs, and not on every render: setting the knob cancels
    /// GtkSwitch's slide after a click, and with it the click.
    pub(super) fn sync_knob(&self, widgets: &<Self as Component>::Widgets) {
        let switch = &widgets.power_switch;
        if switch.is_active() != self.powered {
            switch.block_signal(&widgets.power_toggle_handler);
            switch.set_active(self.powered);
            switch.unblock_signal(&widgets.power_toggle_handler);
        }
    }

    /// Initiate a single scan. Results arrive reactively via IWD's signals, so
    /// the list updates without further action.
    pub(super) fn request_scan(&self) {
        if let Some(station) = self.iwd.station.get() {
            station.scan();
        }
    }
}
