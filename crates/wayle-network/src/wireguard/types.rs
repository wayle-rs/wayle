use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use zbus::Connection;

use crate::core::settings::Settings;

#[doc(hidden)]
pub struct WireGuardParams<'a> {
    pub(crate) connection: &'a Connection,
    pub(crate) settings: Arc<Settings>,
}

#[doc(hidden)]
pub struct LiveWireGuardParams<'a> {
    pub(crate) connection: &'a Connection,
    pub(crate) cancellation_token: &'a CancellationToken,
    pub(crate) settings: Arc<Settings>,
}
