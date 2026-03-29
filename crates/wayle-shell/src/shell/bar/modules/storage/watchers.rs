use std::{path::Path, sync::Arc};

use relm4::ComponentSender;
use wayle_config::schemas::{modules::StorageConfig, styling::evaluate_thresholds};
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch;

use super::{StorageModule, helpers::format_label, messages::StorageCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<StorageModule>,
    config: &StorageConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let format = config.format.clone();
    let mount_point = config.mount_point.clone();
    let thresholds = config.thresholds.clone();

    let sysinfo_disks = sysinfo.clone();
    let sysinfo_format = sysinfo.clone();

    let thresholds_watch = thresholds.clone();
    let sysinfo_thresholds = sysinfo.clone();
    let mount_point_thresholds = mount_point.clone();
    watch!(sender, [thresholds_watch.watch()], |out| {
        let disks = sysinfo_thresholds.disks.get();
        let target = mount_point_thresholds.get();
        let target_path = Path::new(&target);

        if let Some(disk) = disks.iter().find(|d| d.mount_point == target_path) {
            let colors = evaluate_thresholds(disk.usage_percent as f64, &thresholds_watch.get());
            let _ = out.send(StorageCmd::UpdateThresholdColors(colors));
        }
    });

    watch!(sender, [sysinfo.disks.watch()], |out| {
        let disks = sysinfo_disks.disks.get();
        let target = mount_point.get();
        let target_path = Path::new(&target);

        if let Some(disk) = disks.iter().find(|d| d.mount_point == target_path) {
            let label = format_label(&format.get(), disk);
            let _ = out.send(StorageCmd::UpdateLabel(label));

            let colors = evaluate_thresholds(disk.usage_percent as f64, &thresholds.get());
            let _ = out.send(StorageCmd::UpdateThresholdColors(colors));
        }
    });

    let format_watch = config.format.clone();
    let mount_point_format = config.mount_point.clone();
    watch!(sender, [format_watch.watch()], |out| {
        let disks = sysinfo_format.disks.get();
        let target = mount_point_format.get();
        let target_path = Path::new(&target);

        if let Some(disk) = disks.iter().find(|d| d.mount_point == target_path) {
            let label = format_label(&format_watch.get(), disk);
            let _ = out.send(StorageCmd::UpdateLabel(label));
        }
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(StorageCmd::UpdateIcon(icon_name.get().clone()));
    });
}
