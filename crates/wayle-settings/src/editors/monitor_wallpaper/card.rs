//! Single monitor wallpaper card. Shows monitor name, wallpaper path,
//! and fit mode in a compact card layout.

use gtk4::{gio, prelude::*};
use relm4::{factory::FactoryView, prelude::*};
use serde::{Deserialize, de::value::StrDeserializer};
use wayle_config::{
    EnumVariants,
    schemas::wallpaper::{FitMode, MonitorWallpaperConfig},
};
use wayle_i18n::t;

pub(super) struct MonitorCard {
    name: String,
    wallpaper: String,
    fit_mode: FitMode,
    name_entry: gtk4::Entry,
    wallpaper_entry: gtk4::Entry,
}

#[derive(Debug)]
pub(super) enum MonitorCardMsg {
    NameChanged,
    WallpaperChanged,
    FitModeSelected(u32),
    Browse,
    FileSelected(String),
}

#[derive(Debug)]
pub(super) enum MonitorCardOutput {
    Remove(DynamicIndex),
    Changed,
}

impl MonitorCard {
    pub fn to_config(&self) -> MonitorWallpaperConfig {
        MonitorWallpaperConfig {
            name: self.name.clone(),
            fit_mode: self.fit_mode,
            wallpaper: self.wallpaper.clone(),
        }
    }
}

fn fit_mode_labels() -> Vec<String> {
    FitMode::variants()
        .iter()
        .map(|variant| {
            let resolved = t(variant.fluent_key);

            if resolved == variant.fluent_key {
                variant.value.to_owned()
            } else {
                resolved
            }
        })
        .collect()
}

fn fit_mode_index(mode: &FitMode) -> u32 {
    FitMode::variants()
        .iter()
        .position(|variant| fit_mode_from_value(variant.value).as_ref() == Some(mode))
        .unwrap_or(0) as u32
}

fn fit_mode_from_index(index: u32) -> Option<FitMode> {
    let variant = FitMode::variants().get(index as usize)?;
    fit_mode_from_value(variant.value)
}

fn fit_mode_from_value(value: &str) -> Option<FitMode> {
    let deserializer: StrDeserializer<'_, serde::de::value::Error> = StrDeserializer::new(value);
    FitMode::deserialize(deserializer).ok()
}

#[relm4::factory(pub(super))]
impl FactoryComponent for MonitorCard {
    type Init = MonitorWallpaperConfig;
    type Input = MonitorCardMsg;
    type Output = MonitorCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk4::Box;

    view! {
        #[root]
        gtk4::Box {
            add_css_class: "monitor-card",
            set_orientation: gtk4::Orientation::Vertical,

            #[name = "header"]
            gtk4::Box {
                add_css_class: "monitor-card-header",
                set_orientation: gtk4::Orientation::Horizontal,

                gtk4::Label {
                    add_css_class: "monitor-card-label",
                    set_label: &t("settings-wallpaper-monitor-label"),
                },

                #[name = "name_entry"]
                gtk4::Entry {
                    add_css_class: "monitor-name-entry",
                    set_placeholder_text: Some("DP-1"),
                    set_hexpand: true,
                    connect_changed => MonitorCardMsg::NameChanged,
                },

                #[name = "remove_button"]
                gtk4::Button {
                    add_css_class: "ghost-icon",
                    set_icon_name: "ld-trash-2-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked[sender, index] => move |_button| {
                        let _ = sender.output(MonitorCardOutput::Remove(index.clone()));
                    },
                },
            },

            #[name = "body"]
            gtk4::Box {
                add_css_class: "monitor-card-body",
                set_orientation: gtk4::Orientation::Horizontal,

                gtk4::Label {
                    add_css_class: "monitor-card-label",
                    set_label: &t("settings-wallpaper-path-label"),
                },

                #[name = "wallpaper_entry"]
                gtk4::Entry {
                    add_css_class: "monitor-wallpaper-entry",
                    set_placeholder_text: Some(&t("settings-wallpaper-path-placeholder")),
                    set_hexpand: true,
                    connect_changed => MonitorCardMsg::WallpaperChanged,
                },

                #[name = "browse_button"]
                gtk4::Button {
                    add_css_class: "icon",
                    set_icon_name: "ld-folder-open-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => MonitorCardMsg::Browse,
                },

                #[name = "fit_dropdown"]
                gtk4::DropDown {
                    add_css_class: "monitor-fit-dropdown",
                },
            },
        }
    }

    fn init_model(config: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name: config.name,
            wallpaper: config.wallpaper,
            fit_mode: config.fit_mode,
            name_entry: gtk4::Entry::new(),
            wallpaper_entry: gtk4::Entry::new(),
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        widgets.name_entry.set_text(&self.name);
        widgets.wallpaper_entry.set_text(&self.wallpaper);

        let labels = fit_mode_labels();
        let string_list =
            gtk4::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());
        widgets.fit_dropdown.set_model(Some(&string_list));
        widgets
            .fit_dropdown
            .set_selected(fit_mode_index(&self.fit_mode));

        let fit_sender = sender.input_sender().clone();
        widgets
            .fit_dropdown
            .connect_selected_notify(move |dropdown| {
                let _ = fit_sender.send(MonitorCardMsg::FitModeSelected(dropdown.selected()));
            });

        self.name_entry = widgets.name_entry.clone();
        self.wallpaper_entry = widgets.wallpaper_entry.clone();

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            MonitorCardMsg::NameChanged => {
                self.name = self.name_entry.text().to_string();
                let _ = sender.output(MonitorCardOutput::Changed);
            }

            MonitorCardMsg::WallpaperChanged => {
                self.wallpaper = self.wallpaper_entry.text().to_string();
                let _ = sender.output(MonitorCardOutput::Changed);
            }

            MonitorCardMsg::FitModeSelected(index) => {
                if let Some(mode) = fit_mode_from_index(index) {
                    self.fit_mode = mode;
                    let _ = sender.output(MonitorCardOutput::Changed);
                }
            }

            MonitorCardMsg::Browse => {
                let dialog = gtk4::FileDialog::new();
                let input_sender = sender.input_sender().clone();

                let root = self.wallpaper_entry.root();
                let window = root
                    .as_ref()
                    .and_then(|root| root.downcast_ref::<gtk4::Window>());

                dialog.open(window, gio::Cancellable::NONE, move |result| {
                    if let Ok(file) = result
                        && let Some(path) = file.path()
                    {
                        let path_str = path.to_string_lossy().into_owned();
                        let _ = input_sender.send(MonitorCardMsg::FileSelected(path_str));
                    }
                });
            }

            MonitorCardMsg::FileSelected(path) => {
                self.wallpaper_entry.set_text(&path);
                self.wallpaper = path;
                let _ = sender.output(MonitorCardOutput::Changed);
            }
        }
    }
}
