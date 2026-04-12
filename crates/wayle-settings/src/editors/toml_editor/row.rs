use relm4::{gtk::prelude::*, prelude::*};
use serde::{Deserialize, Serialize};
use wayle_config::{ConfigProperty, schemas::styling::HexColor};

use crate::{
    editors::{
        make_dirty_badge,
        toml_editor::{TomlEditorControl, TomlEditorInit, helpers::serialize_with_key},
    },
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

/// Full-width row with a TOML source editor for any serializable property, keyed under the given table name. Shows a dirty badge since edits apply on explicit save.
pub(crate) fn toml_editor<T>(
    property: &ConfigProperty<T>,
    key: &'static str,
    palette_bg: &ConfigProperty<HexColor>,
) -> SettingRowInit
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    toml_editor_sized(property, key, 0, palette_bg)
}

/// Full-width TOML editor row with a minimum visible line count, for properties whose default size would be too short to edit comfortably.
pub(crate) fn toml_editor_sized<T>(
    property: &ConfigProperty<T>,
    key: &'static str,
    min_lines: u32,
    palette_bg: &ConfigProperty<HexColor>,
) -> SettingRowInit
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    let badge = make_dirty_badge();
    let lines = if min_lines > 0 { Some(min_lines) } else { None };

    let controller = TomlEditorControl::builder()
        .launch(TomlEditorInit {
            property: property.clone(),
            key,
            dirty_badge: badge.clone(),
            min_lines: lines,
            palette_bg: palette_bg.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, move |value| serialize_with_key(value, key)),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
    }
}
