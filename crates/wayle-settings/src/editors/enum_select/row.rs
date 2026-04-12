use relm4::{gtk::prelude::*, prelude::*};
use serde::{Deserialize, Serialize};
use wayle_config::{ConfigProperty, EnumVariants};

use crate::{
    editors::enum_select::EnumSelectControl, pages::spec::SettingRowInit,
    property_handle::PropertyHandle, row::RowBehavior,
};

/// Row with a dropdown populated from the property type's `EnumVariants` impl.
pub(crate) fn enum_select<E>(property: &ConfigProperty<E>) -> SettingRowInit
where
    E: EnumVariants
        + Clone
        + Send
        + Sync
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + 'static,
{
    let controller = EnumSelectControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| {
            serde_json::to_string(value)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned()
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
