mod messages;
pub(super) mod providers;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_network::NetworkService;
use wayle_widgets::prelude::*;

pub(crate) use self::messages::{
    VpnProviderConfig, VpnTunnelsInit, VpnTunnelsInput, VpnTunnelsOutput,
};
use self::messages::{TunnelState, VpnTunnelsCmd};

pub(crate) struct VpnTunnels {
    network: Arc<NetworkService>,
    config: Arc<ConfigService>,
    provider: VpnProviderConfig,
    tunnels: Vec<TunnelState>,
    service_available: bool,
}

#[relm4::component(pub(crate))]
impl Component for VpnTunnels {
    type Init = VpnTunnelsInit;
    type Input = VpnTunnelsInput;
    type Output = VpnTunnelsOutput;
    type CommandOutput = VpnTunnelsCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[watch]
            set_visible: model.service_available,

            #[name = "section_label"]
            gtk::Label {
                add_css_class: "section-label",
                set_halign: gtk::Align::Start,
                #[watch]
                set_visible: !model.tunnels.is_empty(),
            },

            #[template]
            Card {
                add_css_class: "network-connections-group",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: !model.tunnels.is_empty(),

                #[name = "tunnels_box"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                },
            },

            #[name = "import_row"]
            gtk::Box {
                add_css_class: "network-vpn-import-row",
                set_halign: gtk::Align::Start,
                set_margin_top: 4,

                #[template]
                GhostButton {
                    add_css_class: "network-vpn-import",
                    #[template_child]
                    label {
                        set_label: &crate::i18n::t!("dropdown-network-wireguard-import"),
                    },
                    connect_clicked => VpnTunnelsInput::ImportConfig,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let service_available = providers::wireguard_available(&init.network);
        let mut tunnels = providers::wireguard_tunnels(&init.network);
        apply_aliases(&mut tunnels, &init.config);

        watchers::spawn(&sender, &init.network, &init.config);

        let model = Self {
            network: init.network.clone(),
            config: init.config,
            provider: init.provider,
            tunnels,
            service_available,
        };

        let widgets = view_output!();

        // Set provider-specific section label
        widgets.section_label.set_label(&model.provider.section_label);

        rebuild_tunnel_cards(
            &model.tunnels,
            &model.provider,
            &widgets.tunnels_box,
            &sender,
        );

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            VpnTunnelsInput::ToggleTunnel(index) => {
                self.toggle_tunnel(index, &sender);
            }
            VpnTunnelsInput::RenameTunnel(index, new_name) => {
                self.rename_tunnel(index, &new_name);
                // Rebuild cards to reflect the updated name
                if let Some(card) =
                    root.last_child().and_then(|c| c.prev_sibling())
                    && let Some(tunnels_box) = card.first_child()
                    && let Ok(bx) = tunnels_box.downcast::<gtk::Box>()
                {
                    rebuild_tunnel_cards(
                        &self.tunnels,
                        &self.provider,
                        &bx,
                        &sender,
                    );
                }
            }
            VpnTunnelsInput::ImportConfig => {
                self.open_import_dialog(&sender);
            }
            VpnTunnelsInput::FileSelected(name, content) => {
                self.import_file(&name, &content, &sender);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: VpnTunnelsCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            VpnTunnelsCmd::TunnelsChanged(mut tunnels) => {
                apply_aliases(&mut tunnels, &self.config);
                self.tunnels = tunnels;

                // Find the tunnels_box child widget to rebuild
                if let Some(card) = root.last_child().and_then(|c| c.prev_sibling())
                    && let Some(tunnels_box) = card.first_child()
                    && let Ok(bx) = tunnels_box.downcast::<gtk::Box>()
                {
                    rebuild_tunnel_cards(
                        &self.tunnels,
                        &self.provider,
                        &bx,
                        &sender,
                    );
                }
            }
            VpnTunnelsCmd::ServiceChanged => {
                self.service_available =
                    providers::wireguard_available(&self.network);
                watchers::spawn_tunnel_watchers(&sender, &self.network);
            }
            VpnTunnelsCmd::ToggleResult(index, result) => {
                if let Err(err) = result {
                    let name = self
                        .tunnels
                        .get(index)
                        .map(|t| t.name.clone())
                        .unwrap_or_default();
                    let _ = sender
                        .output(VpnTunnelsOutput::Error(format!("{name}: {err}")));
                }
            }
            VpnTunnelsCmd::ImportResult(result) => match result {
                Ok(name) => {
                    let _ = sender.output(VpnTunnelsOutput::Activated(name));
                }
                Err(err) => {
                    let _ = sender.output(VpnTunnelsOutput::Error(err));
                }
            },
        }
    }
}

impl VpnTunnels {
    fn toggle_tunnel(&self, index: usize, sender: &ComponentSender<Self>) {
        let Some(tunnel) = self.tunnels.get(index) else {
            return;
        };

        let network = self.network.clone();
        let connection_path = tunnel.connection_path.clone();
        let active = tunnel.active;

        // Currently dispatches to WireGuard. When adding new providers,
        // this should dispatch based on self.provider or connection type.
        sender.command(move |out, _shutdown| async move {
            let Some(wg) = network.wireguard.get() else {
                let _ = out.send(VpnTunnelsCmd::ToggleResult(
                    index,
                    Err(String::from("VPN service not available")),
                ));
                return;
            };

            let result = if active {
                let tunnel = wg
                    .tunnels
                    .get()
                    .into_iter()
                    .find(|t| t.profile.object_path == connection_path);

                if let Some(tunnel) = tunnel {
                    wg.deactivate(&tunnel).await
                } else {
                    Err(wayle_network::Error::OperationFailed {
                        operation: "find tunnel",
                        source: "tunnel not found".into(),
                    })
                }
            } else {
                wg.activate(&connection_path).await.map(|_| ())
            };

            let _ = out.send(VpnTunnelsCmd::ToggleResult(
                index,
                result.map_err(|e| e.to_string()),
            ));
        });
    }

    fn rename_tunnel(&mut self, index: usize, new_name: &str) {
        let Some(tunnel) = self.tunnels.get_mut(index) else {
            return;
        };

        let mut aliases = self
            .config
            .config()
            .modules
            .network
            .vpn_aliases
            .get();

        let trimmed = new_name.trim().to_owned();
        if trimmed.is_empty() {
            aliases.remove(&tunnel.uuid);
            // Re-derive original name from provider data
            let original = providers::wireguard_tunnels(&self.network);
            if let Some(orig) = original.iter().find(|t| t.uuid == tunnel.uuid) {
                tunnel.name = orig.name.clone();
            }
        } else {
            aliases.insert(tunnel.uuid.clone(), trimmed.clone());
            tunnel.name = trimmed;
        }

        self.config
            .config()
            .modules
            .network
            .vpn_aliases
            .set(aliases);
    }

    fn open_import_dialog(&self, sender: &ComponentSender<Self>) {
        let input_sender = sender.input_sender().clone();
        let import_filter = self.provider.import_filter.to_owned();
        let import_filter_label = self.provider.import_filter_label.to_owned();

        relm4::spawn_local(async move {
            let dialog = gtk::FileDialog::builder()
                .title("Import VPN Configuration")
                .build();

            let filter = gtk::FileFilter::new();
            filter.add_pattern(&import_filter);
            filter.set_name(Some(&import_filter_label));

            let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
            filters.append(&filter);
            dialog.set_filters(Some(&filters));

            let window: Option<&gtk::Window> = None;
            if let Ok(file) = dialog.open_future(window).await
                && let Some(path) = file.path()
            {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("vpn")
                    .to_owned();

                if let Ok(content) = std::fs::read_to_string(&path) {
                    input_sender
                        .emit(VpnTunnelsInput::FileSelected(name, content));
                }
            }
        });
    }

    fn import_file(
        &self,
        name: &str,
        content: &str,
        sender: &ComponentSender<Self>,
    ) {
        let network = self.network.clone();
        let name = name.to_owned();
        let content = content.to_owned();

        // Currently dispatches to WireGuard. When adding new providers,
        // this should dispatch based on self.provider or connection type.
        sender.command(move |out, _shutdown| async move {
            let Some(wg) = network.wireguard.get() else {
                let _ = out.send(VpnTunnelsCmd::ImportResult(Err(
                    String::from("VPN service not available"),
                )));
                return;
            };

            let result = wg
                .import(&name, &content)
                .await
                .map(|_| name.clone())
                .map_err(|e| e.to_string());

            let _ = out.send(VpnTunnelsCmd::ImportResult(result));
        });
    }
}

/// Applies user-configured display name aliases to tunnel states.
fn apply_aliases(tunnels: &mut [TunnelState], config: &ConfigService) {
    let aliases = config.config().modules.network.vpn_aliases.get();
    for tunnel in tunnels.iter_mut() {
        if let Some(alias) = aliases.get(&tunnel.uuid) {
            tunnel.name = alias.clone();
        }
    }
}

fn rebuild_tunnel_cards(
    tunnels: &[TunnelState],
    provider: &VpnProviderConfig,
    container: &gtk::Box,
    sender: &ComponentSender<VpnTunnels>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    for (index, tunnel) in tunnels.iter().enumerate() {
        let card = build_tunnel_card(index, tunnel, provider, sender);
        container.append(&card);
    }
}

fn build_tunnel_card(
    index: usize,
    tunnel: &TunnelState,
    provider: &VpnProviderConfig,
    sender: &ComponentSender<VpnTunnels>,
) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    card.add_css_class("network-connection-card");

    // Icon
    let icon_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    icon_box.add_css_class("network-connection-icon");
    icon_box.add_css_class("vpn");
    icon_box.set_hexpand(false);

    let icon = gtk::Image::from_icon_name(provider.icon);
    icon.set_halign(gtk::Align::Center);
    icon.set_valign(gtk::Align::Center);
    icon_box.append(&icon);
    card.append(&icon_box);

    // Info with inline-editable name
    let info_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    info_box.add_css_class("network-connection-info");
    info_box.set_hexpand(true);

    let name_stack = gtk::Stack::new();
    name_stack.set_transition_type(gtk::StackTransitionType::None);

    let name_label = gtk::Label::new(Some(&tunnel.name));
    name_label.add_css_class("network-connection-name");
    name_label.set_halign(gtk::Align::Start);
    name_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let name_entry = gtk::Entry::new();
    name_entry.set_text(&tunnel.name);
    name_entry.add_css_class("network-connection-name-edit");
    name_entry.set_halign(gtk::Align::Fill);
    name_entry.set_hexpand(true);

    name_stack.add_named(&name_label, Some("label"));
    name_stack.add_named(&name_entry, Some("entry"));
    name_stack.set_visible_child_name("label");

    // Double-click to enter edit mode
    let gesture = gtk::GestureClick::new();
    gesture.set_button(1);
    let stack_clone = name_stack.clone();
    let entry_clone = name_entry.clone();
    let label_clone = name_label.clone();
    gesture.connect_pressed(move |gesture, n_press, _, _| {
        if n_press == 2 {
            entry_clone.set_text(&label_clone.text());
            stack_clone.set_visible_child_name("entry");
            entry_clone.grab_focus();
            gesture.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    name_label.add_controller(gesture);

    // Enter commits the rename
    let stack_commit = name_stack.clone();
    let rename_sender = sender.input_sender().clone();
    name_entry.connect_activate(move |entry| {
        let new_name = entry.text().to_string();
        stack_commit.set_visible_child_name("label");
        rename_sender.emit(VpnTunnelsInput::RenameTunnel(index, new_name));
    });

    // Escape cancels editing
    let key_controller = gtk::EventControllerKey::new();
    let stack_cancel = name_stack.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            stack_cancel.set_visible_child_name("label");
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    name_entry.add_controller(key_controller);

    info_box.append(&name_stack);

    // Detail line (IP/interface when active)
    if tunnel.active {
        let detail_parts: Vec<String> = [
            tunnel.ip4_address.clone(),
            tunnel.interface_name.clone(),
        ]
        .into_iter()
        .flatten()
        .collect();

        if !detail_parts.is_empty() {
            let detail_label =
                gtk::Label::new(Some(&detail_parts.join(" - ")));
            detail_label.add_css_class("network-connection-detail");
            detail_label.set_halign(gtk::Align::Start);
            info_box.append(&detail_label);
        }
    }

    card.append(&info_box);

    // Toggle switch — disabled for externally managed tunnels (e.g. wg-quick)
    // to prevent NM from corrupting their config files.
    let switch = gtk::Switch::new();
    switch.set_active(tunnel.active);
    switch.set_valign(gtk::Align::Center);
    switch.add_css_class("network-vpn-toggle");
    switch.set_sensitive(!tunnel.externally_managed);

    if tunnel.externally_managed {
        switch.set_tooltip_text(Some("Managed externally (e.g. wg-quick)"));
    }

    let toggle_sender = sender.input_sender().clone();
    switch.connect_state_set(move |_s, _active| {
        toggle_sender.emit(VpnTunnelsInput::ToggleTunnel(index));
        gtk::glib::Propagation::Stop
    });

    card.append(&switch);

    card
}
