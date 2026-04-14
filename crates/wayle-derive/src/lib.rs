//! Derive macros for Wayle configuration management.

mod derives;
mod enum_variants;
mod field_utils;
mod wayle_config;
mod wayle_enum;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Fields, FieldsNamed};

fn validate_named_struct(input: &DeriveInput) -> Result<&FieldsNamed, TokenStream> {
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => Ok(fields),
            _ => Err(syn::Error::new_spanned(
                input,
                "Can only be derived for structs with named fields",
            )
            .to_compile_error()
            .into()),
        },

        _ => Err(
            syn::Error::new_spanned(input, "Can only be derived for structs")
                .to_compile_error()
                .into(),
        ),
    }
}

/// Derives all config layer traits, adds `#[serde(default)]`, and generates
/// `Default` from `#[default(...)]` field annotations. Pass `bar_button` or
/// `bar_container` to enforce that the required styling fields are present.
///
/// When `i18n_prefix` is set, each `ConfigProperty` field automatically
/// gets a fluent key built from the prefix + its serde key. Fields marked
/// `#[i18n(skip)]` are excluded.
///
/// ```ignore
/// #[wayle_config(i18n_prefix = "settings-general")]
/// pub struct GeneralConfig {
///     #[serde(rename = "font-sans")]
///     #[default(String::from("Inter"))]
///     pub font_sans: ConfigProperty<String>,
///
///     #[serde(rename = "tearing-mode")]
///     #[default(false)]
///     pub tearing_mode: ConfigProperty<bool>,
/// }
/// ```
///
/// The macro strips `#[default(...)]` and generates a `Default` impl.
/// With `i18n_prefix`, each field uses `with_i18n_key` so the settings
/// GUI can look up the field's label and description:
///
/// ```ignore
/// impl Default for GeneralConfig {
///     fn default() -> Self {
///         Self {
///             font_sans: ConfigProperty::with_i18n_key(
///                 String::from("Inter"),
///                 "settings-general-font-sans",
///             ),
///             tearing_mode: ConfigProperty::with_i18n_key(
///                 false,
///                 "settings-general-tearing-mode",
///             ),
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn wayle_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    wayle_config::wayle_config(attr, item)
}

/// Injects standard derives and `#[serde(rename_all = "kebab-case")]` for
/// config enums. Pass `default` to also derive `Default`.
///
/// ```ignore
/// #[wayle_enum]
/// pub enum BarLocation {
///     Top,
///     Bottom,
/// }
///
/// #[wayle_enum(default)]
/// pub enum Shadow {
///     #[default]
///     None,
///     Subtle,
///     Strong,
/// }
/// ```
#[proc_macro_attribute]
pub fn wayle_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    wayle_enum::wayle_enum(attr, item)
}

/// Lists all variants of a config enum for the settings GUI dropdown.
/// Reads `#[serde(rename_all)]` to produce correct TOML values, and
/// generates fluent keys from the enum + variant name (e.g.
/// `Location::TopLeft` becomes `"enum-location-top-left"`).
///
/// Only works on enums with unit variants. Enums with data (like
/// `ColorValue` or `ClickAction`) need custom widgets instead.
///
/// ```ignore
/// #[derive(EnumVariants)]
/// #[serde(rename_all = "kebab-case")]
/// pub enum Location { Top, Bottom, Left, Right }
///
/// // Location::variants() returns:
/// // [
/// //   EnumVariant { value: "top", fluent_key: "enum-location-top" },
/// //   EnumVariant { value: "bottom", fluent_key: "enum-location-bottom" },
/// //   ...
/// // ]
/// ```
#[proc_macro_derive(EnumVariants, attributes(wayle))]
pub fn derive_enum_variants(input: TokenStream) -> TokenStream {
    enum_variants::derive(input)
}

/// Derives [`ApplyConfigLayer`], loading config.toml values into each field's
/// config layer by matching serde-renamed TOML keys.
///
/// ```ignore
/// #[derive(ApplyConfigLayer)]
/// pub struct ClockConfig {
///     #[serde(rename = "format")]
///     pub format: ConfigProperty<String>,
/// }
/// ```
#[proc_macro_derive(ApplyConfigLayer, attributes(wayle))]
pub fn derive_apply_config_layer(input: TokenStream) -> TokenStream {
    derives::apply_config_layer(input)
}

/// Loads runtime.toml overrides into each field's runtime layer. Also used
/// by the CLI's `wayle config set` to apply individual field changes.
///
/// Same shape as `ApplyConfigLayer`, but propagates errors so invalid
/// runtime overrides get caught before they take effect.
#[proc_macro_derive(ApplyRuntimeLayer, attributes(wayle))]
pub fn derive_apply_runtime_layer(input: TokenStream) -> TokenStream {
    derives::apply_runtime_layer(input)
}

/// Derives [`ExtractRuntimeValues`], serializing all runtime overrides into a
/// sparse TOML table for writing back to runtime.toml. Fields without
/// overrides are omitted.
///
/// ```ignore
/// #[derive(ExtractRuntimeValues)]
/// pub struct ClockConfig {
///     pub format: ConfigProperty<String>,
/// }
/// ```
#[proc_macro_derive(ExtractRuntimeValues, attributes(wayle))]
pub fn derive_extract_runtime_values(input: TokenStream) -> TokenStream {
    derives::extract_runtime_values(input)
}

/// Derives [`ResetConfigLayer`], clearing each field's config layer without
/// firing watchers. Used mid hot-reload so stale values are gone before
/// fresh ones get applied.
#[proc_macro_derive(ResetConfigLayer, attributes(wayle))]
pub fn derive_reset_config_layer(input: TokenStream) -> TokenStream {
    derives::simple_field_walk(input, "ResetConfigLayer", "reset_config_layer")
}

/// Part of the hot-reload cycle: quietly clears runtime layer values so
/// fresh ones can be re-applied without triggering watchers mid-reload.
///
/// Same shape as `ResetConfigLayer` but targets the runtime layer.
#[proc_macro_derive(ResetRuntimeLayer, attributes(wayle))]
pub fn derive_reset_runtime_layer(input: TokenStream) -> TokenStream {
    derives::simple_field_walk(input, "ResetRuntimeLayer", "reset_runtime_layer")
}

/// Derives [`CommitConfigReload`], the final step of the hot-reload cycle:
/// recomputes effective values from both layers and fires watcher
/// notifications for anything that changed.
#[proc_macro_derive(CommitConfigReload, attributes(wayle))]
pub fn derive_commit_config_reload(input: TokenStream) -> TokenStream {
    derives::simple_field_walk(input, "CommitConfigReload", "commit_config_reload")
}

/// Derives [`ClearAllRuntime`], clearing the runtime override on every field
/// and forcing a watcher notification even when the effective value is
/// unchanged. Needed for "reset all" actions where subscribers must see the
/// clear regardless of value equality.
#[proc_macro_derive(ClearAllRuntime, attributes(wayle))]
pub fn derive_clear_all_runtime(input: TokenStream) -> TokenStream {
    derives::simple_field_walk(input, "ClearAllRuntime", "clear_all_runtime")
}

/// Derives [`SubscribeChanges`], forwarding a `()` on `tx` whenever any
/// field's effective config value changes. Resetting a runtime override also
/// fires, even when the resulting value is unchanged.
///
/// ```ignore
/// let (tx, mut rx) = mpsc::unbounded_channel();
/// config.subscribe_changes(tx);
/// while rx.recv().await.is_some() {
///     // a field changed
/// }
/// ```
#[proc_macro_derive(SubscribeChanges, attributes(wayle))]
pub fn derive_subscribe_changes(input: TokenStream) -> TokenStream {
    derives::subscribe_changes(input)
}

/// Walks a dot-separated path like `"bar.border-width"` through the struct
/// tree and clears the runtime override on the matching field. Used by the
/// CLI's `wayle config reset` command.
///
/// ```ignore
/// match segment {
///     "font-sans" => self.font_sans.clear_runtime_by_path(rest),
///     "tearing-mode" => self.tearing_mode.clear_runtime_by_path(rest),
///     other => Err(format!("unknown field '{other}'")),
/// }
/// ```
#[proc_macro_derive(ClearRuntimeByPath, attributes(wayle))]
pub fn derive_clear_runtime_by_path(input: TokenStream) -> TokenStream {
    derives::clear_runtime_by_path(input)
}
