//! Config layer derive macro implementations.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::{
    field_utils::{serde_key, should_skip},
    validate_named_struct,
};

/// Generates `ApplyConfigLayer`. When config.toml is loaded, the parsed
/// TOML table lands here. Each field gets its value by looking up the
/// serde-renamed key (e.g. `inset-edge` for `inset_edge`). Fields with
/// `#[wayle(skip)]` are ignored.
pub fn apply_config_layer(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_updates = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            let toml_key = serde_key(field);

            quote! {
                if let Some(field_value) = table.get(#toml_key) {
                    let child_path = if path.is_empty() {
                        String::from(#toml_key)
                    } else {
                        format!("{}.{}", path, #toml_key)
                    };
                    self.#field_ident.apply_config_layer(field_value, &child_path);
                }
            }
        });

    let generated = quote! {
        impl wayle_config::ApplyConfigLayer for #struct_name {
            fn apply_config_layer(&self, value: &toml::Value, path: &str) {
                if let toml::Value::Table(table) = value {
                    #(#field_updates)*
                }
            }
        }
    };

    TokenStream::from(generated)
}

/// Generates `ApplyRuntimeLayer`. Works like [`apply_config_layer`] but for
/// runtime.toml (GUI overrides). Returns `Err` if a field rejects the value,
/// so bad overrides get caught before they propagate.
pub fn apply_runtime_layer(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_updates = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            let toml_key = serde_key(field);

            quote! {
                if let Some(field_value) = table.get(#toml_key) {
                    let child_path = if path.is_empty() {
                        String::from(#toml_key)
                    } else {
                        format!("{}.{}", path, #toml_key)
                    };
                    self.#field_ident.apply_runtime_layer(field_value, &child_path)?;
                }
            }
        });

    let generated = quote! {
        impl wayle_config::ApplyRuntimeLayer for #struct_name {
            fn apply_runtime_layer(&self, value: &toml::Value, path: &str) -> Result<(), String> {
                if let toml::Value::Table(table) = value {
                    #(#field_updates)*
                }
                Ok(())
            }
        }
    };

    TokenStream::from(generated)
}

/// Generates `ExtractRuntimeValues` impl. Walks fields and collects any
/// non-None runtime values into a sparse TOML table for persistence.
pub fn extract_runtime_values(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_extractions = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            let toml_key = serde_key(field);

            quote! {
                if let Some(value) = self.#field_ident.extract_runtime_values() {
                    table.insert(String::from(#toml_key), value);
                }
            }
        });

    let generated = quote! {
        impl wayle_config::ExtractRuntimeValues for #struct_name {
            fn extract_runtime_values(&self) -> Option<toml::Value> {
                let mut table = toml::map::Map::new();
                #(#field_extractions)*

                if table.is_empty() {
                    None
                } else {
                    Some(toml::Value::Table(table))
                }
            }
        }
    };

    TokenStream::from(generated)
}

/// Generates `SubscribeChanges`. Clones the `mpsc::UnboundedSender<()>`
/// into each field. When any `ConfigProperty` changes its effective value,
/// a `()` fires on the channel, which is how `PersistenceWatcher` and
/// page-level watchers know something changed.
pub fn subscribe_changes(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_subscriptions = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            quote! { self.#field_ident.subscribe_changes(tx.clone()); }
        });

    let generated = quote! {
        impl wayle_config::SubscribeChanges for #struct_name {
            fn subscribe_changes(&self, tx: tokio::sync::mpsc::UnboundedSender<()>) {
                #(#field_subscriptions)*
            }
        }
    };

    TokenStream::from(generated)
}

/// Generates `ClearRuntimeByPath`. Given a dot-separated path like
/// `"bar.border-width"`, walks the struct tree to find the matching
/// `ConfigProperty` and clears its runtime override. Field lookup uses
/// serde-renamed keys, so the path matches TOML names, not Rust names.
///
/// Currently only used by the CLI (`wayle config reset <path>`).
pub fn clear_runtime_by_path(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let match_arms = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            let toml_key = serde_key(field);

            quote! {
                #toml_key => self.#field_ident.clear_runtime_by_path(rest),
            }
        });

    let generated = quote! {
        impl wayle_config::ClearRuntimeByPath for #struct_name {
            fn clear_runtime_by_path(&self, path: &str) -> Result<bool, String> {
                let (segment, rest) = match path.split_once('.') {
                    Some((seg, rest)) => (seg, rest),
                    None => (path, ""),
                };

                match segment {
                    #(#match_arms)*
                    "" => Err(String::from("empty path")),
                    other => Err(format!("unknown field '{other}'")),
                }
            }
        }
    };

    TokenStream::from(generated)
}

/// Generates a trait impl that calls the given method on every non-skipped
/// field in the struct. For example, passing `"ResetConfigLayer"` and
/// `"reset_config_layer"` emits `self.scale.reset_config_layer()`,
/// `self.padding.reset_config_layer()`, and so on...
///
/// Used by `ResetConfigLayer`, `ResetRuntimeLayer`, and `CommitConfigReload`
/// since they only differ in which method gets called.
pub fn simple_field_walk(input: TokenStream, trait_name: &str, method_name: &str) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let struct_name = &derive_input.ident;

    let struct_fields = match validate_named_struct(&derive_input) {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let trait_ident = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
    let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

    let field_calls = struct_fields
        .named
        .iter()
        .filter(|field| !should_skip(field))
        .map(|field| {
            let field_ident = &field.ident;
            quote! { self.#field_ident.#method_ident(); }
        });

    let generated = quote! {
        impl wayle_config::#trait_ident for #struct_name {
            fn #method_ident(&self) {
                #(#field_calls)*
            }
        }
    };

    TokenStream::from(generated)
}
