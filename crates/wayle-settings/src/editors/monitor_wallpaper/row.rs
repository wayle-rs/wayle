use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, schemas::wallpaper::MonitorWallpaperConfig};

use crate::{
    editors::monitor_wallpaper::MonitorWallpaperControl, pages::spec::SettingRowInit,
    property_handle::PropertyHandle, row::RowBehavior,
};

pub(crate) fn monitor_wallpaper(
    property: &ConfigProperty<Vec<MonitorWallpaperConfig>>,
) -> SettingRowInit {
    let controller = MonitorWallpaperControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |monitors| monitors.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
