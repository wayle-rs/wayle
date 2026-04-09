//! TOML serialization helpers for key-wrapped round-tripping.

use serde::{Deserialize, Serialize};

pub(crate) fn serialize_with_key<T: Serialize>(value: &T, key: &str) -> String {
    let Ok(toml_value) = toml::Value::try_from(value) else {
        return String::new();
    };

    let mut table = toml::Table::new();
    table.insert(key.to_string(), toml_value);

    toml::to_string_pretty(&table).unwrap_or_default()
}

pub(super) fn deserialize_with_key<T: for<'de> Deserialize<'de>>(
    text: &str,
    key: &str,
) -> Option<T> {
    let table: toml::Table = toml::from_str(text).ok()?;
    let value = table.get(key)?;

    T::deserialize(value.clone()).ok()
}
