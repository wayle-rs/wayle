use relm4::ComponentController;
use wayle_brightness::BacklightDevice;
use wayle_config::schemas::{modules::BrightnessConfig, styling::evaluate_thresholds};
use wayle_widgets::prelude::BarButtonInput;

use super::{
    BrightnessModule,
    helpers::{IconContext, format_label, select_icon},
};

impl BrightnessModule {
    pub(super) fn update_display(&self, config: &BrightnessConfig, device: &BacklightDevice) {
        let percentage = device.percentage().value();

        let label = format_label(&config.format.get(), percentage);
        self.bar_button.emit(BarButtonInput::SetLabel(label));

        let icons = config.level_icons.get();
        let icon = select_icon(&IconContext {
            percentage,
            level_icons: &icons,
        });
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
    }

    pub(super) fn apply_thresholds(&self, config: &BrightnessConfig, device: &BacklightDevice) {
        let percentage = device.percentage().value();
        let colors = evaluate_thresholds(percentage, &config.thresholds.get());
        self.bar_button
            .emit(BarButtonInput::SetThresholdColors(colors));
    }
}
