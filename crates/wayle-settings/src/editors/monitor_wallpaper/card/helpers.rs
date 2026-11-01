//! Pure helpers for `FitMode` <-> dropdown-index translation.

use serde::{
    Deserialize,
    de::value::{Error as SerdeValueError, StrDeserializer},
};
use wayle_config::{EnumVariants, schemas::wallpaper::FitMode};
use wayle_i18n::t;

pub(super) fn fit_mode_labels() -> Vec<String> {
    FitMode::variants()
        .iter()
        .map(|variant| {
            let resolved = t(variant.fluent_key);

            if resolved == variant.fluent_key {
                variant.value.to_owned()
            } else {
                resolved
            }
        })
        .collect()
}

pub(super) fn fit_mode_index(mode: &FitMode) -> Option<u32> {
    FitMode::variants()
        .iter()
        .position(|variant| fit_mode_from_value(variant.value).as_ref() == Some(mode))
        .map(|index| index as u32)
}

pub(super) fn fit_mode_from_index(index: u32) -> Option<FitMode> {
    let variant = FitMode::variants().get(index as usize)?;
    fit_mode_from_value(variant.value)
}

fn fit_mode_from_value(value: &str) -> Option<FitMode> {
    let deserializer: StrDeserializer<'_, SerdeValueError> = StrDeserializer::new(value);
    FitMode::deserialize(deserializer).ok()
}
