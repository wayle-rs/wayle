//! Battery service bootstrap.

use crate::services::BatteryService;

pub(super) async fn build() -> Result<BatteryService, String> {
    BatteryService::new().await
}
