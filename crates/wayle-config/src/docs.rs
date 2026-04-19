//! How a config schema opts in to the docs generator.
//!
//! A schema that should get its own reference page in the VitePress site
//! implements [`ModuleInfoProvider`] and calls [`register_module!`]. The
//! generator in `wayle/src/docs` picks it up through the `inventory` crate
//! and writes one markdown page from it.
//!
//! ```text
//! impl ModuleInfoProvider for BatteryConfig {
//!     fn module_info() -> ModuleInfo {
//!         ModuleInfo {
//!             name: String::from("battery"),
//!             schema: || schema_for!(BatteryConfig),
//!             layout_id: Some(String::from("battery")),
//!             array_entry: false,
//!         }
//!     }
//!
//!     fn groups() -> Vec<ConfigGroup> {
//!         GroupDefaults::bar_button()
//!     }
//! }
//!
//! register_module!(BatteryConfig);
//! ```
//!
//! Each page is a stack of H2 sections; [`ConfigGroup`] defines one section
//! and [`GroupRule`] decides which fields it claims. [`GroupDefaults`] bundles
//! presets for the common module shapes so most schemas don't have to build
//! their own group list.

use schemars::Schema;

/// Produces the JSON schema for a config type.
///
/// Typically wraps `schemars::schema_for!(MyConfig)` in a zero-capture closure
/// so the generator can lazily materialise each schema.
pub type SchemaFn = fn() -> Schema;

/// One reference page's worth of metadata.
pub struct ModuleInfo {
    /// Kebab-case slug used as the generated filename (`<name>.md`).
    pub name: String,

    /// Entry point the generator uses to read the schema tree.
    pub schema: SchemaFn,

    /// Identifier the user types inside a `[[bar.layout]]` array when the
    /// schema is a bar module. `None` for top-level sections like `bar` or
    /// `styling`.
    pub layout_id: Option<String>,

    /// `true` when the schema represents one entry of an array-of-tables
    /// (e.g. `[[modules.custom]]`). The default TOML block is rendered with
    /// `[[section]]` syntax instead of `[section]`.
    pub array_entry: bool,
}

/// One H2 section on a generated reference page: a title plus a [`GroupRule`]
/// that claims fields from the schema.
pub struct ConfigGroup {
    /// H2 heading for the group.
    pub title: &'static str,

    /// How the group claims fields from the schema.
    pub rule: GroupRule,
}

/// How a [`ConfigGroup`] selects its fields.
pub enum GroupRule {
    /// Collects every field no other group claimed. Place this group last;
    /// only one catch-all per page.
    CatchAll,

    /// Matches by the field's schema type. Used to pull every color-valued
    /// field or every click-action field into its own section.
    ByType(TypeTag),

    /// Matches kebab-case field names starting with this prefix.
    Prefix(&'static str),

    /// Matches one field by exact name. Renders as a row like any other
    /// match; the separate section exists so the field sits under its own
    /// heading rather than under whatever catch-all would have grabbed it.
    Standalone(&'static str),
}

/// Schema types the generator recognises by name.
///
/// Used as the [`ByType`](GroupRule::ByType) discriminator so groups can claim
/// every field of a particular type.
pub enum TypeTag {
    /// [`crate::schemas::styling::ColorValue`].
    ColorValue,

    /// [`crate::ClickAction`].
    ClickAction,
}

impl TypeTag {
    /// The schema type name this tag corresponds to (matches schemars' `$ref`
    /// output and the schema's `$defs` keys).
    pub const fn schema_name(&self) -> &'static str {
        match self {
            Self::ColorValue => "ColorValue",
            Self::ClickAction => "ClickAction",
        }
    }
}

impl ConfigGroup {
    /// Catch-all "General" group. Collects every field no other group claimed.
    pub const fn general() -> Self {
        Self {
            title: "General",
            rule: GroupRule::CatchAll,
        }
    }

    /// "Colors" group: every field typed as `ColorValue`.
    pub const fn colors() -> Self {
        Self {
            title: "Colors",
            rule: GroupRule::ByType(TypeTag::ColorValue),
        }
    }

    /// "Click actions" group: every field typed as `ClickAction`.
    pub const fn click() -> Self {
        Self {
            title: "Click actions",
            rule: GroupRule::ByType(TypeTag::ClickAction),
        }
    }

    /// Group containing every field whose kebab-case name starts with `prefix`.
    pub const fn prefix(title: &'static str, prefix: &'static str) -> Self {
        Self {
            title,
            rule: GroupRule::Prefix(prefix),
        }
    }

    /// Group containing one field by exact name.
    pub const fn standalone(title: &'static str, field: &'static str) -> Self {
        Self {
            title,
            rule: GroupRule::Standalone(field),
        }
    }
}

/// Preset group lists for common module shapes.
///
/// These mirror the group layouts most schemas want. A schema with unusual
/// sections can build its own `Vec<ConfigGroup>` instead.
pub struct GroupDefaults;

impl GroupDefaults {
    /// Single "General" catch-all group. Matches the default
    /// [`ModuleInfoProvider::groups`] return value, so schemas rarely need to
    /// call this explicitly.
    pub fn standard() -> Vec<ConfigGroup> {
        vec![ConfigGroup::general()]
    }

    /// Four groups for bar-button modules: General, Colors, Click actions,
    /// Dropdown (prefix `dropdown-`).
    pub fn bar_button() -> Vec<ConfigGroup> {
        vec![
            ConfigGroup::general(),
            ConfigGroup::colors(),
            ConfigGroup::click(),
            ConfigGroup::prefix("Dropdown", "dropdown-"),
        ]
    }
}

/// Exposes a config struct to the docs generator.
///
/// Implement on every schema that should get a reference page, then pair the
/// impl with a [`register_module!`] call so the generator picks it up.
pub trait ModuleInfoProvider {
    /// Metadata for the generated page.
    fn module_info() -> ModuleInfo;

    /// Groups to render on the page, in order. Defaults to a single
    /// catch-all "General" group.
    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

/// A registry entry built by [`register_module!`] and collected at runtime.
///
/// Not constructed directly; the macro handles it.
pub struct ModuleRegistration {
    /// Produces the module's info and group layout on demand. Indirected
    /// through a function so the schema is only materialised when the
    /// generator asks for it.
    pub build_entry: fn() -> (ModuleInfo, Vec<ConfigGroup>),
}

inventory::collect!(ModuleRegistration);

#[doc(hidden)]
pub use inventory;

/// Registers a [`ModuleInfoProvider`] type with the docs generator.
///
/// Call once per schema, at the bottom of the module that defines it. The
/// generator picks up every registered type through `inventory`'s linker
/// collection, so there is no central list to maintain.
#[macro_export]
macro_rules! register_module {
    ($ty:ty) => {
        $crate::docs::inventory::submit! {
            $crate::docs::ModuleRegistration {
                build_entry: || {
                    (
                        <$ty as $crate::docs::ModuleInfoProvider>::module_info(),
                        <$ty as $crate::docs::ModuleInfoProvider>::groups(),
                    )
                },
            }
        }
    };
}
