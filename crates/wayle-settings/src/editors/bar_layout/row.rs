use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarLayout, modules::CustomModuleDefinition},
};

use crate::{
    editors::bar_layout::{BarLayoutControl, BarLayoutInit},
    pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn bar_layout(
    property: &ConfigProperty<Vec<BarLayout>>,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
) -> SettingSpec {
    let controller = BarLayoutControl::builder()
        .launch(BarLayoutInit {
            property: property.clone(),
            custom_modules: custom_modules.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |layouts| layouts.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
