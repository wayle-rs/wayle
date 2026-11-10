use std::sync::Arc;

use relm4::ComponentController;
use wayle_config::schemas::modules::MullvadConfig;
use wayle_mullvad::{ConnectionStatus, MullvadService};
use wayle_widgets::prelude::BarButtonInput;

use super::{
    MullvadModule,
    helpers::{format_label, select_icon},
};

impl MullvadModule {
    pub(super) fn compute_display(
        config: &MullvadConfig,
        mullvad: &Option<Arc<MullvadService>>,
    ) -> (String, String) {
        // No service = daemon unavailable: render like a logged-out account
        // (disabled icon + logged-out label). The login state is folded into
        // `status`, so everything else follows from it.
        let status = mullvad
            .as_ref()
            .map_or(ConnectionStatus::LoggedOut, |mullvad| {
                mullvad.mullvad.status.get()
            });
        (select_icon(config, &status), format_label(&status))
    }

    pub(super) fn update_display(
        &self,
        config: &MullvadConfig,
        mullvad: &Option<Arc<MullvadService>>,
    ) {
        let (icon, label) = Self::compute_display(config, mullvad);
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
        self.bar_button.emit(BarButtonInput::SetLabel(label));
    }
}
