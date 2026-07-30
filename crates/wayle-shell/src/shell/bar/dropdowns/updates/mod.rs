mod factory;
mod helpers;
mod messages;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
use self::messages::{UpdatesDropdownCmd, UpdatesDropdownInit, UpdatesDropdownInput};
use crate::shell::bar::dropdowns::scaled_dimension;

const BASE_WIDTH: f32 = 340.0;
const BASE_HEIGHT: f32 = 310.0;

pub(crate) struct UpdatesDropdown {
    #[allow(dead_code)]
    config: Arc<ConfigService>,

    scaled_width: i32,
    scaled_height: i32,

    pacman_count: String,
    aur_count: String,
    flatpak_count: String,
    total_count: String,
    status_text: String,
}

#[relm4::component(pub(crate))]
impl Component for UpdatesDropdown {
    type Init = UpdatesDropdownInit;
    type Input = UpdatesDropdownInput;
    type Output = ();
    type CommandOutput = UpdatesDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "sysinfo-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_icon_name: Some("md-package_2-symbolic"),
                        set_visible: true,
                    },
                    #[template_child]
                    label {
                        set_label: "System Updates",
                    },
                },

                #[template]
                DropdownContent {
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,
                        set_margin_top: 4,
                        set_margin_bottom: 8,

                        // ── Update Counts Card ──────────────────
                        gtk::Box {
                            add_css_class: "card",
                            add_css_class: "sysinfo-card",
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Box {
                                add_css_class: "sysinfo-card-header",
                                gtk::Image {
                                    set_icon_name: Some("md-inventory_2-symbolic"),
                                    add_css_class: "sysinfo-header-icon",
                                    set_pixel_size: 18,
                                },
                                gtk::Label {
                                    set_label: "PACKAGES",
                                    add_css_class: "sysinfo-header-label",
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 1,
                                add_css_class: "sysinfo-stat-list",

                                gtk::Box { add_css_class: "sysinfo-stat",
                                    gtk::Label { set_label: "Official", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                    gtk::Label { #[watch] set_label: &model.pacman_count, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                },
                                gtk::Box { add_css_class: "sysinfo-stat",
                                    gtk::Label { set_label: "AUR", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                    gtk::Label { #[watch] set_label: &model.aur_count, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                },
                                gtk::Box { add_css_class: "sysinfo-stat",
                                    gtk::Label { set_label: "Flatpak", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                    gtk::Label { #[watch] set_label: &model.flatpak_count, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                },
                                gtk::Separator {},
                                gtk::Box { add_css_class: "sysinfo-stat",
                                    gtk::Label { set_label: "Total", add_css_class: "sysinfo-stat-key", set_hexpand: true, set_xalign: 0.0, },
                                    gtk::Label { #[watch] set_label: &model.total_count, add_css_class: "sysinfo-stat-val", set_xalign: 1.0, },
                                },
                            },
                        },

                        // ── Status ──────────────────────────────
                        gtk::Label {
                            #[watch]
                            set_label: &model.status_text,
                            add_css_class: "sysinfo-stat-key",
                            set_xalign: 0.5,
                            #[watch]
                            set_visible: !model.status_text.is_empty(),
                        },

                        // ── Action Buttons Card ─────────────────
                        gtk::Box {
                            add_css_class: "card",
                            add_css_class: "sysinfo-card",
                            set_orientation: gtk::Orientation::Horizontal,
                            set_homogeneous: true,
                            set_spacing: 8,
                            set_margin_start: 4,
                            set_margin_end: 4,
                            set_margin_top: 4,
                            set_margin_bottom: 8,

                            gtk::Button {
                                add_css_class: "quick-action",
                                set_hexpand: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_spacing: 4,

                                    gtk::Box {
                                        add_css_class: "quick-action-icon",
                                        set_halign: gtk::Align::Center,
                                        gtk::Image {
                                            set_icon_name: Some("md-deployed_code_update-symbolic"),
                                            set_pixel_size: 24,
                                        },
                                    },
                                    gtk::Label {
                                        set_label: "Refresh",
                                        add_css_class: "quick-action-label",
                                        set_halign: gtk::Align::Center,
                                    },
                                },

                                connect_clicked => UpdatesDropdownInput::Refresh,
                            },

                            gtk::Button {
                                add_css_class: "quick-action",
                                set_hexpand: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_spacing: 4,

                                    gtk::Box {
                                        add_css_class: "quick-action-icon",
                                        set_halign: gtk::Align::Center,
                                        gtk::Image {
                                            set_icon_name: Some("md-download-symbolic"),
                                            set_pixel_size: 24,
                                        },
                                    },
                                    gtk::Label {
                                        set_label: "Update All",
                                        add_css_class: "quick-action-label",
                                        set_halign: gtk::Align::Center,
                                    },
                                },

                                connect_clicked => UpdatesDropdownInput::UpdateAll,
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn(&sender, &init.config);

        let model = Self {
            config: init.config,

            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),

            pacman_count: String::from("..."),
            aur_count: String::from("..."),
            flatpak_count: String::from("..."),
            total_count: String::from("..."),
            status_text: String::from("Checking for updates..."),
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            UpdatesDropdownInput::Refresh => {
                // Re-show popover (GTK may close it on button activation)
                root.popup();

                self.status_text = String::from("Checking for updates...");
                self.pacman_count = String::from("...");
                self.aur_count = String::from("...");
                self.flatpak_count = String::from("...");
                self.total_count = String::from("...");

                let official_cmd = self.config.config().modules.updates.check_official_command.get().clone();
                let aur_cmd = self.config.config().modules.updates.check_aur_command.get().clone();
                let flatpak_cmd = self.config.config().modules.updates.check_flatpak_command.get().clone();

                sender.command(move |out, shutdown| {
                    shutdown
                        .register(async move {
                            let _ = out.send(UpdatesDropdownCmd::SetChecking(true));

                            let (pacman, aur, flatpak) = tokio::join!(
                                helpers::run_count_command(&official_cmd),
                                helpers::run_count_command(&aur_cmd),
                                helpers::run_count_command(&flatpak_cmd),
                            );

                            let _ = out.send(UpdatesDropdownCmd::UpdateCounts { pacman, aur, flatpak });
                            let _ = out.send(UpdatesDropdownCmd::SetChecking(false));

                            // Signal bar module to update its label too
                            crate::shell::bar::modules::updates::helpers::REFRESH_NOTIFY.notify_one();
                        })
                        .drop_on_shutdown()
                });
            }
            UpdatesDropdownInput::UpdateAll => {
                let update_cmd = self.config.config().modules.updates.update_command.get().clone();
                let official_cmd = self.config.config().modules.updates.check_official_command.get().clone();
                let aur_cmd = self.config.config().modules.updates.check_aur_command.get().clone();
                let flatpak_cmd = self.config.config().modules.updates.check_flatpak_command.get().clone();

                sender.command(move |out, shutdown| {
                    shutdown
                        .register(async move {
                            // Wait for terminal to close
                            if helpers::spawn_update_in_terminal(&update_cmd).await {
                                // Re-check counts after update finished
                                let _ = out.send(UpdatesDropdownCmd::SetChecking(true));

                                let (pacman, aur, flatpak) = tokio::join!(
                                    helpers::run_count_command(&official_cmd),
                                    helpers::run_count_command(&aur_cmd),
                                    helpers::run_count_command(&flatpak_cmd),
                                );

                                let _ = out.send(UpdatesDropdownCmd::UpdateCounts { pacman, aur, flatpak });
                                let _ = out.send(UpdatesDropdownCmd::SetChecking(false));

                                // Signal bar module to update its label
                                crate::shell::bar::modules::updates::helpers::REFRESH_NOTIFY.notify_one();
                            }
                        })
                        .drop_on_shutdown()
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            UpdatesDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }
            UpdatesDropdownCmd::UpdateCounts { pacman, aur, flatpak } => {
                let total = pacman + aur + flatpak;
                self.pacman_count = pacman.to_string();
                self.aur_count = aur.to_string();
                self.flatpak_count = flatpak.to_string();
                self.total_count = total.to_string();
            }
            UpdatesDropdownCmd::SetChecking(checking) => {
                if checking {
                    self.status_text = String::from("Checking for updates...");
                } else {
                    self.status_text = String::new();
                }
            }
        }
    }
}
