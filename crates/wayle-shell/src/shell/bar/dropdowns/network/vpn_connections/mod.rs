mod messages;
mod row;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use tracing::warn;
use wayle_network::NetworkService;
use wayle_widgets::prelude::*;

pub(crate) use self::messages::{VpnConnectionsInit, VpnConnectionsInput};
use self::{
    messages::{VpnConnectionsCmd, VpnRowState},
    row::{VpnRow, VpnRowInit, VpnRowInput, VpnRowOutput},
};
use crate::i18n::t;

pub(crate) struct VpnConnections {
    network: Arc<NetworkService>,
    rows: Vec<VpnRowState>,
    banner: Option<String>,
    items: FactoryVecDeque<VpnRow>,
}

#[relm4::component(pub(crate))]
impl Component for VpnConnections {
    type Init = VpnConnectionsInit;
    type Input = VpnConnectionsInput;
    type Output = ();
    type CommandOutput = VpnConnectionsCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            #[watch]
            set_visible: !model.rows.is_empty(),

            gtk::Label {
                add_css_class: "section-label",
                set_halign: gtk::Align::Start,
                set_label: &t!("dropdown-network-vpn"),
            },

            #[template]
            Card {
                add_css_class: "network-connections-group",
                set_orientation: gtk::Orientation::Vertical,

                #[local_ref]
                items_widget -> gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                },

                gtk::Label {
                    add_css_class: "network-connection-detail",
                    add_css_class: "vpn-banner",
                    set_halign: gtk::Align::Start,
                    set_xalign: 0.0,
                    set_wrap: true,
                    #[watch]
                    set_label: model.banner.as_deref().unwrap_or(""),
                    #[watch]
                    set_visible: model.banner.as_ref().is_some_and(|b| !b.is_empty()),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let items = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |output| match output {
                VpnRowOutput::Connect(path) => VpnConnectionsInput::Connect(path),
                VpnRowOutput::Disconnect(path) => VpnConnectionsInput::Disconnect(path),
            });

        let initial_rows = watchers::build_rows(
            &init.network.vpn.connections.get(),
            &init.network.vpn.active_connections.get(),
        );

        let mut model = Self {
            network: init.network.clone(),
            rows: Vec::new(),
            banner: init.network.vpn.banner.get(),
            items,
        };

        model.reconcile_rows(initial_rows);

        watchers::spawn(&sender, &init.network);

        let items_widget = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            VpnConnectionsInput::Connect(path) => {
                let vpn = self.network.vpn.clone();
                sender.command(move |_out, _shutdown| async move {
                    if let Err(err) = vpn.connect(path).await {
                        warn!(error = %err, "vpn connect failed");
                    }
                });
            }
            VpnConnectionsInput::Disconnect(path) => {
                let vpn = self.network.vpn.clone();
                sender.command(move |_out, _shutdown| async move {
                    if let Err(err) = vpn.disconnect(path).await {
                        warn!(error = %err, "vpn disconnect failed");
                    }
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
            VpnConnectionsCmd::RowsChanged(rows) => {
                self.reconcile_rows(rows);
            }
            VpnConnectionsCmd::BannerChanged(banner) => {
                self.banner = banner;
            }
        }
    }
}

impl VpnConnections {
    fn reconcile_rows(&mut self, rows: Vec<VpnRowState>) {
        let mut guard = self.items.guard();

        for (index, row) in rows.iter().enumerate() {
            match self.rows.get(index) {
                Some(existing) if existing.connection_path == row.connection_path => {
                    if existing.id != row.id {
                        guard.send(index, VpnRowInput::IdUpdated(row.id.clone()));
                    }
                    let active_changed = match (&existing.active, &row.active) {
                        (Some(a), Some(b)) => {
                            a.object_path != b.object_path || a.state != b.state
                        }
                        (None, None) => false,
                        _ => true,
                    };
                    if active_changed {
                        guard.send(index, VpnRowInput::ActiveUpdated(row.active.clone()));
                    }
                }
                _ => {
                    while guard.len() > index {
                        guard.pop_back();
                    }
                    guard.push_back(VpnRowInit {
                        connection_path: row.connection_path.clone(),
                        id: row.id.clone(),
                        active: row.active.clone(),
                    });
                }
            }
        }

        while guard.len() > rows.len() {
            guard.pop_back();
        }

        drop(guard);
        self.rows = rows;
    }
}
