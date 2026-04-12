//! Theme preset selector control factory.

use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::styling::{PaletteConfig, ThemeEntry},
};

use crate::{
    editors::theme_selector::{ThemeSelectorControl, ThemeSelectorInit},
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn theme_selector(
    available: &ConfigProperty<Vec<ThemeEntry>>,
    palette: &PaletteConfig,
    i18n_key: &'static str,
) -> SettingRowInit {
    let controller = ThemeSelectorControl::builder()
        .launch(ThemeSelectorInit {
            available: available.clone(),
            palette: palette.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: Some(i18n_key),
        handle: PropertyHandle::new(available, |themes| format!("{} themes", themes.len())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Action,
    }
}
