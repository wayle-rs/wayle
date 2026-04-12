//! File picker for path-valued config properties.

mod row;
pub(crate) use row::*;

use gtk4::{gio, prelude::*};
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct FilePickerControl {
    property: ConfigProperty<String>,
    entry: gtk4::Entry,
    activate_id: gtk4::glib::SignalHandlerId,
    changed_id: gtk4::glib::SignalHandlerId,
    dirty_badge: gtk4::Label,
}

pub(crate) struct FilePickerInit {
    pub property: ConfigProperty<String>,
    pub dirty_badge: gtk4::Label,
}

#[derive(Debug)]
pub(crate) enum FilePickerMsg {
    Browse,
    FileSelected(String),
    Refresh,
}

impl SimpleComponent for FilePickerControl {
    type Init = FilePickerInit;
    type Input = FilePickerMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .hexpand(false)
            .valign(gtk4::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.add_css_class("file-picker");

        let entry = gtk4::Entry::builder()
            .text(init.property.get())
            .valign(gtk4::Align::Center)
            .build();

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = entry.connect_changed(move |_entry| {
            dirty_badge.set_visible(true);
        });

        let prop = init.property.clone();
        let output_sender = sender.output_sender().clone();
        let dirty_badge_commit = init.dirty_badge.clone();

        let activate_id = entry.connect_activate(move |entry| {
            prop.set(entry.text().to_string());
            dirty_badge_commit.set_visible(false);
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        let browse_button = gtk4::Button::builder()
            .icon_name("ld-folder-open-symbolic")
            .valign(gtk4::Align::Center)
            .build();
        browse_button.add_css_class("icon");
        browse_button.set_cursor_from_name(Some("pointer"));

        let input_sender = sender.input_sender().clone();
        browse_button.connect_clicked(move |_| {
            let _ = input_sender.send(FilePickerMsg::Browse);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            let _ = input_sender.send(FilePickerMsg::Refresh);
        });

        root.append(&entry);
        root.append(&browse_button);

        let model = Self {
            property: init.property,
            entry,
            activate_id,
            changed_id,
            dirty_badge: init.dirty_badge,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            FilePickerMsg::Browse => {
                let dialog = gtk4::FileDialog::new();
                let input_sender = sender.input_sender().clone();

                let root = self.entry.root();
                let window = root
                    .as_ref()
                    .and_then(|root| root.downcast_ref::<gtk4::Window>());

                dialog.open(window, gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result
                        && let Some(path) = file.path()
                    {
                        let path_str = path.to_string_lossy().into_owned();
                        let _ = input_sender.send(FilePickerMsg::FileSelected(path_str));
                    }
                });
            }

            FilePickerMsg::FileSelected(path) => {
                self.entry.set_text(&path);
                self.property.set(path);
                self.dirty_badge.set_visible(false);
                let _ = sender.output(ControlOutput::ValueChanged);
            }

            FilePickerMsg::Refresh => {
                self.entry.block_signal(&self.activate_id);
                self.entry.block_signal(&self.changed_id);
                self.entry.set_text(&self.property.get());
                self.entry.unblock_signal(&self.changed_id);
                self.entry.unblock_signal(&self.activate_id);
            }
        }
    }
}
