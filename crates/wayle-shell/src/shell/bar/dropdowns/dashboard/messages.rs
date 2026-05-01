use std::sync::Arc;

use wayle_audio::AudioService;
use wayle_battery::BatteryService;
use wayle_bluetooth::BluetoothService;
use wayle_config::ConfigService;
use wayle_core::DeferredService;
use wayle_media::MediaService;
use wayle_network::NetworkService;
use wayle_notification::NotificationService;
use wayle_power_profiles::PowerProfilesService;
use wayle_sysinfo::SysinfoService;

use crate::services::IdleInhibitService;

pub(crate) struct DashboardDropdownInit {
    pub audio: Option<Arc<AudioService>>,
    pub battery: Option<Arc<BatteryService>>,
    pub bluetooth: DeferredService<BluetoothService>,
    pub config: Arc<ConfigService>,
    pub media: Option<Arc<MediaService>>,
    pub network: Option<Arc<NetworkService>>,
    pub notification: Option<Arc<NotificationService>>,
    pub power_profiles: DeferredService<PowerProfilesService>,
    pub sysinfo: Arc<SysinfoService>,
    pub idle_inhibit: Arc<IdleInhibitService>,
}

#[derive(Debug)]
pub(crate) enum DashboardDropdownMsg {
    VisibilityChanged(bool),
    OpenSettings,
}

#[derive(Debug)]
pub(crate) enum DashboardDropdownCmd {
    ScaleChanged(f32),
}
