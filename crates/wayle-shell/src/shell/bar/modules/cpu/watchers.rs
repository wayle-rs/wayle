use std::{sync::Arc, time::Duration};

use relm4::ComponentSender;
use wayle_common::watch;
use wayle_config::schemas::{modules::CpuConfig, styling::evaluate_thresholds};
use wayle_sysinfo::SysinfoService;

use super::{CpuModule, helpers::format_label, messages::CpuCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<CpuModule>,
    config: &CpuConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let format = config.format.clone();
    let thresholds = config.thresholds.clone();

    let sysinfo_cpu = sysinfo.clone();
    watch!(sender, [sysinfo.cpu.watch()], |out| {
        let cpu = sysinfo_cpu.cpu.get();
        let label = format_label(&format.get(), &cpu);
        let _ = out.send(CpuCmd::UpdateLabel(label));

        let colors = evaluate_thresholds(cpu.usage_percent as f64, &thresholds.get());
        let _ = out.send(CpuCmd::UpdateThresholdColors(colors));
    });

    let format_watch = config.format.clone();
    let sysinfo_format = sysinfo.clone();
    watch!(sender, [format_watch.watch()], |out| {
        let cpu = sysinfo_format.cpu.get();
        let label = format_label(&format_watch.get(), &cpu);
        let _ = out.send(CpuCmd::UpdateLabel(label));
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(CpuCmd::UpdateIcon(icon_name.get().clone()));
    });

    let temp_sensor = config.temp_sensor.clone();
    let sysinfo_sensor = sysinfo.clone();
    watch!(sender, [temp_sensor.watch()], |_out| {
        sysinfo_sensor.set_cpu_temp_sensor(&temp_sensor.get());
    });

    let poll_interval = config.poll_interval_ms.clone();
    let sysinfo_interval = sysinfo.clone();
    watch!(sender, [poll_interval.watch()], |_out| {
        sysinfo_interval.set_cpu_interval(Duration::from_millis(poll_interval.get()));
    });
}
