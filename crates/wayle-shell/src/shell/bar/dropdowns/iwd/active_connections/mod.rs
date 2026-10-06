mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_iwd::IwdService;
use wayle_widgets::{WatcherToken, prelude::*};

use self::messages::{ActiveConnectionsCmd, StationState};
pub(crate) use self::messages::{ActiveConnectionsInit, ActiveConnectionsInput};
use crate::i18n::t;

pub(crate) struct ActiveConnections {
    iwd: Arc<IwdService>,
    config: Arc<ConfigService>,
    station: StationState,
    station_watcher: WatcherToken,
}

#[relm4::component(pub(crate))]
impl Component for ActiveConnections {
    type Init = ActiveConnectionsInit;
    type Input = ActiveConnectionsInput;
    type Output = ();
    type CommandOutput = ActiveConnectionsCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[watch]
            set_visible: model.card_visible(),

            #[name = "section_label"]
            gtk::Label {
                add_css_class: "section-label",
                set_halign: gtk::Align::Start,
                set_label: &t!("dropdown-iwd-active-connection"),
            },

            #[template]
            Card {
                add_css_class: "network-connections-group",
                set_orientation: gtk::Orientation::Vertical,

                #[name = "wifi_card"]
                gtk::Box {
                    add_css_class: "network-connection-card",
                    #[watch]
                    set_visible: model.card_visible(),
                    #[name = "wifi_icon_container"]
                    gtk::Box {
                        add_css_class: "network-connection-icon",
                        #[watch]
                        set_css_classes: &model.wifi_icon_classes(),
                        set_hexpand: false,

                        #[name = "wifi_icon"]
                        gtk::Image {
                            #[watch]
                            set_icon_name: Some(model.effective_wifi_icon().as_str()),
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        },
                    },

                    #[name = "wifi_info"]
                    gtk::Box {
                        add_css_class: "network-connection-info",
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,

                        #[name = "wifi_name"]
                        gtk::Label {
                            add_css_class: "network-connection-name",
                            set_xalign: 0.0,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_max_width_chars: 1,
                            #[watch]
                            set_label: &model.display_wifi_name(),
                        },

                        #[name = "wifi_detail_box"]
                        gtk::Box {
                            #[watch]
                            set_visible: model.wifi_detail_visible(),
                            #[watch]
                            set_tooltip_text: model.error_tooltip().as_deref(),

                            #[name = "wifi_detail"]
                            gtk::Label {
                                #[watch]
                                set_css_classes: &model.wifi_detail_classes(),
                                set_hexpand: true,
                                set_xalign: 0.0,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                set_max_width_chars: 1,
                                #[watch]
                                set_label: &model.wifi_detail(),
                            },
                        },
                    },

                    #[name = "wifi_hover_stack"]
                    gtk::Stack {
                        add_css_class: "network-hover-stack",
                        set_transition_type: gtk::StackTransitionType::Crossfade,
                        set_transition_duration: 150,
                        set_valign: gtk::Align::Center,
                        set_hexpand: false,
                        add_named[Some("status")] = &gtk::Box {
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,

                            gtk::Label {
                                add_css_class: "network-connection-status",
                                #[watch]
                                set_label: &model.active_status_label(),
                                set_vexpand: false,
                                set_valign: gtk::Align::Center,
                                #[watch]
                                set_visible: model.is_active(),
                            },

                            #[template]
                            SubtleBadge {
                                #[watch]
                                set_css_classes: &model.status_classes(),
                                #[watch]
                                set_label: &model.status_label(),
                                set_vexpand: false,
                                set_valign: gtk::Align::Center,
                                #[watch]
                                set_visible: model.is_connecting() || model.has_wifi_error(),
                            },
                        },

                        add_named[Some("actions")] = &gtk::Box {
                            add_css_class: "network-connection-actions",
                            set_valign: gtk::Align::Center,

                            #[template]
                            GhostButton {
                                add_css_class: "network-action-disconnect",
                                #[template_child]
                                label {
                                    set_label: &t!("dropdown-iwd-disconnect"),
                                },
                                connect_clicked => ActiveConnectionsInput::Disconnect,
                            },

                            #[template]
                            GhostButton {
                                add_css_class: "network-action-forget",
                                #[watch]
                                set_visible: model.offers_forget(),
                                #[template_child]
                                label {
                                    set_label: &t!("dropdown-iwd-forget"),
                                },
                                connect_clicked => ActiveConnectionsInput::Forget,
                            },
                        },

                        add_named[Some("error-actions")] = &gtk::Box {
                            add_css_class: "network-connection-actions",
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,

                            #[template]
                            GhostButton {
                                add_css_class: "network-action-dismiss",
                                #[template_child]
                                label {
                                    set_label: &t!("dropdown-iwd-dismiss"),
                                },
                                connect_clicked => ActiveConnectionsInput::DismissError,
                            },
                        },

                        add_named[Some("cancel-actions")] = &gtk::Box {
                            add_css_class: "network-connection-actions",
                            set_halign: gtk::Align::End,
                            set_valign: gtk::Align::Center,

                            #[template]
                            GhostButton {
                                add_css_class: "network-action-disconnect",
                                #[template_child]
                                label {
                                    set_label: &t!("dropdown-iwd-cancel"),
                                },
                                // Cancelling an in-progress connection is just a
                                // disconnect; the service then reconciles us out
                                // of the Connecting state.
                                connect_clicked => ActiveConnectionsInput::Disconnect,
                            },

                            #[template]
                            GhostButton {
                                add_css_class: "network-action-forget",
                                #[watch]
                                set_visible: model.offers_forget(),
                                #[template_child]
                                label {
                                    set_label: &t!("dropdown-iwd-forget"),
                                },
                                connect_clicked => ActiveConnectionsInput::Forget,
                            },
                        },

                        #[watch]
                        set_visible_child_name:
                            if model.station.hovered && model.is_connecting() {
                                "cancel-actions"
                            } else if model.station.hovered && model.is_active() {
                                "actions"
                            } else if model.station.hovered && model.has_wifi_error() {
                                "error-actions"
                            } else {
                                "status"
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
        let station = init
            .iwd
            .station
            .get()
            .map(|station| StationState::from_station(&station))
            .unwrap_or_default();

        let mut model = Self {
            iwd: init.iwd.clone(),
            config: init.config.clone(),
            station,
            station_watcher: WatcherToken::new(),
        };

        watchers::spawn_device_watchers(&sender, &init.iwd);
        watchers::spawn_config_watchers(&sender, &init.config);

        model.reset_station_watchers(&sender);

        let widgets = view_output!();

        let hover = gtk::EventControllerMotion::new();

        let hover_sender = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            hover_sender.emit(ActiveConnectionsInput::CardHovered(true));
        });

        let leave_sender = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave_sender.emit(ActiveConnectionsInput::CardHovered(false));
        });

        widgets.wifi_card.add_controller(hover);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ActiveConnectionsInput::Disconnect => self.disconnect(),
            ActiveConnectionsInput::Forget => self.forget(),
            ActiveConnectionsInput::DismissError => self.dismiss_error(),
            ActiveConnectionsInput::CardHovered(hovered) => {
                self.station.hovered = hovered;
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: ActiveConnectionsCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            ActiveConnectionsCmd::StateChanged {
                connection,
                strength,
                frequency,
                last_error,
            } => {
                self.station.connection = connection;
                self.station.strength = strength;
                self.station.frequency = frequency;
                self.station.last_error = last_error;
            }
            ActiveConnectionsCmd::StationDeviceChanged => {
                if self.iwd.station.get().is_none() {
                    self.station = StationState::default();
                }

                self.reset_station_watchers(&sender);
            }
            // No state to update — returning re-evaluates the `#[watch]` bindings,
            // so `effective_wifi_icon` re-reads the configured icons.
            ActiveConnectionsCmd::ConfigChanged => {}
        }
    }
}
