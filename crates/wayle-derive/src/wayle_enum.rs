//! `#[wayle_enum]` attribute macro for config enums.
//!
//! Injects the standard derive set and `#[serde(rename_all = "kebab-case")]`
//! unless the user already specified one. Serde and wayle attributes on
//! the enum are moved after the derive block to avoid the
//! "derive helper used before introduced" error.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{ItemEnum, parse_macro_input};

pub fn wayle_enum(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut parsed_enum = parse_macro_input!(item as ItemEnum);
    let has_default = parse_has_default(attr);

    let has_rename_all = parsed_enum.attrs.iter().any(|attr| {
        attr.path().is_ident("serde")
            && attr
                .meta
                .require_list()
                .is_ok_and(|list| list.tokens.to_string().contains("rename_all"))
    });

    let (derive_helper_attrs, other_attrs): (Vec<_>, Vec<_>) =
        parsed_enum.attrs.drain(..).partition(|attr| {
            attr.path().is_ident("serde") || attr.path().is_ident("wayle")
        });

    parsed_enum.attrs = other_attrs;

    let output = generate(parsed_enum, has_default, has_rename_all, derive_helper_attrs);
    output.into()
}

fn parse_has_default(attr: TokenStream) -> bool {
    let attr_str = attr.to_string();
    attr_str.contains("default")
}

fn generate(
    parsed_enum: ItemEnum,
    has_default: bool,
    has_rename_all: bool,
    derive_helper_attrs: Vec<syn::Attribute>,
) -> TokenStream2 {
    let enum_attrs = &parsed_enum.attrs;
    let enum_vis = &parsed_enum.vis;
    let enum_name = &parsed_enum.ident;
    let enum_variants = &parsed_enum.variants;

    let default_derive = if has_default {
        quote! { Default, }
    } else {
        quote! {}
    };

    let default_rename_all = if !has_rename_all {
        quote! { #[serde(rename_all = "kebab-case")] }
    } else {
        quote! {}
    };

    quote! {
        #(#enum_attrs)*
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            #default_derive
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            wayle_derive::EnumVariants,
        )]
        #default_rename_all
        #(#derive_helper_attrs)*
        #enum_vis enum #enum_name {
            #enum_variants
        }
    }
}
