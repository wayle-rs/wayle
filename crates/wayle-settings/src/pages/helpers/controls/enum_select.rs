use gtk4::prelude::*;
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use wayle_config::{ConfigProperty, EnumVariants};

use crate::{
    controls::enum_select::EnumSelectControl, pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle, row::RowBehavior,
};

pub(crate) fn enum_select<E>(property: &ConfigProperty<E>) -> SettingSpec
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

    SettingSpec {
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
