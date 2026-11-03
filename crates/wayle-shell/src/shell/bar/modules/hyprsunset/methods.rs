use relm4::prelude::*;
use tracing::debug;
use wayle_config::schemas::modules::HyprsunsetConfig;
use wayle_widgets::prelude::BarButtonInput;

use super::{
    HyprsunsetModule,
    helpers::{self, LabelContext},
    messages::HyprsunsetCmd,
};

impl HyprsunsetModule {
    pub(super) fn toggle_filter(&self, sender: &ComponentSender<Self>, config: &HyprsunsetConfig) {
        let enabled = self.enabled;
        let temp = config.temperature.get();
        let gamma = config.gamma.get();
        let use_hyprsunset_config = config.use_hyprsunset_config.get();

        debug!(current_enabled = enabled, "toggle_filter called");

        sender.oneshot_command(async move {
            if enabled {
                debug!("stopping hyprsunset");
                let _ = helpers::stop().await;
                HyprsunsetCmd::StateChanged(None)
            } else {
                debug!(temp, gamma, use_hyprsunset_config, "starting hyprsunset");
                let overrides = (!use_hyprsunset_config).then_some((temp, gamma));
                let _ = helpers::start(overrides).await;
                HyprsunsetCmd::StateChanged(Some(helpers::HyprsunsetState { temp, gamma }))
            }
        });
    }

    pub(super) fn update_display(&self, config: &HyprsunsetConfig) {
        let icon =
            helpers::select_icon(self.enabled, &config.icon_off.get(), &config.icon_on.get());
        self.bar_button.emit(BarButtonInput::SetIcon(icon));

        let label = helpers::build_label(&LabelContext {
            format: &config.format.get(),
            temp: self.current_temp,
            gamma: self.current_gamma,
            config_temp: config.temperature.get(),
            config_gamma: config.gamma.get(),
            enabled: self.enabled,
        });
        self.bar_button.emit(BarButtonInput::SetLabel(label));
    }
}
