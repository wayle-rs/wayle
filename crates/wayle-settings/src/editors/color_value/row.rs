use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, schemas::styling::ColorValue};

use crate::{
    editors::color_value::ColorValueControl, pages::spec::SettingRowInit,
    property_handle::PropertyHandle, row::RowBehavior,
};

/// Row for a `ColorValue` property, letting the user pick between auto, transparent, a palette token, or a custom hex color.
pub(crate) fn color_value(property: &ConfigProperty<ColorValue>) -> SettingRowInit {
    let controller = ColorValueControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| match value {
            ColorValue::Auto => "auto".to_owned(),
            ColorValue::Transparent => "transparent".to_owned(),
            ColorValue::Custom(hex) => hex.to_string(),
            ColorValue::Token(token) => token
                .as_str()
                .strip_prefix("--")
                .unwrap_or(token.as_str())
                .to_owned(),
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
