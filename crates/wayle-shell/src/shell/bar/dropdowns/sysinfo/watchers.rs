use std::{sync::Arc, time::Duration};

use nvml_wrapper::Nvml;
use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch;

use super::{
    SysinfoDropdown,
    helpers::{self, CpuSummary, MemoryInfo},
    messages::SysinfoDropdownCmd,
};

const GPU_POLL_MS: u64 = 3000;

pub(super) fn spawn(
    sender: &ComponentSender<SysinfoDropdown>,
    sysinfo: &Arc<SysinfoService>,
    config: &Arc<ConfigService>,
    nvml: Option<Arc<Nvml>>,
    amd_card: Option<u32>,
    cpu_name: String,
) {
    spawn_scale_watcher(sender, config);
    spawn_gpu_poller(sender, nvml, amd_card);
    spawn_cpu_watcher(sender, sysinfo, cpu_name);
    spawn_memory_watcher(sender, sysinfo);
}

fn spawn_scale_watcher(
    sender: &ComponentSender<SysinfoDropdown>,
    config: &Arc<ConfigService>,
) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(SysinfoDropdownCmd::ScaleChanged(scale.get().value()));
    });
}

fn spawn_gpu_poller(
    sender: &ComponentSender<SysinfoDropdown>,
    nvml: Option<Arc<Nvml>>,
    amd_card: Option<u32>,
) {
    let poll_interval = Duration::from_millis(GPU_POLL_MS);

    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                loop {
                    if let Some(ref nvml) = nvml {
                        let data = helpers::read_nvidia(nvml, 0);
                        let _ = out.send(SysinfoDropdownCmd::UpdateNvidia(data));
                    }

                    if let Some(card) = amd_card {
                        let data = helpers::read_amd(card);
                        let _ = out.send(SysinfoDropdownCmd::UpdateAmd(data));
                    }

                    tokio::time::sleep(poll_interval).await;
                }
            })
            .drop_on_shutdown()
    });
}

fn spawn_cpu_watcher(
    sender: &ComponentSender<SysinfoDropdown>,
    sysinfo: &Arc<SysinfoService>,
    cpu_name: String,
) {
    let cpu_prop = sysinfo.cpu.clone();

    watch!(sender, [cpu_prop.watch()], |out| {
        let cpu = cpu_prop.get();
        let summary = CpuSummary {
            name: cpu_name.clone(),
            usage_percent: cpu.usage_percent,
            temperature_celsius: cpu.temperature_celsius,
            avg_freq_ghz: cpu.avg_frequency_mhz as f64 / 1000.0,
            max_freq_ghz: cpu.max_frequency_mhz as f64 / 1000.0,
            core_count: cpu.cores.len(),
        };
        let _ = out.send(SysinfoDropdownCmd::UpdateCpu(summary));
    });
}

fn spawn_memory_watcher(
    sender: &ComponentSender<SysinfoDropdown>,
    sysinfo: &Arc<SysinfoService>,
) {
    let mem_prop = sysinfo.memory.clone();

    watch!(sender, [mem_prop.watch()], |out| {
        let mem = mem_prop.get();
        let to_gb = |bytes: u64| bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        let info = MemoryInfo {
            ram_used_gb: to_gb(mem.used_bytes),
            ram_total_gb: to_gb(mem.total_bytes),
            ram_percent: mem.usage_percent,
            swap_used_gb: to_gb(mem.swap_used_bytes),
            swap_total_gb: to_gb(mem.swap_total_bytes),
            swap_percent: mem.swap_percent,
        };
        let _ = out.send(SysinfoDropdownCmd::UpdateMemory(info));
    });
}
