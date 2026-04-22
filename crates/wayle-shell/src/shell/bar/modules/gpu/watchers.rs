use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::schemas::{modules::GpuConfig, styling::evaluate_thresholds};
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch;

use super::{GpuModule, helpers::format_label, messages::GpuCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<GpuModule>,
    config: &GpuConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let format = config.format.clone();
    let thresholds = config.thresholds.clone();

    let sysinfo_gpu = sysinfo.clone();

    let thresholds_watch = thresholds.clone();
    let sysinfo_thresholds = sysinfo.clone();
    watch!(sender, [thresholds_watch.watch()], |out| {
        let gpu = sysinfo_thresholds.gpu.get();
        let colors = evaluate_thresholds(
            gpu.average_utilization_percent as f64,
            &thresholds_watch.get(),
        );
        let _ = out.send(GpuCmd::UpdateThresholdColors(colors));
    });

    watch!(sender, [sysinfo.gpu.watch()], |out| {
        let gpu = sysinfo_gpu.gpu.get();
        let label = format_label(&format.get(), &gpu);
        let _ = out.send(GpuCmd::UpdateLabel(label));

        let colors = evaluate_thresholds(gpu.average_utilization_percent as f64, &thresholds.get());
        let _ = out.send(GpuCmd::UpdateThresholdColors(colors));
    });

    let format_watch = config.format.clone();
    let sysinfo_format = sysinfo.clone();
    watch!(sender, [format_watch.watch()], |out| {
        let gpu = sysinfo_format.gpu.get();
        let label = format_label(&format_watch.get(), &gpu);
        let _ = out.send(GpuCmd::UpdateLabel(label));
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(GpuCmd::UpdateIcon(icon_name.get().clone()));
    });
}
