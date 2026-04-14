//! Source-info derivation and badge/reset visibility predicates.

use wayle_config::ValueSource;
use wayle_i18n::{t, t_attr};

use super::{RowBehavior, SettingRow};

impl SettingRow {
    pub(super) fn has_runtime_override(&self) -> bool {
        self.behavior == RowBehavior::Setting
            && matches!(
                self.source,
                ValueSource::RuntimeOnly | ValueSource::Overridden
            )
    }

    pub(super) fn has_source_badge(&self) -> bool {
        self.behavior == RowBehavior::Setting
            && self.source != ValueSource::Default
            && !self.config_matches_default
    }

    pub(super) fn source_css_class(&self) -> &'static str {
        match self.source {
            ValueSource::Default => "",
            ValueSource::Config => "info",
            ValueSource::RuntimeOnly => "success",
            ValueSource::Overridden => "warning",
        }
    }

    pub(super) fn update_source_info(&mut self) {
        self.source = self.handle.source();

        self.source_label = match self.source {
            ValueSource::Default => String::new(),
            ValueSource::Config => t("settings-source-config"),
            ValueSource::RuntimeOnly => t("settings-source-custom"),
            ValueSource::Overridden => t("settings-source-override"),
        };

        self.source_tooltip = match self.source {
            ValueSource::Default => String::new(),
            ValueSource::Config => t_attr("settings-source-config", "description"),
            ValueSource::RuntimeOnly => t_attr("settings-source-custom", "description"),
            ValueSource::Overridden => t_attr("settings-source-override", "description"),
        };

        self.config_matches_default = self.source == ValueSource::Config
            && self.handle.config_display() == Some(self.handle.default_display());
    }
}
