//! Verifies bar autohide configuration schema properties and TOML deserialization.

use wayle_config::schemas::bar::BarConfig;

#[test]
fn test_bar_autohide_defaults() {
    let config = BarConfig::default();
    assert!(!config.autohide.get());
    assert_eq!(config.autohide_timeout.get(), 3000);
    assert_eq!(config.autohide_trigger_size.get().value(), 2.0);
}

#[test]
fn test_bar_autohide_toml_deserialization() {
    let toml_str = r#"
        autohide = true
        autohide-timeout = 1500
        autohide-trigger-size = 4.0
    "#;
    let config: BarConfig = toml::from_str(toml_str).expect("Failed to deserialize BarConfig");
    assert!(config.autohide.get());
    assert_eq!(config.autohide_timeout.get(), 1500);
    assert_eq!(config.autohide_trigger_size.get().value(), 4.0);
}
