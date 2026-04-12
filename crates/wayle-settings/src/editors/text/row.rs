use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use crate::{
    editors::{
        make_dirty_badge,
        text::{TextControl, TextInit, TextLike},
    },
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn text(property: &ConfigProperty<String>) -> SettingRowInit {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn text_like<T: TextLike>(property: &ConfigProperty<T>) -> SettingRowInit {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &T| value.to_entry_text()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
    }
}
