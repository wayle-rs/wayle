//! Message handlers for `MonitorCard`.

use relm4::{
    factory::FactorySender,
    gtk,
    gtk::{gio, prelude::*},
};

use super::{MonitorCard, MonitorCardMsg, MonitorCardOutput, helpers::fit_mode_from_index};

impl MonitorCard {
    pub(super) fn on_name_changed(&mut self, sender: &FactorySender<Self>) {
        self.name = self.name_entry.text().to_string();
        let _ = sender.output(MonitorCardOutput::Changed);
    }

    pub(super) fn on_wallpaper_changed(&mut self, sender: &FactorySender<Self>) {
        self.wallpaper = self.wallpaper_entry.text().to_string();
        let _ = sender.output(MonitorCardOutput::Changed);
    }

    pub(super) fn on_fit_mode_selected(&mut self, index: u32, sender: &FactorySender<Self>) {
        let Some(mode) = fit_mode_from_index(index) else {
            return;
        };
        self.fit_mode = mode;
        let _ = sender.output(MonitorCardOutput::Changed);
    }

    pub(super) fn on_browse(&mut self, sender: &FactorySender<Self>) {
        let dialog = gtk::FileDialog::new();
        let input_sender = sender.input_sender().clone();

        let root = self.wallpaper_entry.root();
        let window = root
            .as_ref()
            .and_then(|root| root.downcast_ref::<gtk::Window>());

        dialog.open(window, gio::Cancellable::NONE, move |result| {
            if let Ok(file) = result
                && let Some(path) = file.path()
            {
                let path_str = path.to_string_lossy().into_owned();
                let _ = input_sender.send(MonitorCardMsg::FileSelected(path_str));
            }
        });
    }

    pub(super) fn on_file_selected(&mut self, path: String, sender: &FactorySender<Self>) {
        self.wallpaper_entry.set_text(&path);
        self.wallpaper = path;
        let _ = sender.output(MonitorCardOutput::Changed);
    }
}
