#![allow(missing_docs)]

use wayle_config::{ApplyConfigLayer, CommitConfigReload, ConfigProperty};
use wayle_derive::{ApplyConfigLayer, CommitConfigReload};

#[derive(ApplyConfigLayer, CommitConfigReload)]
struct SimpleConfig {
    enabled: ConfigProperty<bool>,
    count: ConfigProperty<u32>,
}

#[test]
fn updates_all_fields_from_table() {
    let config = SimpleConfig {
        enabled: ConfigProperty::new(false),
        count: ConfigProperty::new(0),
    };

    let toml_value = toml::toml! {
        enabled = true
        count = 42
    };

    config.apply_config_layer(&toml::Value::Table(toml_value), "");
    config.commit_config_reload();

    assert!(config.enabled.get());
    assert_eq!(config.count.get(), 42);
}

#[test]
fn updates_partial_fields() {
    let config = SimpleConfig {
        enabled: ConfigProperty::new(false),
        count: ConfigProperty::new(10),
    };

    let toml_value = toml::toml! {
        enabled = true
    };

    config.apply_config_layer(&toml::Value::Table(toml_value), "");
    config.commit_config_reload();

    assert!(config.enabled.get());
    assert_eq!(config.count.get(), 10);
}

#[test]
fn ignores_unknown_fields() {
    let config = SimpleConfig {
        enabled: ConfigProperty::new(false),
        count: ConfigProperty::new(0),
    };

    let toml_value = toml::toml! {
        enabled = true
        unknown_field = "ignored"
        count = 99
    };

    config.apply_config_layer(&toml::Value::Table(toml_value), "");
    config.commit_config_reload();

    assert!(config.enabled.get());
    assert_eq!(config.count.get(), 99);
}

#[test]
fn handles_non_table_value() {
    let config = SimpleConfig {
        enabled: ConfigProperty::new(true),
        count: ConfigProperty::new(5),
    };

    config.apply_config_layer(&toml::Value::String("not a table".to_string()), "");
    config.commit_config_reload();

    assert!(config.enabled.get());
    assert_eq!(config.count.get(), 5);
}

#[test]
fn handles_empty_table() {
    let config = SimpleConfig {
        enabled: ConfigProperty::new(true),
        count: ConfigProperty::new(5),
    };

    use toml::map::Map;
    let toml_value = Map::new();

    config.apply_config_layer(&toml::Value::Table(toml_value), "");
    config.commit_config_reload();

    assert!(config.enabled.get());
    assert_eq!(config.count.get(), 5);
}

#[derive(ApplyConfigLayer, CommitConfigReload)]
struct NestedConfig {
    simple: SimpleConfig,
    name: ConfigProperty<String>,
}

#[test]
fn updates_nested_structs() {
    let config = NestedConfig {
        simple: SimpleConfig {
            enabled: ConfigProperty::new(false),
            count: ConfigProperty::new(0),
        },
        name: ConfigProperty::new("old".to_string()),
    };

    let toml_value = toml::toml! {
        name = "new"
        [simple]
        enabled = true
        count = 100
    };

    config.apply_config_layer(&toml::Value::Table(toml_value), "");
    config.commit_config_reload();

    assert_eq!(config.name.get(), "new");
    assert!(config.simple.enabled.get());
    assert_eq!(config.simple.count.get(), 100);
}
