//! Config-driven icon installation behind `wayle icons sync`.
//!
//! Reads the user's effective config, finds every icon name it references,
//! and installs whatever isn't already on disk.

use std::collections::{BTreeMap, BTreeSet};

use tracing::{debug, instrument};

use crate::{
    IconManager,
    sources::{self, CUSTOM_PREFIX, all_prefixes},
};

/// An icon referenced in config but not yet on disk.
#[derive(Debug, Clone)]
pub struct MissingIcon {
    /// Full icon name, e.g. `ld-bell-symbolic`.
    pub name: String,
    /// Where this icon would be installed from.
    pub origin: IconOrigin,
    /// Slug between the prefix and `-symbolic`, e.g. `bell`.
    pub slug: String,
}

/// How a missing icon can (or can't) be installed.
#[derive(Debug, Clone)]
pub enum IconOrigin {
    /// Auto-installable from a CDN source named by its CLI identifier
    /// (`lucide`, `tabler`, `tabler-filled`, `simple-icons`, `material`).
    Source(&'static str),
    /// User-imported via `wayle icons import`. Sync can't fetch these because
    /// the original SVG only lives on the source machine.
    UserImported,
}

/// Outcome of a sync run.
#[derive(Debug, Default)]
pub struct SyncSummary {
    /// Icons that finished installing.
    pub installed: Vec<String>,
    /// Icons that errored mid-install.
    pub failed: Vec<SyncFailure>,
    /// Icons that can't be auto-installed (typically `cm-` user-imported).
    pub skipped: Vec<String>,
}

/// One icon that errored out, with the message from its source.
#[derive(Debug)]
pub struct SyncFailure {
    /// `<source>/<slug>` identifier of the icon.
    pub name: String,
    /// Error message from the source.
    pub error: String,
}

/// Walks a config tree and returns every icon name it references.
///
/// Splits each string leaf on non-icon-name characters and keeps chunks
/// matching `<known-prefix>-<slug>-symbolic`. Schema-agnostic, so new
/// icon-bearing fields are picked up without code changes.
pub fn extract_referenced_icons(value: &toml::Value) -> BTreeSet<String> {
    let prefixes = all_prefixes();
    let mut icons = BTreeSet::new();
    walk_value(value, &prefixes, &mut icons);
    icons
}

/// Returns icons that are referenced but not present in `installed`.
///
/// Names that don't parse into a known prefix are dropped silently; callers
/// looking for "what's installed but not referenced" should diff the other
/// direction themselves.
pub fn find_missing(
    referenced: &BTreeSet<String>,
    installed: &BTreeSet<String>,
) -> Vec<MissingIcon> {
    let sources_lookup = build_sources_lookup();
    referenced
        .iter()
        .filter(|name| !installed.contains(*name))
        .filter_map(|name| MissingIcon::from_name(name, &sources_lookup))
        .collect()
}

/// Installs every CDN-resolvable icon in `missing`, one batch per source.
///
/// `UserImported` entries go to [`SyncSummary::skipped`] because the original
/// SVG isn't available to refetch. CDN failures go to [`SyncSummary::failed`]
/// with the source's error message attached.
#[instrument(skip(missing, manager), fields(missing_count = missing.len()))]
pub async fn install_missing(missing: Vec<MissingIcon>, manager: &IconManager) -> SyncSummary {
    let mut summary = SyncSummary::default();
    let mut slugs_by_source: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();

    for icon in missing {
        match icon.origin {
            IconOrigin::Source(source_name) => {
                slugs_by_source
                    .entry(source_name)
                    .or_default()
                    .push(icon.slug);
            }
            IconOrigin::UserImported => summary.skipped.push(icon.name),
        }
    }

    for (source_name, slugs) in slugs_by_source {
        install_from_source(source_name, slugs, manager, &mut summary).await;
    }

    summary
}

fn walk_value(value: &toml::Value, prefixes: &[&str], sink: &mut BTreeSet<String>) {
    match value {
        toml::Value::String(text) => scan_string(text, prefixes, sink),
        toml::Value::Array(items) => {
            for item in items {
                walk_value(item, prefixes, sink);
            }
        }
        toml::Value::Table(table) => {
            for child in table.values() {
                walk_value(child, prefixes, sink);
            }
        }
        _ => {}
    }
}

fn scan_string(text: &str, prefixes: &[&str], sink: &mut BTreeSet<String>) {
    for chunk in text.split(|character: char| !is_icon_name_char(character)) {
        if looks_like_icon(chunk, prefixes) {
            sink.insert(chunk.to_owned());
        }
    }
}

fn is_icon_name_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '-' || character == '_'
}

fn looks_like_icon(chunk: &str, prefixes: &[&str]) -> bool {
    let Some(stripped) = chunk.strip_suffix("-symbolic") else {
        return false;
    };
    let Some((prefix, slug)) = stripped.split_once('-') else {
        return false;
    };
    !slug.is_empty() && prefixes.contains(&prefix)
}

/// Prefix-to-CLI-name pairs, built once per `find_missing` call so each
/// classify lookup is a flat scan instead of reallocating the source list.
fn build_sources_lookup() -> Vec<(&'static str, &'static str)> {
    sources::all()
        .iter()
        .map(|source| (source.prefix(), source.cli_name()))
        .collect()
}

impl MissingIcon {
    /// Parses an icon name into its prefix, slug, and origin.
    ///
    /// Returns `None` for names that don't end in `-symbolic`, have an empty
    /// slug, or carry an unrecognized prefix.
    fn from_name(name: &str, sources_lookup: &[(&'static str, &'static str)]) -> Option<Self> {
        let stripped = name.strip_suffix("-symbolic")?;
        let (prefix, slug) = stripped.split_once('-')?;
        if slug.is_empty() {
            return None;
        }

        let origin = match cli_name_for_prefix(prefix, sources_lookup) {
            Some(cli_name) => IconOrigin::Source(cli_name),
            None if prefix == CUSTOM_PREFIX => IconOrigin::UserImported,
            None => return None,
        };

        Some(Self {
            name: name.to_owned(),
            origin,
            slug: slug.to_owned(),
        })
    }
}

fn cli_name_for_prefix(
    prefix: &str,
    sources_lookup: &[(&'static str, &'static str)],
) -> Option<&'static str> {
    sources_lookup
        .iter()
        .find(|(known_prefix, _)| *known_prefix == prefix)
        .map(|(_, cli_name)| *cli_name)
}

async fn install_from_source(
    source_name: &'static str,
    slugs: Vec<String>,
    manager: &IconManager,
    summary: &mut SyncSummary,
) {
    debug!(
        source = source_name,
        count = slugs.len(),
        "installing batch"
    );

    let source = match sources::from_cli_name(source_name) {
        Ok(source) => source,
        Err(err) => {
            record_failures(&mut summary.failed, source_name, &slugs, &err.to_string());
            return;
        }
    };

    let slug_refs: Vec<&str> = slugs.iter().map(String::as_str).collect();
    match manager.install(source.as_ref(), &slug_refs).await {
        Ok(result) => {
            summary.installed.extend(result.installed);
            summary
                .failed
                .extend(result.failed.into_iter().map(|failure| SyncFailure {
                    name: format!("{source_name}/{}", failure.slug),
                    error: failure.error,
                }));
        }
        Err(err) => record_failures(&mut summary.failed, source_name, &slugs, &err.to_string()),
    }
}

fn record_failures(
    failures: &mut Vec<SyncFailure>,
    source_name: &str,
    slugs: &[String],
    error: &str,
) {
    failures.extend(slugs.iter().map(|slug| SyncFailure {
        name: format!("{source_name}/{slug}"),
        error: error.to_owned(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn icons_in(text: &str) -> BTreeSet<String> {
        extract_referenced_icons(&toml::Value::String(text.to_owned()))
    }

    #[test]
    fn picks_known_prefixes_out_of_free_text() {
        let icons = icons_in("use ld-bell-symbolic and tb-alert-triangle-symbolic together");
        assert!(icons.contains("ld-bell-symbolic"));
        assert!(icons.contains("tb-alert-triangle-symbolic"));
    }

    #[test]
    fn keeps_hyphenated_slugs_intact() {
        let icons = icons_in("ld-alert-triangle-symbolic");
        assert!(icons.contains("ld-alert-triangle-symbolic"));
    }

    #[test]
    fn rejects_unknown_prefixes() {
        assert!(icons_in("xy-foo-symbolic").is_empty());
    }

    #[test]
    fn rejects_missing_symbolic_suffix() {
        assert!(icons_in("ld-bell ld-home").is_empty());
    }

    #[test]
    fn keeps_tbf_separate_from_tb() {
        let icons = icons_in("tbf-circle-symbolic");
        assert!(icons.contains("tbf-circle-symbolic"));
    }

    #[test]
    fn walks_nested_tables() -> Result<(), toml::de::Error> {
        let toml_text = r#"
            [modules.notification]
            icon-name = "ld-bell-symbolic"
            [modules.power]
            icon-name = "ld-power-symbolic"
        "#;
        let icons = extract_referenced_icons(&toml::from_str(toml_text)?);
        assert!(icons.contains("ld-bell-symbolic"));
        assert!(icons.contains("ld-power-symbolic"));
        Ok(())
    }

    #[test]
    fn classifies_cdn_icon_with_source() -> Result<(), &'static str> {
        let lookup = build_sources_lookup();
        let icon = MissingIcon::from_name("ld-bell-symbolic", &lookup)
            .ok_or("ld- prefix should classify")?;
        assert!(matches!(icon.origin, IconOrigin::Source("lucide")));
        assert_eq!(icon.slug, "bell");
        Ok(())
    }

    #[test]
    fn classifies_custom_icon_as_user_imported() -> Result<(), &'static str> {
        let lookup = build_sources_lookup();
        let icon = MissingIcon::from_name("cm-offline-symbolic", &lookup)
            .ok_or("cm- prefix should classify")?;
        assert!(matches!(icon.origin, IconOrigin::UserImported));
        Ok(())
    }

    #[test]
    fn classifies_hyphenated_slug() -> Result<(), &'static str> {
        let lookup = build_sources_lookup();
        let icon = MissingIcon::from_name("ld-arrow-left-symbolic", &lookup)
            .ok_or("hyphenated slug should classify")?;
        assert_eq!(icon.slug, "arrow-left");
        Ok(())
    }

    #[test]
    fn classify_rejects_unknown_prefix() {
        let lookup = build_sources_lookup();
        assert!(MissingIcon::from_name("xy-foo-symbolic", &lookup).is_none());
    }

    #[test]
    fn find_missing_returns_only_uninstalled() {
        let referenced: BTreeSet<String> = [
            "ld-bell-symbolic",
            "tb-alert-triangle-symbolic",
            "ld-power-symbolic",
        ]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
        let installed: BTreeSet<String> = ["ld-bell-symbolic"]
            .iter()
            .map(|name| (*name).to_owned())
            .collect();

        let missing = find_missing(&referenced, &installed);
        let names: Vec<_> = missing.iter().map(|icon| icon.name.as_str()).collect();
        assert_eq!(names, ["ld-power-symbolic", "tb-alert-triangle-symbolic"]);
    }
}
