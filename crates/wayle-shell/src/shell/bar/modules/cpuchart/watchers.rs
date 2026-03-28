use std::sync::{Arc, atomic::AtomicUsize, atomic::Ordering};

use relm4::ComponentSender;
use wayle_config::schemas::modules::CpuChartConfig;
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch;

use super::{CpuChartModule, messages::CpuChartCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<CpuChartModule>,
    config: &CpuChartConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let sysinfo_cpu = sysinfo.clone();
    let last_num_cores = Arc::new(AtomicUsize::new(0));

    watch!(sender, [sysinfo.cpu.watch()], |out| {
        let cpu = sysinfo_cpu.cpu.get();
        let core_values: Vec<f64> = cpu
            .cores
            .iter()
            .map(|core| (core.usage_percent as f64) / 100.0)
            .collect();

        let num_cores = core_values.len();
        let prev_cores = last_num_cores.swap(num_cores, Ordering::Relaxed);
        if num_cores != prev_cores {
            let _ = out.send(CpuChartCmd::Resize);
        }

        let _ = out.send(CpuChartCmd::Update(core_values));
    });

    // Watch size-affecting configs and trigger resize
    let bar_width = config.bar_width.clone();
    let bar_gap = config.bar_gap.clone();
    let padding = config.internal_padding.clone();

    watch!(
        sender,
        [bar_width.watch(), bar_gap.watch(), padding.watch()],
        |out| {
            let _ = out.send(CpuChartCmd::Resize);
        }
    );

    // Watch visual-only configs and trigger redraw
    let direction = config.direction.clone();
    let color = config.color.clone();
    let sysinfo_visual = sysinfo.clone();

    watch!(sender, [direction.watch(), color.watch()], |out| {
        let cpu = sysinfo_visual.cpu.get();
        let core_values: Vec<f64> = cpu
            .cores
            .iter()
            .map(|core| (core.usage_percent as f64) / 100.0)
            .collect();
        let _ = out.send(CpuChartCmd::Update(core_values));
    });
}
