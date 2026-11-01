//! `#[derive(EnumVariants)]` implementation.
//!
//! Generates `EnumVariants` for simple config enums so they can be
//! used as settings GUI dropdowns. Only works on enums with unit
//! variants (no data). Reads `#[serde(rename_all)]` to produce the
//! correct serialized values.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);

    match generate(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let enum_name = &input.ident;

    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(syn::Error::new_spanned(
                input,
                "EnumVariants can only be derived for enums",
            ));
        }
    };

    let rename_all = parse_rename_all(&input.attrs);

    let mut variant_entries = Vec::new();

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new_spanned(
                variant,
                "EnumVariants only supports unit variants (no data). \
                 Use a custom widget for enums like ColorValue or ClickAction.",
            ));
        }

        let serde_value = variant_serde_name(&variant.ident, &variant.attrs, &rename_all);
        let fluent_key = variant_fluent_key(enum_name, &variant.ident, &variant.attrs);

        variant_entries.push(quote! {
            wayle_config::EnumVariant {
                value: #serde_value,
                fluent_key: #fluent_key,
            }
        });
    }

    Ok(quote! {
        impl wayle_config::EnumVariants for #enum_name {
            fn variants() -> &'static [wayle_config::EnumVariant] {
                &[#(#variant_entries),*]
            }
        }
    })
}

/// Reads `#[serde(rename_all = "...")]` from the enum's attributes.
fn parse_rename_all(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut rename_all = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rename_all = Some(value.value());
            }
            Ok(())
        });

        if rename_all.is_some() {
            return rename_all;
        }
    }

    None
}

/// Gets the serde-serialized name for a variant. Checks for per-variant
/// `#[serde(rename = "...")]` first, then applies `rename_all` from the enum.
fn variant_serde_name(
    variant_ident: &Ident,
    attrs: &[syn::Attribute],
    rename_all: &Option<String>,
) -> String {
    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        let mut rename = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let value: syn::LitStr = meta.value()?.parse()?;
                rename = Some(value.value());
            }
            Ok(())
        });

        if let Some(name) = rename {
            return name;
        }
    }

    let ident_str = variant_ident.to_string();

    match rename_all.as_deref() {
        Some("lowercase") => ident_str.to_lowercase(),
        Some("kebab-case") => pascal_to_kebab(&ident_str),
        _ => ident_str,
    }
}

/// Generates the fluent key for a variant. Checks for `#[wayle(fluent_key = "...")]`
/// override first, then auto-generates from enum name + variant name.
///
/// `Location::TopLeft` becomes `"enum-location-top-left"`.
fn variant_fluent_key(
    enum_name: &Ident,
    variant_ident: &Ident,
    attrs: &[syn::Attribute],
) -> String {
    for attr in attrs {
        if !attr.path().is_ident("wayle") {
            continue;
        }

        let mut fluent_key = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("fluent_key") {
                let value: syn::LitStr = meta.value()?.parse()?;
                fluent_key = Some(value.value());
            }
            Ok(())
        });

        if let Some(key) = fluent_key {
            return key;
        }
    }

    let enum_kebab = pascal_to_kebab(&enum_name.to_string());
    let variant_kebab = pascal_to_kebab(&variant_ident.to_string());

    format!("enum-{enum_kebab}-{variant_kebab}")
}

fn pascal_to_kebab(input: &str) -> String {
    let mut result = String::with_capacity(input.len() + 4);

    for (index, character) in input.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            result.push('-');
        }
        result.push(character.to_lowercase().next().unwrap_or(character));
    }

    result
}
