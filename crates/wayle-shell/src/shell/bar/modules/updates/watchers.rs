use std::time::Duration;

use relm4::ComponentSender;
use wayle_config::schemas::{modules::UpdatesConfig, styling::evaluate_thresholds};
use wayle_widgets::watch;

use super::{
    UpdatesModule,
    helpers::{self, UpdateCounts},
    messages::UpdatesCmd,
};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<UpdatesModule>,
    config: &UpdatesConfig,
) {
    let format = config.format.clone();
    let thresholds = config.thresholds.clone();
    let poll_interval = config.poll_interval_ms.clone();
    let official_cmd = config.check_official_command.clone();
    let aur_cmd = config.check_aur_command.clone();
    let flatpak_cmd = config.check_flatpak_command.clone();
    let hide_if_zero = config.hide_if_zero.clone();

    // Poll for updates on interval
    let format_poll = format.clone();
    let thresholds_poll = thresholds.clone();
    let poll_ms = poll_interval.clone();
    let official_poll = official_cmd.clone();
    let aur_poll = aur_cmd.clone();
    let flatpak_poll = flatpak_cmd.clone();
    let hide_poll = hide_if_zero.clone();
    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                // Wait for network to be ready on login before first check
                tokio::time::sleep(Duration::from_secs(2)).await;

                let mut first_run = true;
                loop {
                    let counts = helpers::check_updates(
                        &official_poll.get(),
                        &aur_poll.get(),
                        &flatpak_poll.get(),
                    )
                    .await;

                    let label = helpers::format_label(&format_poll.get(), &counts);
                    let _ = out.send(UpdatesCmd::UpdateLabel(label));

                    let colors =
                        evaluate_thresholds(counts.total() as f64, &thresholds_poll.get());
                    let _ = out.send(UpdatesCmd::UpdateThresholdColors(colors));

                    if hide_poll.get() {
                        let _ = out.send(UpdatesCmd::UpdateVisibility(counts.total() > 0));
                    }

                    // On first run, if official returned 0 but others didn't,
                    // retry quickly (checkupdates may have failed due to network)
                    if first_run && counts.pacman == 0 && (counts.aur > 0 || counts.flatpak > 0) {
                        first_run = false;
                        tracing::debug!("official count was 0 on first run, retrying in 30s");
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        continue;
                    }
                    first_run = false;

                    let interval = Duration::from_millis(poll_ms.get());
                    tokio::time::sleep(interval).await;
                }
            })
            .drop_on_shutdown()
    });

    // Watch format changes
    let format_watch = format.clone();
    watch!(sender, [format_watch.watch()], |out| {
        let label = helpers::format_label(&format_watch.get(), &UpdateCounts::default());
        let _ = out.send(UpdatesCmd::UpdateLabel(label));
    });

    // Watch icon changes
    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(UpdatesCmd::UpdateIcon(icon_name.get().clone()));
    });

    // Watch threshold changes
    let thresholds_watch = thresholds.clone();
    watch!(sender, [thresholds_watch.watch()], |out| {
        let colors = evaluate_thresholds(0.0, &thresholds_watch.get());
        let _ = out.send(UpdatesCmd::UpdateThresholdColors(colors));
    });
}
