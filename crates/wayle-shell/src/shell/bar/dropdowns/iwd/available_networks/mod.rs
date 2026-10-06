mod messages;
mod methods;
mod network_item;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_iwd::{IwdService, SignalStrength};
use wayle_widgets::{WatcherToken, prelude::*};

pub(crate) use self::messages::{AvailableNetworksInit, AvailableNetworksInput};
use self::{messages::AvailableNetworksCmd, network_item::NetworkItem};
use crate::{
    i18n::t,
    shell::bar::dropdowns::iwd::password_form::{PasswordForm, PasswordFormInput},
};

pub(crate) struct AvailableNetworks {
    iwd: Arc<IwdService>,
    config: Arc<ConfigService>,
    network_list: FactoryVecDeque<NetworkItem>,
    /// Whether the popover is open; rows are only click-guarded while it is.
    visible: bool,
    /// Whether the active-connection card above the list was shown at the
    /// last rebuild: its appearing or going away moves every row.
    card_shown: bool,
    /// Shows IWD's passphrase request, while there is one.
    password_form: Controller<PasswordForm>,
    networks_watcher: WatcherToken,
}

#[relm4::component(pub(crate))]
impl Component for AvailableNetworks {
    type Init = AvailableNetworksInit;
    type Input = AvailableNetworksInput;
    type Output = ();
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
                // Not over the "no networks" empty state.
                #[watch]
                set_visible: model.wifi_available()
                    && model.powered()
                    && (!model.network_list.is_empty() || model.prompting()),
            },

            #[local_ref]
            password_form_widget -> gtk::Box {
                #[watch]
                set_visible: model.wifi_available()
                    && model.powered()
                    && model.prompting(),
            },

            #[name = "network_list_card"]
            #[template]
            Card {
                add_css_class: "network-list",
                set_overflow: gtk::Overflow::Hidden,
                set_vexpand: true,
                #[watch]
                set_visible: model.wifi_available() && model.powered() && !model.network_list.is_empty(),

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
                set_visible: model.wifi_available()
                    && model.powered()
                    && model.network_list.is_empty()
                    && !model.prompting(),

                #[template]
                EmptyState {
                    // As wide as the dropdown, so the description wraps
                    // within it: a label wider than the open dropdown would
                    // make GTK close it.
                    set_halign: gtk::Align::Fill,
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
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 1,
                        set_label: &t!("dropdown-iwd-no-networks-description"),
                    },
                },
            },

            #[name = "empty_powered_off"]
            gtk::Box {
                #[watch]
                set_visible: model.wifi_available() && !model.powered(),

                #[template]
                EmptyState {
                    // As wide as the dropdown, so the description wraps
                    // within it: a label wider than the open dropdown would
                    // make GTK close it.
                    set_halign: gtk::Align::Fill,
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        #[watch]
                        set_icon_name: Some(if model.hardware_blocked() {
                            "tb-wifi-off-symbolic".to_owned()
                        } else {
                            model.disabled_icon()
                        }.as_str()),
                    },
                    #[template_child]
                    title {
                        #[watch]
                        set_label: &if model.hardware_blocked() {
                            t!("dropdown-iwd-blocked-title")
                        } else {
                            t!("dropdown-iwd-disabled-title")
                        },
                    },
                    #[template_child]
                    description {
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 1,
                        #[watch]
                        set_label: &if model.hardware_blocked() {
                            t!("dropdown-iwd-blocked-description")
                        } else {
                            t!("dropdown-iwd-disabled-description")
                        },
                    },
                },
            },

            #[name = "empty_no_adapter"]
            gtk::Box {
                #[watch]
                set_visible: !model.wifi_available(),

                #[template]
                EmptyState {
                    // As wide as the dropdown, so the description wraps
                    // within it: a label wider than the open dropdown would
                    // make GTK close it.
                    set_halign: gtk::Align::Fill,
                    #[template_child]
                    icon {
                        add_css_class: "sm",
                        #[watch]
                        set_icon_name: Some("tb-wifi-off-symbolic"),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-iwd-no-adapter-title"),
                    },
                    #[template_child]
                    description {
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 1,
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

        let mut model = Self {
            iwd: init.iwd.clone(),
            config: init.config.clone(),
            network_list,
            visible: false,
            card_shown: false,
            password_form,
            networks_watcher: WatcherToken::new(),
        };

        // Its first value starts the station's watchers.
        watchers::spawn_station_watcher(&sender, &init.iwd);
        watchers::spawn_config_watchers(&sender, &init.config);
        watchers::spawn_passphrase_watcher(&sender, &init.iwd);

        model.rebuild_network_list();

        let password_form_widget = model.password_form.widget();
        let network_list_widget = model.network_list.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AvailableNetworksInput::VisibilityChanged(visible) => {
                self.handle_visibility_changed(visible);
            }
            AvailableNetworksInput::NetworkSelected(network) => Self::select_network(&network),
            AvailableNetworksInput::ForgetNetwork(network) => Self::forget_network(&network),
            AvailableNetworksInput::PasswordForm(form_output) => {
                self.handle_password_form(form_output);
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
            AvailableNetworksCmd::StationChanged => self.handle_station_changed(&sender),
            AvailableNetworksCmd::NetworksChanged => self.rebuild_network_list(),
            AvailableNetworksCmd::PassphraseRequestChanged => self.show_passphrase_request(),
            AvailableNetworksCmd::ConfigChanged => {
                // Rebuild the list so each item re-reads the configured icons, and
                // refresh the open password form's icon (without resetting entry).
                self.rebuild_network_list();

                if let Some(request) = self.iwd.passphrase_request.get() {
                    let strength = SignalStrength::from_dbm(request.network.signal.get());
                    self.password_form
                        .emit(PasswordFormInput::SetSignalIcon(self.signal_icon(strength)));
                }
            }
        }
    }
}
