use wayle_config::schemas::modules::IwdConfig;
use wayle_iwd::IwdService;

use super::{
    IwdModule,
    helpers::{StationContext, wifi_icon, wifi_label},
};
use crate::i18n::t;

impl IwdModule {
    pub(super) fn compute_display(config: &IwdConfig, iwd: &IwdService) -> (String, String) {
        if let Some(station) = iwd.station.get() {
            let ctx = StationContext {
                powered: station.powered.get(),
                connection: station.connection.get(),
                strength: station.strength.get(),
            };
            (wifi_icon(config, &ctx), wifi_label(&ctx))
        } else {
            (config.wifi_offline_icon.get().clone(), t!("bar-iwd-no-wifi"))
        }
    }
}
