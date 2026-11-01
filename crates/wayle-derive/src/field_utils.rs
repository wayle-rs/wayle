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

/// A TOML key, plus whether matching it should warn the user.
pub struct LookupKey {
    /// The key.
    pub name: String,
    /// `true` for `#[wayle(deprecated_alias)]` keys; matches emit a `tracing::warn!`.
    pub deprecated: bool,
}

/// Returns the canonical TOML key for a field. Uses `#[serde(rename = "...")]`
/// if present, otherwise falls back to the Rust field name. Aliases are
/// ignored; use [`serde_keys`] for read paths that should accept aliases too.
pub fn serde_key(field: &Field) -> String {
    serde_keys(field)
        .into_iter()
        .next()
        .map(|key| key.name)
        .unwrap_or_default()
}

/// Returns all TOML keys a field can match on read.
///
/// Order: canonical first (`#[serde(rename)]` or Rust field name), then any
/// `#[serde(alias = "...")]` entries (non-deprecated synonyms), then any
/// `#[wayle(deprecated_alias = "...")]` entries (legacy names that emit a
/// runtime deprecation warning when matched).
pub fn serde_keys(field: &Field) -> Vec<LookupKey> {
    let mut rename: Option<String> = None;
    let mut serde_aliases: Vec<String> = Vec::new();
    let mut deprecated_aliases: Vec<String> = Vec::new();

    for attr in &field.attrs {
        collect_serde_keys(attr, &mut rename, &mut serde_aliases);
        collect_deprecated_aliases(attr, &mut deprecated_aliases);
    }

    let canonical = rename.unwrap_or_else(|| field_ident_string(field));

    let mut keys = Vec::with_capacity(1 + serde_aliases.len() + deprecated_aliases.len());
    keys.push(LookupKey {
        name: canonical,
        deprecated: false,
    });
    keys.extend(serde_aliases.into_iter().map(|name| LookupKey {
        name,
        deprecated: false,
    }));
    keys.extend(deprecated_aliases.into_iter().map(|name| LookupKey {
        name,
        deprecated: true,
    }));
    keys
}

fn collect_serde_keys(
    attr: &Attribute,
    rename: &mut Option<String>,
    serde_aliases: &mut Vec<String>,
) {
    if !attr.path().is_ident("serde") {
        return;
    }

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            *rename = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            return Ok(());
        }
        if meta.path.is_ident("alias") {
            serde_aliases.push(meta.value()?.parse::<syn::LitStr>()?.value());
        }
        Ok(())
    });
}

fn collect_deprecated_aliases(attr: &Attribute, deprecated_aliases: &mut Vec<String>) {
    if !attr.path().is_ident("wayle") {
        return;
    }

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("deprecated_alias") {
            deprecated_aliases.push(meta.value()?.parse::<syn::LitStr>()?.value());
        }
        Ok(())
    });
}

fn field_ident_string(field: &Field) -> String {
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
