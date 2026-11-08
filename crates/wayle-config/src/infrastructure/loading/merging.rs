use toml::{Value, map::Map};

pub(super) fn merge_toml_configs(imports: Vec<Value>, main: Value) -> Value {
    let mut accumulated = Value::Table(Map::new());

    for import in imports {
        accumulated = merge_two_toml_configs(accumulated, import, false);
    }

    merge_two_toml_configs(accumulated, main, false)
}

fn merge_two_toml_configs(base: Value, overlay: Value, merge_custom_modules: bool) -> Value {
    match (base, overlay) {
        (Value::Table(base_table), Value::Table(overlay_table)) => {
            let mut merged_table = overlay_table;

            for (key, base_value) in base_table {
                match merged_table.remove(&key) {
                    None => {
                        merged_table.insert(key, base_value);
                    }
                    Some(overlay_value) => {
                        let merged_value = merge_two_toml_configs(
                            base_value,
                            overlay_value,
                            (!merge_custom_modules && key == "modules")
                                || (merge_custom_modules && key == "custom"),
                        );
                        merged_table.insert(key, merged_value);
                    }
                }
            }

            Value::Table(merged_table)
        }
        (Value::Array(mut base_array), Value::Array(overlay_array)) if merge_custom_modules => {
            base_array.extend(overlay_array);
            Value::Array(base_array)
        }
        (_, overlay) => overlay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod merge_two_toml_configs {
        use super::*;

        #[test]
        fn overlay_wins_for_same_key() {
            let base = toml::toml! { enabled = false };
            let overlay = toml::toml! { enabled = true };

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
        }

        #[test]
        fn preserves_base_keys_not_in_overlay() {
            let base = toml::toml! {
                enabled = true
                count = 42
            };
            let overlay = toml::toml! {
                enabled = false
            };

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(false)));
            assert_eq!(table.get("count"), Some(&Value::Integer(42)));
        }

        #[test]
        fn merges_nested_tables() {
            let base = toml::toml! {
                [clock]
                enabled = true
                format = "%H:%M"
            };
            let overlay = toml::toml! {
                [clock]
                format = "%I:%M %p"
            };

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let clock = result.get("clock").unwrap().as_table().unwrap();

            assert_eq!(clock.get("enabled"), Some(&Value::Boolean(true)));
            assert_eq!(clock.get("format"), Some(&Value::String("%I:%M %p".into())));
        }

        #[test]
        fn overlay_replaces_base_table_with_primitive() {
            let base = toml::toml! {
                [clock]
                enabled = true
            };
            let overlay = toml::toml! {
                clock = "disabled"
            };

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let table = result.as_table().unwrap();

            assert_eq!(table.get("clock"), Some(&Value::String("disabled".into())));
        }

        #[test]
        fn handles_empty_overlay() {
            let base = toml::toml! {
                enabled = true
                count = 42
            };
            let overlay = Map::new();

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
            assert_eq!(table.get("count"), Some(&Value::Integer(42)));
        }

        #[test]
        fn handles_empty_base() {
            let base = Map::new();
            let overlay = toml::toml! {
                enabled = true
            };

            let result = merge_two_toml_configs(Value::Table(base), Value::Table(overlay), false);
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
        }
    }

    mod merge_toml_configs {
        use super::*;

        #[test]
        fn main_config_takes_precedence() {
            let import1 = toml::toml! { enabled = false };
            let main = toml::toml! { enabled = true };

            let result = merge_toml_configs(vec![Value::Table(import1)], Value::Table(main));
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
        }

        #[test]
        fn later_imports_override_earlier() {
            let import1 = toml::toml! { count = 1 };
            let import2 = toml::toml! { count = 2 };
            let main = Map::new();

            let result = merge_toml_configs(
                vec![Value::Table(import1), Value::Table(import2)],
                Value::Table(main),
            );
            let table = result.as_table().unwrap();

            assert_eq!(table.get("count"), Some(&Value::Integer(2)));
        }

        #[test]
        fn accumulates_all_keys() {
            let import1 = toml::toml! { key1 = "a" };
            let import2 = toml::toml! { key2 = "b" };
            let main = toml::toml! { key3 = "c" };

            let result = merge_toml_configs(
                vec![Value::Table(import1), Value::Table(import2)],
                Value::Table(main),
            );
            let table = result.as_table().unwrap();

            assert_eq!(table.get("key1"), Some(&Value::String("a".into())));
            assert_eq!(table.get("key2"), Some(&Value::String("b".into())));
            assert_eq!(table.get("key3"), Some(&Value::String("c".into())));
        }

        #[test]
        fn handles_no_imports() {
            let main = toml::toml! { enabled = true };

            let result = merge_toml_configs(vec![], Value::Table(main));
            let table = result.as_table().unwrap();

            assert_eq!(table.get("enabled"), Some(&Value::Boolean(true)));
        }

        #[test]
        fn merges_custom_module_arrays_from_imports() {
            let import1 = toml::toml! {
                [modules]
                custom = [{ id = "module-a" }]
            };
            let import2 = toml::toml! {
                [modules]
                custom = [{ id = "module-b" }]
            };

            let result = merge_toml_configs(
                vec![Value::Table(import1), Value::Table(import2)],
                Value::Table(Map::new()),
            );
            let custom = result["modules"]["custom"].as_array().unwrap();

            assert_eq!(custom.len(), 2);
            assert_eq!(custom[0]["id"], Value::String("module-a".into()));
            assert_eq!(custom[1]["id"], Value::String("module-b".into()));
        }

        #[test]
        fn replaces_non_custom_arrays() {
            let import = toml::toml! {
                [modules]
                order = ["module-a"]
            };
            let main = toml::toml! {
                [modules]
                order = ["module-b"]
            };

            let result = merge_toml_configs(vec![Value::Table(import)], Value::Table(main));

            assert_eq!(
                result["modules"]["order"],
                toml::Value::Array(vec![toml::Value::String("module-b".into())])
            );
        }
    }
}
