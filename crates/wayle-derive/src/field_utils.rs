//! Shared helpers for field attribute parsing across all macros.

use syn::{Attribute, Expr, Field};

/// Returns `true` if the field has `#[wayle(skip)]`. Skipped fields are
/// excluded from all generated trait impls: config/runtime layer operations,
/// change subscriptions, and path-based clearing.
pub fn should_skip(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("wayle") {
            return false;
        }

        let mut skip = false;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("skip") {
                skip = true;
            }
            Ok(())
        });

        skip
    })
}

/// Returns the TOML key for a field. Uses `#[serde(rename = "...")]` if present,
/// otherwise falls back to the Rust field name.
pub fn serde_key(field: &Field) -> String {
    for attr in &field.attrs {
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

    field
        .ident
        .as_ref()
        .map(|ident| ident.to_string())
        .unwrap_or_default()
}

/// Pulls `#[default(expr)]` out of a field's attributes.
/// Returns the default expression (if any) and the remaining attributes
/// with `#[default]` stripped.
pub fn extract_default_attr(attrs: &[Attribute]) -> syn::Result<(Option<Expr>, Vec<&Attribute>)> {
    let mut default_expr = None;
    let mut remaining = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("default") {
            if default_expr.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "duplicate #[default] attribute",
                ));
            }
            default_expr = Some(attr.parse_args::<Expr>()?);
        } else {
            remaining.push(attr);
        }
    }

    Ok((default_expr, remaining))
}

/// The `#[i18n]` attribute on a config field. Every `ConfigProperty`
/// field either maps to a fluent key for the settings GUI, or is skipped.
pub enum I18nAttr {
    /// `#[i18n("settings-bar-bg")]` - the fluent message ID used to
    /// look up this field's label and `.description` in the settings UI.
    Key(String),

    /// `#[i18n(skip)]` - don't show this field in the settings GUI.
    Skip,
}

/// Finds `#[i18n(...)]` among a field's attributes, parses it, and
/// returns everything else unchanged. Errors on duplicates or bad syntax.
pub fn extract_i18n_attr<'a>(
    attrs: &[&'a Attribute],
) -> syn::Result<(Option<I18nAttr>, Vec<&'a Attribute>)> {
    let mut i18n = None;
    let mut remaining = Vec::new();

    for &attr in attrs {
        if !attr.path().is_ident("i18n") {
            remaining.push(attr);
            continue;
        }

        if i18n.is_some() {
            return Err(syn::Error::new_spanned(attr, "duplicate #[i18n] attribute"));
        }

        i18n = Some(parse_i18n_attr(attr)?);
    }

    Ok((i18n, remaining))
}

/// `#[i18n("fluent-key")]` -> `Key("fluent-key")`
/// `#[i18n(skip)]` -> `Skip`
fn parse_i18n_attr(attr: &Attribute) -> syn::Result<I18nAttr> {
    let tokens = attr.meta.require_list()?.tokens.clone();
    let as_str: Result<syn::LitStr, _> = syn::parse2(tokens.clone());

    if let Ok(lit) = as_str {
        return Ok(I18nAttr::Key(lit.value()));
    }

    let as_ident: Result<syn::Ident, _> = syn::parse2(tokens);

    match as_ident {
        Ok(ident) if ident == "skip" => Ok(I18nAttr::Skip),
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[i18n(\"fluent-key\")] or #[i18n(skip)]",
        )),
    }
}
