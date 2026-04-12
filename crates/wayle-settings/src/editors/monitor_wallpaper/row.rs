use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, schemas::wallpaper::MonitorWallpaperConfig};

use crate::{
    editors::monitor_wallpaper::MonitorWallpaperControl, pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle, row::RowBehavior,
};

pub(crate) fn monitor_wallpaper(
    property: &ConfigProperty<Vec<MonitorWallpaperConfig>>,
) -> SettingSpec {
    let controller = MonitorWallpaperControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |monitors| monitors.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
