use std::sync::Arc;

use wayle_network::{NetworkService, types::states::NMActiveConnectionState};
use zbus::zvariant::OwnedObjectPath;

pub(crate) struct VpnConnectionsInit {
    pub network: Arc<NetworkService>,
}

#[derive(Debug, Clone)]
pub(crate) struct VpnActiveInfo {
    pub object_path: OwnedObjectPath,
    pub state: NMActiveConnectionState,
}

#[derive(Debug, Clone)]
pub(crate) struct VpnRowState {
    pub connection_path: OwnedObjectPath,
    pub id: String,
    pub active: Option<VpnActiveInfo>,
}

#[derive(Debug)]
pub(crate) enum VpnConnectionsInput {
    Connect(OwnedObjectPath),
    Disconnect(OwnedObjectPath),
}

#[derive(Debug)]
pub(crate) enum VpnConnectionsCmd {
    RowsChanged(Vec<VpnRowState>),
    BannerChanged(Option<String>),
}
