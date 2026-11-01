//! Enum variant metadata for settings GUI dropdowns.
//!
//! Config enums like `RoundingLevel` or `Location` need to list their
//! variants at runtime so the settings dropdown can populate itself.
//! Implement [`EnumVariants`] (or derive it) to make an enum work
//! as a dropdown.

/// A single variant's serde value paired with its fluent i18n key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    /// What goes into TOML, e.g. `"top"` or `"kebab-case-name"`.
    pub value: &'static str,

    /// Fluent message ID for the dropdown label, e.g. `"enum-location-top"`.
    pub fluent_key: &'static str,
}

/// Lists all variants of a config enum for the settings GUI dropdown.
///
/// ```ignore
/// #[derive(EnumVariants)]
/// #[serde(rename_all = "kebab-case")]
/// pub enum Location {
///     Top,
///     Bottom,
///     Left,
///     Right,
/// }
///
/// // Location::variants() returns:
/// // [
/// //     EnumVariant { value: "top", fluent_key: "enum-location-top" },
/// //     EnumVariant { value: "bottom", fluent_key: "enum-location-bottom" },
/// //     ...
/// // ]
/// ```
pub trait EnumVariants: Sized {
    /// All variants with their serde values and fluent keys.
    fn variants() -> &'static [EnumVariant];
}
