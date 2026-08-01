//! Live source of the dropdowns a module's click actions can open.

/// A value that can report which dropdowns its click actions currently open.
///
/// `#[wayle_config]` implements this for any config with `ConfigProperty<ClickAction>`
/// fields, reading the properties *live* — so a bar module that hands its config to a
/// dropdown opener exposes the up-to-date set even after the click bindings are
/// re-configured at runtime, keeping `wayle dropdown list` and CLI addressing in sync
/// without any per-module wiring. Types whose click actions don't live on a config
/// (e.g. the `custom` module's per-instance definition) implement it by hand.
pub trait DropdownSources {
    /// The distinct dropdown names this value's click actions currently open, in
    /// first-seen order.
    fn dropdown_names(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use crate::{Config, DropdownSources};

    #[test]
    fn wayle_config_derives_dropdown_names_from_click_actions() {
        // The `#[wayle_config]` macro implements `DropdownSources` for any config with
        // `ConfigProperty<ClickAction>` fields. Battery's default left-click opens the
        // "battery" dropdown; its other clicks default to `ClickAction::None` and are
        // excluded — so the derived names are exactly `["battery"]`. This locks the
        // structural `ConfigProperty<ClickAction>` match in the derive macro.
        let config = Config::default();
        assert_eq!(
            config.modules.battery.dropdown_names(),
            vec!["battery".to_string()],
        );
    }
}
