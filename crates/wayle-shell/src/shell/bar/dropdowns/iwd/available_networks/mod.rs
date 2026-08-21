mod messages;
mod methods;
mod network_item;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_iwd::IwdService;
use wayle_widgets::{WatcherToken, prelude::*};

pub(crate) use self::messages::{
    AvailableNetworksInit, AvailableNetworksInput, AvailableNetworksOutput,
};
use self::{
    messages::{AvailableNetworksCmd, SelectedNetwork},
    network_item::NetworkItem,
};
use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::password_form::{PasswordForm, PasswordFormInput},
};

pub(crate) struct AvailableNetworks {
    iwd: Arc<IwdService>,
    config: Arc<ConfigService>,
    wifi_available: bool,
    powered: bool,
    network_list: FactoryVecDeque<NetworkItem>,
    state: ListState,
    selection: Option<SelectedNetwork>,
    password_form: Controller<PasswordForm>,
    networks_watcher: WatcherToken,
}

#[derive(PartialEq)]
pub(super) enum ListState {
    Normal,
    PasswordEntry,
}

#[relm4::component(pub(crate))]
impl Component for AvailableNetworks {
    type Init = AvailableNetworksInit;
    type Input = AvailableNetworksInput;
    type Output = AvailableNetworksOutput;
    type CommandOutput = AvailableNetworksCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[name = "section_label"]
            gtk::Label {
                add_css_class: "section-label",
                set_halign: gtk::Align::Start,
                set_label: &t!("dropdown-iwd-available"),
                #[watch]
                set_visible: model.wifi_available && model.powered,
            },

            #[local_ref]
            password_form_widget -> gtk::Box {
                #[watch]
                set_visible: model.wifi_available
                    && model.powered
                    && model.state == ListState::PasswordEntry,
            },

            #[name = "network_list_card"]
            #[template]
            Card {
                add_css_class: "network-list",
                set_overflow: gtk::Overflow::Hidden,
                set_vexpand: true,
                #[watch]
                set_visible: model.wifi_available && model.powered && !model.network_list.is_empty(),

                #[name = "network_list_scroll"]
                gtk::ScrolledWindow {
                    add_css_class: "network-list-scroll",
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[local_ref]
                    network_list_widget -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                    },
                },
            },

            #[name = "empty_no_networks"]
            gtk::Box {
                // Hidden during password entry: the list is empty only because
                // the prompted network is shown in the form above, not because
                // no networks exist.
                #[watch]
                set_visible: model.wifi_available
                    && model.powered
                    && model.network_list.is_empty()
                    && model.state != ListState::PasswordEntry,

                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        #[watch]
                        set_icon_name: Some(model.disabled_icon().as_str()),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-iwd-no-networks-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-iwd-no-networks-description"),
                    },
                },
            },

            #[name = "empty_powered_off"]
            gtk::Box {
                #[watch]
                set_visible: model.wifi_available && !model.powered,

                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        #[watch]
                        set_icon_name: Some(model.disabled_icon().as_str()),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-iwd-disabled-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-iwd-disabled-description"),
                    },
                },
            },

            #[name = "empty_no_adapter"]
            gtk::Box {
                #[watch]
                set_visible: !model.wifi_available,

                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        set_icon_name: Some("tb-wifi-off-symbolic"),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-iwd-no-adapter-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-iwd-no-adapter-description"),
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
        let password_form = PasswordForm::builder()
            .launch(())
            .forward(sender.input_sender(), |form_output| {
                AvailableNetworksInput::PasswordForm(form_output)
            });

        let network_list = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), methods::forward_network_item_output);

        let station = init.iwd.station.get();
        let wifi_available = station.is_some();
        let powered = station.as_ref().is_some_and(|station| station.powered.get());

        let mut model = Self {
            iwd: init.iwd.clone(),
            config: init.config.clone(),
            wifi_available,
            powered,
            network_list,
            state: ListState::Normal,
            selection: None,
            password_form,
            networks_watcher: WatcherToken::new(),
        };

        if let Some(station) = station {
            let token = model.networks_watcher.reset();
            watchers::spawn(&sender, &station, token);
        }

        watchers::spawn_config_watchers(&sender, &init.config);

        model.rebuild_network_list();

        let password_form_widget = model.password_form.widget();
        let network_list_widget = model.network_list.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AvailableNetworksInput::StationAvailabilityChanged(available) => {
                self.handle_wifi_availability(available, &sender);
            }
            AvailableNetworksInput::PoweredChanged(enabled) => {
                self.handle_wifi_enabled(enabled);
            }
            AvailableNetworksInput::NetworkSelected(index) => {
                self.select_network(index, &sender);
            }
            AvailableNetworksInput::ForgetNetwork(path) => {
                self.forget_network(path, &sender);
            }
            AvailableNetworksInput::PasswordForm(form_output) => {
                self.handle_password_form(form_output, &sender);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: AvailableNetworksCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AvailableNetworksCmd::NetworksChanged => {
                // Dismiss first so the rebuild reflects the post-dismiss state
                // (e.g. another client connected to the prompted network).
                self.dismiss_stale_password_entry();
                self.rebuild_network_list();
            }
            AvailableNetworksCmd::ConnectionSettled => {
                self.state = ListState::Normal;
                self.clear_selection();
                self.rebuild_network_list();
            }
            AvailableNetworksCmd::ConnectionAuthFailed => {
                // Only re-prompt if the attempt is still current. If a backend
                // event (WiFi toggled off, device removed) tore down the
                // selection while the connect was in flight, don't orphan the
                // list in PasswordEntry with no form — return to normal browsing.
                self.state = if self.selection.is_some() {
                    ListState::PasswordEntry
                } else {
                    ListState::Normal
                };
                self.rebuild_network_list();

                if let Some(selection) = &self.selection {
                    self.password_form.emit(PasswordFormInput::Show {
                        ssid: selection.ssid.clone(),
                        security_label: selection.security_label.clone(),
                        signal_icon: self.signal_icon(selection.strength),
                        error_message: Some(t!("dropdown-iwd-error-wrong-password")),
                    });
                }
            }
            AvailableNetworksCmd::ConnectionFailed(reason) => {
                self.handle_connection_failure(reason, &sender);
            }
            AvailableNetworksCmd::ConfigChanged => {
                // Rebuild the list so each item re-reads the configured icons, and
                // refresh the open password form's icon (without resetting entry).
                self.rebuild_network_list();

                if self.state == ListState::PasswordEntry
                    && let Some(strength) = self.selection.as_ref().map(|selection| selection.strength)
                {
                    self.password_form
                        .emit(PasswordFormInput::SetSignalIcon(self.signal_icon(strength)));
                }
            }
        }
    }
}
