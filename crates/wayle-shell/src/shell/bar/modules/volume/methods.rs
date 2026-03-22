use relm4::ComponentController;
use wayle_audio::core::device::output::OutputDevice;
use wayle_config::schemas::{modules::VolumeConfig, styling::evaluate_thresholds};
use wayle_widgets::prelude::BarButtonInput;

use super::{
    VolumeModule,
    helpers::{IconContext, format_label, select_icon},
};

impl VolumeModule {
    pub(super) fn update_display(&self, config: &VolumeConfig, device: &OutputDevice) {
        let percentage = device.volume.get().average_percentage().round() as u16;
        let muted = device.muted.get();

        let label = format_label(percentage);
        self.bar_button.emit(BarButtonInput::SetLabel(label));

        let icons = config.level_icons.get();
        let muted_icon_val = config.icon_muted.get();
        let icon = select_icon(&IconContext {
            percentage,
            muted,
            level_icons: &icons,
            muted_icon: &muted_icon_val,
        });
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
    }

    pub(super) fn apply_thresholds(&self, config: &VolumeConfig, device: &OutputDevice) {
        let percentage = device.volume.get().average_percentage().round() as u16;
        let colors = evaluate_thresholds(percentage as f64, &config.thresholds.get());
        self.bar_button
            .emit(BarButtonInput::SetThresholdColors(colors));
    }
}
