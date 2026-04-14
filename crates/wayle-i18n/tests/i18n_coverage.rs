//! Verifies that every auto-generated i18n key from the config schema
//! has a matching FTL entry with a `.description` attribute.
//!
//! Runs as part of `cargo test -p wayle-i18n`. If this fails, a config
//! field was added without a corresponding FTL entry in
//! `crates/wayle-i18n/locales/en-US/config/`.

use wayle_config::schemas::{
    bar::BarConfig, general::GeneralConfig, modules::ModulesConfig, osd::OsdConfig,
    styling::StylingConfig, wallpaper::WallpaperConfig,
};
use wayle_i18n::loader;

#[test]
fn all_config_i18n_keys_have_ftl_entries() {
    let ftl = loader();

    let mut missing_keys = Vec::new();
    let mut missing_descriptions = Vec::new();

    let all_keys = collect_all_keys();

    for key in &all_keys {
        if !ftl.has(key) {
            missing_keys.push(key.to_string());
            continue;
        }

        if !ftl.has_attr(key, "description") {
            missing_descriptions.push(key.to_string());
        }
    }

    let mut failures = String::new();

    if !missing_keys.is_empty() {
        failures.push_str(&format!(
            "\n{} keys missing from FTL:\n",
            missing_keys.len()
        ));

        for key in &missing_keys {
            failures.push_str(&format!("  {key}\n"));
        }
    }

    if !missing_descriptions.is_empty() {
        failures.push_str(&format!(
            "\n{} keys missing .description:\n",
            missing_descriptions.len()
        ));

        for key in &missing_descriptions {
            failures.push_str(&format!("  {key}\n"));
        }
    }

    assert!(
        failures.is_empty(),
        "FTL coverage failures ({} keys checked):{failures}",
        all_keys.len()
    );
}

fn collect_all_keys() -> Vec<&'static str> {
    let mut keys = Vec::new();

    keys.extend(GeneralConfig::all_i18n_keys());
    keys.extend(BarConfig::all_i18n_keys());
    keys.extend(StylingConfig::all_i18n_keys());
    keys.extend(OsdConfig::all_i18n_keys());
    keys.extend(WallpaperConfig::all_i18n_keys());
    keys.extend(ModulesConfig::all_i18n_keys());

    keys
}
