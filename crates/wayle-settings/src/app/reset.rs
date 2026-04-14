//! "Reset all" flow: confirmation modal wiring and the runtime-clear call.

use relm4::prelude::*;
use tracing::{info, warn};
use wayle_config::ConfigService;
use wayle_widgets::primitives::confirm_modal::{ConfirmModal, ConfirmModalOutput};

use super::{SettingsApp, SettingsAppMsg};

pub(super) fn build_confirm_modal(
    sender: &ComponentSender<SettingsApp>,
) -> Controller<ConfirmModal> {
    ConfirmModal::builder()
        .launch(())
        .forward(sender.input_sender(), |output| match output {
            ConfirmModalOutput::Confirmed => SettingsAppMsg::ExecuteResetAll,
            ConfirmModalOutput::Cancelled => SettingsAppMsg::Noop,
        })
}

pub(super) fn perform_reset_all(config_service: &ConfigService) {
    match config_service.reset_all_runtime() {
        Ok(()) => info!("all runtime overrides cleared"),
        Err(err) => warn!(error = %err, "reset-all partially failed"),
    }
}
