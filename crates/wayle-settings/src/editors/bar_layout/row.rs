use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarLayout, modules::CustomModuleDefinition},
};

use crate::{
    editors::bar_layout::{BarLayoutControl, BarLayoutInit},
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

/// Full-width bar layout editor with left, center, and right zones.
pub(crate) fn bar_layout(
    property: &ConfigProperty<Vec<BarLayout>>,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
) -> SettingRowInit {
    let controller = BarLayoutControl::builder()
        .launch(BarLayoutInit {
            property: property.clone(),
            custom_modules: custom_modules.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |layouts| layouts.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: None,
    }
}
