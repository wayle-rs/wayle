use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{UpdatesDropdown, helpers, messages::UpdatesDropdownCmd};

pub(super) fn spawn(
    sender: &ComponentSender<UpdatesDropdown>,
    config: &Arc<ConfigService>,
) {
    spawn_scale_watcher(sender, config);
    spawn_initial_check(sender, config);
}

fn spawn_scale_watcher(
    sender: &ComponentSender<UpdatesDropdown>,
    config: &Arc<ConfigService>,
) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(UpdatesDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

fn spawn_initial_check(
    sender: &ComponentSender<UpdatesDropdown>,
    config: &Arc<ConfigService>,
) {
    let official_cmd = config.config().modules.updates.check_official_command.get().clone();
    let aur_cmd = config.config().modules.updates.check_aur_command.get().clone();
    let flatpak_cmd = config.config().modules.updates.check_flatpak_command.get().clone();

    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                let _ = out.send(UpdatesDropdownCmd::SetChecking(true));

                let (pacman, aur, flatpak) = tokio::join!(
                    helpers::run_count_command(&official_cmd),
                    helpers::run_count_command(&aur_cmd),
                    helpers::run_count_command(&flatpak_cmd),
                );

                // If official returned 0 but others didn't, retry after delay
                // (checkupdates may fail on startup due to network not ready)
                if pacman == 0 && (aur > 0 || flatpak > 0) {
                    tracing::debug!("dropdown: official count was 0, retrying in 30s");
                    let _ = out.send(UpdatesDropdownCmd::UpdateCounts { pacman, aur, flatpak });

                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                    let pacman_retry = helpers::run_count_command(&official_cmd).await;
                    let _ = out.send(UpdatesDropdownCmd::UpdateCounts { pacman: pacman_retry, aur, flatpak });
                } else {
                    let _ = out.send(UpdatesDropdownCmd::UpdateCounts { pacman, aur, flatpak });
                }

                let _ = out.send(UpdatesDropdownCmd::SetChecking(false));
            })
            .drop_on_shutdown()
    });
}
