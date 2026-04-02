//! Derive macros for Wayle configuration management.

mod derives;
mod field_utils;
mod wayle_config;

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
/// ```ignore
/// #[wayle_config]
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
/// The macro strips `#[default(...)]` and expands the struct with all the
/// config layer derives (`ApplyConfigLayer`, `ResetConfigLayer`, etc.) plus
/// a `Default` impl that calls `ConfigProperty::new(...)` with each
/// default expression:
///
/// ```ignore
/// impl Default for GeneralConfig {
///     fn default() -> Self {
///         Self {
///             font_sans: ConfigProperty::new(String::from("Inter")),
///             tearing_mode: ConfigProperty::new(false),
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn wayle_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    wayle_config::wayle_config(attr, item)
}

/// Loads config.toml values into each field's config layer by matching
/// serde-renamed keys in the incoming TOML table.
///
///
/// For a struct with #[serde(rename = "font-sans")] on font_sans,
/// the generated impl does:
/// ```ignore
/// if let Some(val) = table.get("font-sans") {
///     self.font_sans.apply_config_layer(val, "general.font-sans");
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

/// Serializes all runtime overrides into a sparse TOML table for writing
/// back to runtime.toml. Fields without overrides are omitted.
///
/// ```ignore
/// if let Some(value) = self.font_sans.extract_runtime_values() {
///     table.insert("font-sans", value);
/// }
/// ```
#[proc_macro_derive(ExtractRuntimeValues, attributes(wayle))]
pub fn derive_extract_runtime_values(input: TokenStream) -> TokenStream {
    derives::extract_runtime_values(input)
}

/// Part of the hot-reload cycle: quietly clears config layer values so
/// fresh ones can be re-applied without triggering watchers mid-reload.
///
/// ```ignore
/// self.font_sans.reset_config_layer();
/// self.tearing_mode.reset_config_layer();
/// ```
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

/// Final step of the hot-reload cycle: recomputes effective values from
/// both layers and fires watcher notifications for anything that changed.
///
/// ```ignore
/// self.font_sans.commit_config_reload();
/// self.tearing_mode.commit_config_reload();
/// ```
#[proc_macro_derive(CommitConfigReload, attributes(wayle))]
pub fn derive_commit_config_reload(input: TokenStream) -> TokenStream {
    derives::simple_field_walk(input, "CommitConfigReload", "commit_config_reload")
}

/// Wires up an `mpsc::UnboundedSender<()>` to every field so any property
/// change sends a signal. Used by `PersistenceWatcher` to know when to
/// save, and by page-level watchers to refresh the settings UI.
///
/// ```ignore
/// self.font_sans.subscribe_changes(tx.clone());
/// self.tearing_mode.subscribe_changes(tx.clone());
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
