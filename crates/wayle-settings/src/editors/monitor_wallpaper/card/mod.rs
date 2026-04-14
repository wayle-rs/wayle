//! Single monitor wallpaper card. Shows monitor name, wallpaper path,
//! and fit mode in a compact card layout.

mod helpers;
mod methods;

use relm4::{factory::FactoryView, gtk, gtk::prelude::*, prelude::*};
use wayle_config::schemas::wallpaper::{FitMode, MonitorWallpaperConfig};
use wayle_i18n::t;

use self::helpers::{fit_mode_index, fit_mode_labels};

pub(super) struct MonitorCard {
    pub(super) name: String,
    pub(super) wallpaper: String,
    pub(super) fit_mode: FitMode,
    pub(super) name_entry: gtk::Entry,
    pub(super) wallpaper_entry: gtk::Entry,
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
    pub(crate) fn to_config(&self) -> MonitorWallpaperConfig {
        MonitorWallpaperConfig {
            name: self.name.clone(),
            fit_mode: self.fit_mode,
            wallpaper: self.wallpaper.clone(),
        }
    }
}

#[relm4::factory(pub(super))]
impl FactoryComponent for MonitorCard {
    type Init = MonitorWallpaperConfig;
    type Input = MonitorCardMsg;
    type Output = MonitorCardOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "monitor-card",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Box {
                add_css_class: "monitor-card-header",
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "monitor-card-label",
                    set_label: &t("settings-wallpaper-monitor-label"),
                },

                #[name = "name_entry"]
                gtk::Entry {
                    add_css_class: "monitor-name-entry",
                    set_placeholder_text: Some("DP-1"),
                    set_hexpand: true,
                    connect_changed => MonitorCardMsg::NameChanged,
                },

                #[name = "remove_button"]
                gtk::Button {
                    add_css_class: "ghost-icon",
                    set_icon_name: "ld-trash-2-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked[sender, index] => move |_button| {
                        let _ = sender.output(MonitorCardOutput::Remove(index.clone()));
                    },
                },
            },

            #[name = "body"]
            gtk::Box {
                add_css_class: "monitor-card-body",
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "monitor-card-label",
                    set_label: &t("settings-wallpaper-path-label"),
                },

                #[name = "wallpaper_entry"]
                gtk::Entry {
                    add_css_class: "monitor-wallpaper-entry",
                    set_placeholder_text: Some(&t("settings-wallpaper-path-placeholder")),
                    set_hexpand: true,
                    connect_changed => MonitorCardMsg::WallpaperChanged,
                },

                #[name = "browse_button"]
                gtk::Button {
                    add_css_class: "icon",
                    set_icon_name: "ld-folder-open-symbolic",
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => MonitorCardMsg::Browse,
                },

                #[name = "fit_dropdown"]
                gtk::DropDown {
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
            name_entry: gtk::Entry::new(),
            wallpaper_entry: gtk::Entry::new(),
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
            gtk::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());
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
            MonitorCardMsg::NameChanged => self.on_name_changed(&sender),
            MonitorCardMsg::WallpaperChanged => self.on_wallpaper_changed(&sender),
            MonitorCardMsg::FitModeSelected(index) => self.on_fit_mode_selected(index, &sender),
            MonitorCardMsg::Browse => self.on_browse(&sender),
            MonitorCardMsg::FileSelected(path) => self.on_file_selected(path, &sender),
        }
    }
}
