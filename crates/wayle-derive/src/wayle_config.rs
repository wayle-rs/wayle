//! `#[wayle_config]` attribute macro for config structs.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Fields, FieldsNamed, Ident, ItemStruct, parse_macro_input};

use crate::field_utils::{I18nAttr, extract_default_attr, extract_i18n_attr};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConfigType {
    Standard,
    BarButton,
    BarContainer,
}

/// Entry point for the `#[wayle_config]` attribute macro. Accepts optional
/// `bar_button` or `bar_container` arguments to validate required fields.
pub fn wayle_config(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_struct = parse_macro_input!(item as ItemStruct);
    let config_type = parse_config_type(attr);

    match generate(parsed_struct, config_type) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn parse_config_type(attr: TokenStream) -> ConfigType {
    if attr.is_empty() {
        return ConfigType::Standard;
    }

    let attr2: TokenStream2 = attr.into();
    let ident: Result<Ident, _> = syn::parse2(attr2);

    match ident {
        Ok(id) if id == "bar_button" => ConfigType::BarButton,
        Ok(id) if id == "bar_container" => ConfigType::BarContainer,
        _ => ConfigType::Standard,
    }
}

fn generate(parsed_struct: ItemStruct, config_type: ConfigType) -> syn::Result<TokenStream2> {
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

    match config_type {
        ConfigType::BarButton => validate_bar_button(struct_fields)?,
        ConfigType::BarContainer => validate_bar_container(struct_fields)?,
        ConfigType::Standard => {}
    }

    let processed = process_fields(struct_fields)?;
    let field_tokens = &processed.field_tokens;
    let default_initializers = &processed.default_initializers;

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
    })
}

struct ProcessedFields {
    field_tokens: Vec<TokenStream2>,
    default_initializers: Vec<TokenStream2>,
    i18n_keys: Vec<(syn::Ident, String)>,
}

fn process_fields(fields: &FieldsNamed) -> syn::Result<ProcessedFields> {
    let mut output = ProcessedFields {
        field_tokens: Vec::new(),
        default_initializers: Vec::new(),
        i18n_keys: Vec::new(),
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

        if is_config_property && i18n_attr.is_none() {
            return Err(syn::Error::new_spanned(
                field,
                format!(
                    "ConfigProperty `{field_ident}` needs #[i18n(\"fluent-key\")] or #[i18n(skip)]"
                ),
            ));
        }

        if let Some(I18nAttr::Key(ref fluent_key)) = i18n_attr {
            output
                .i18n_keys
                .push((field_ident.clone(), fluent_key.clone()));
        }

        output.field_tokens.push(quote! {
            #(#passthrough_attrs)*
            #field_visibility #field_ident: #field_type
        });

        let initializer = match (&default_expr, &i18n_attr) {
            (Some(expr), Some(I18nAttr::Key(key))) => {
                quote! { #field_ident: wayle_config::ConfigProperty::with_i18n_key(#expr, #key) }
            }
            (Some(expr), _) => {
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
