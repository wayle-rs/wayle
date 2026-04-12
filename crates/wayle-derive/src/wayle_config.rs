//! `#[wayle_config]` attribute macro for config structs.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, FieldsNamed, ItemStruct, parse_macro_input};

use crate::field_utils::{I18nAttr, extract_default_attr, extract_i18n_attr, serde_key};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigType {
    Standard,
    BarButton,
    BarContainer,
}

/// Parsed arguments from `#[wayle_config(...)]`.
struct MacroArgs {
    config_type: ConfigType,
    i18n_prefix: Option<String>,
}

/// Entry point for the `#[wayle_config]` attribute macro.
///
/// Accepts `bar_button` or `bar_container` to validate required fields,
/// and `i18n_prefix = "settings-modules-clock"` to auto-generate fluent
/// keys for each `ConfigProperty` field (prefix + serde key).
pub fn wayle_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(item as ItemStruct);
    let args = parse_args(attr);

    match generate(parsed_struct, args) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn parse_args(attr: TokenStream) -> MacroArgs {
    let mut args = MacroArgs {
        config_type: ConfigType::Standard,
        i18n_prefix: None,
    };

    if attr.is_empty() {
        return args;
    }

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("bar_button") {
            args.config_type = ConfigType::BarButton;
        } else if meta.path.is_ident("bar_container") {
            args.config_type = ConfigType::BarContainer;
        } else if meta.path.is_ident("i18n_prefix") {
            let value: syn::LitStr = meta.value()?.parse()?;
            args.i18n_prefix = Some(value.value());
        }
        Ok(())
    });

    let _ = syn::parse::Parser::parse(parser, attr);

    args
}

fn generate(parsed_struct: ItemStruct, args: MacroArgs) -> syn::Result<TokenStream2> {
    let struct_name = &parsed_struct.ident;
    let struct_vis = &parsed_struct.vis;
    let struct_generics = &parsed_struct.generics;
    let struct_attrs = &parsed_struct.attrs;

    let struct_fields = match &parsed_struct.fields {
        Fields::Named(named_fields) => named_fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &parsed_struct,
                "wayle_config only supports structs with named fields",
            ));
        }
    };

    match args.config_type {
        ConfigType::BarButton => validate_bar_button(struct_fields)?,
        ConfigType::BarContainer => validate_bar_container(struct_fields)?,
        ConfigType::Standard => {}
    }

    let processed = process_fields(struct_fields, args.i18n_prefix.as_deref())?;
    let field_tokens = &processed.field_tokens;
    let default_initializers = &processed.default_initializers;
    let i18n_keys = &processed.i18n_keys;

    let child_key_calls = processed.container_fields.iter().map(|(_, field_type)| {
        quote! { keys.extend(<#field_type>::all_i18n_keys()); }
    });

    Ok(quote! {
        #(#struct_attrs)*
        #[derive(
            Debug,
            Clone,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            wayle_derive::ApplyConfigLayer,
            wayle_derive::ApplyRuntimeLayer,
            wayle_derive::ExtractRuntimeValues,
            wayle_derive::SubscribeChanges,
            wayle_derive::ResetConfigLayer,
            wayle_derive::ResetRuntimeLayer,
            wayle_derive::ClearAllRuntime,
            wayle_derive::ClearRuntimeByPath,
            wayle_derive::CommitConfigReload,
        )]
        #[serde(default)]
        #struct_vis struct #struct_name #struct_generics {
            #(#field_tokens),*
        }

        impl #struct_generics Default for #struct_name #struct_generics {
            fn default() -> Self {
                Self {
                    #(#default_initializers),*
                }
            }
        }

        impl #struct_generics #struct_name #struct_generics {
            /// All fluent i18n keys for this struct and its children.
            /// Used by CI to verify every key has an FTL entry.
            pub fn all_i18n_keys() -> Vec<&'static str> {
                let mut keys: Vec<&'static str> = vec![#(#i18n_keys),*];
                #(#child_key_calls)*
                keys
            }
        }
    })
}

struct ProcessedFields {
    field_tokens: Vec<TokenStream2>,
    default_initializers: Vec<TokenStream2>,
    i18n_keys: Vec<String>,
    container_fields: Vec<(syn::Ident, syn::Type)>,
}

fn process_fields(fields: &FieldsNamed, i18n_prefix: Option<&str>) -> syn::Result<ProcessedFields> {
    let mut output = ProcessedFields {
        field_tokens: Vec::new(),
        default_initializers: Vec::new(),
        i18n_keys: Vec::new(),
        container_fields: Vec::new(),
    };

    for field in &fields.named {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;

        let field_type = &field.ty;
        let field_visibility = &field.vis;

        let (default_expr, attrs_without_default) = extract_default_attr(&field.attrs)?;
        let (i18n_attr, passthrough_attrs) = extract_i18n_attr(&attrs_without_default)?;

        let is_config_property = default_expr.is_some();

        let i18n_key = match &i18n_attr {
            Some(I18nAttr::Key(explicit_key)) => Some(explicit_key.clone()),

            Some(I18nAttr::Skip) => None,

            None if is_config_property => i18n_prefix.map(|prefix| {
                let serde_key = serde_key(field);
                format!("{prefix}-{serde_key}")
            }),

            None => None,
        };

        if let Some(ref key) = i18n_key {
            output.i18n_keys.push(key.clone());
        }

        let is_wayle_skipped = field.attrs.iter().any(|attr| {
            attr.path().is_ident("wayle")
                && attr
                    .meta
                    .require_list()
                    .is_ok_and(|list| list.tokens.to_string().contains("skip"))
        });

        if !is_config_property && !is_wayle_skipped {
            output
                .container_fields
                .push((field_ident.clone(), field_type.clone()));
        }

        output.field_tokens.push(quote! {
            #(#passthrough_attrs)*
            #field_visibility #field_ident: #field_type
        });

        let initializer = match (&default_expr, &i18n_key) {
            (Some(expr), Some(key)) => {
                quote! { #field_ident: wayle_config::ConfigProperty::with_i18n_key(#expr, #key) }
            }
            (Some(expr), None) => {
                quote! { #field_ident: wayle_config::ConfigProperty::new(#expr) }
            }
            (None, _) => {
                quote! { #field_ident: Default::default() }
            }
        };

        output.default_initializers.push(initializer);
    }

    Ok(output)
}

const BAR_BUTTON_REQUIRED: &[&str] = &[
    "border_show",
    "border_color",
    "icon_show",
    "icon_color",
    "icon_bg_color",
    "label_show",
    "label_color",
    "label_max_length",
    "button_bg_color",
    "left_click",
    "right_click",
    "middle_click",
    "scroll_up",
    "scroll_down",
];

const BAR_CONTAINER_REQUIRED: &[&str] = &["border_show", "border_color", "button_bg_color"];

fn validate_bar_button(fields: &FieldsNamed) -> syn::Result<()> {
    validate_required_fields(fields, BAR_BUTTON_REQUIRED, "bar_button")
}

fn validate_bar_container(fields: &FieldsNamed) -> syn::Result<()> {
    validate_required_fields(fields, BAR_CONTAINER_REQUIRED, "bar_container")
}

fn validate_required_fields(
    fields: &FieldsNamed,
    required: &[&str],
    config_name: &str,
) -> syn::Result<()> {
    let field_names: Vec<String> = fields
        .named
        .iter()
        .filter_map(|field| field.ident.as_ref().map(|ident| ident.to_string()))
        .collect();

    let missing: Vec<&str> = required
        .iter()
        .filter(|req| !field_names.contains(&(**req).to_string()))
        .copied()
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        fields,
        format!(
            "{config_name} config missing required fields: {}",
            missing.join(", ")
        ),
    ))
}
