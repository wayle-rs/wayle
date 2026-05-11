use std::{sync::Arc, time::Duration};

use nvml_wrapper::Nvml;
use relm4::ComponentSender;
use wayle_config::schemas::{modules::GpuConfig, styling::evaluate_thresholds};
use wayle_sysinfo::SysinfoService;
use wayle_widgets::watch;

use super::{
    GpuModule,
    helpers::{self, GpuData, GpuVendor, format_label},
    messages::GpuCmd,
};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<GpuModule>,
    config: &GpuConfig,
    vendor: GpuVendor,
    card_index: u32,
    nvml: Option<Arc<Nvml>>,
    sysinfo: Arc<SysinfoService>,
) {
    let format = config.format.clone();
    let thresholds = config.thresholds.clone();
    let poll_interval = config.poll_interval_ms.clone();

    // Poll GPU data on interval
    let format_poll = format.clone();
    let thresholds_poll = thresholds.clone();
    let poll_ms = poll_interval.clone();
    let nvml_poll = nvml.clone();
    let sysinfo_poll = sysinfo.clone();
    sender.command(move |out, shutdown| {
        shutdown
            .register(async move {
                loop {
                    let gpu = poll_gpu(vendor, card_index, &nvml_poll);
                    let mem = sysinfo_poll.memory.get();
                    let label = format_label(&format_poll.get(), &gpu, Some(&mem));
                    let _ = out.send(GpuCmd::UpdateLabel(label));

                    let colors =
                        evaluate_thresholds(gpu.usage_percent as f64, &thresholds_poll.get());
                    let _ = out.send(GpuCmd::UpdateThresholdColors(colors));

                    let interval = Duration::from_millis(poll_ms.get());
                    tokio::time::sleep(interval).await;
                }
            })
            .drop_on_shutdown()
    });

    // Watch format changes
    let format_watch = format.clone();
    let nvml_fmt = nvml.clone();
    let sysinfo_fmt = sysinfo.clone();
    watch!(sender, [format_watch.watch()], |out| {
        let gpu = poll_gpu(vendor, card_index, &nvml_fmt);
        let mem = sysinfo_fmt.memory.get();
        let label = format_label(&format_watch.get(), &gpu, Some(&mem));
        let _ = out.send(GpuCmd::UpdateLabel(label));
    });

    // Watch icon changes
    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(GpuCmd::UpdateIcon(icon_name.get().clone()));
    });

    // Watch threshold changes
    let thresholds_watch = thresholds.clone();
    let nvml_thr = nvml.clone();
    watch!(sender, [thresholds_watch.watch()], |out| {
        let gpu = poll_gpu(vendor, card_index, &nvml_thr);
        let colors = evaluate_thresholds(gpu.usage_percent as f64, &thresholds_watch.get());
        let _ = out.send(GpuCmd::UpdateThresholdColors(colors));
    });

    // Watch memory changes — re-render label when RAM data updates
    let format_mem = format.clone();
    let nvml_mem = nvml.clone();
    let sysinfo_mem = sysinfo.clone();
    watch!(sender, [sysinfo_mem.memory.watch()], |out| {
        let gpu = poll_gpu(vendor, card_index, &nvml_mem);
        let mem = sysinfo_mem.memory.get();
        let label = format_label(&format_mem.get(), &gpu, Some(&mem));
        let _ = out.send(GpuCmd::UpdateLabel(label));
    });
}

fn poll_gpu(vendor: GpuVendor, card_index: u32, nvml: &Option<Arc<Nvml>>) -> GpuData {
    match vendor {
        GpuVendor::Nvidia => nvml
            .as_ref()
            .map(|n| helpers::read_nvidia(n, card_index))
            .unwrap_or_default(),
        GpuVendor::Amd => helpers::read_amd(card_index),
        GpuVendor::Unknown => GpuData::default(),
    }
}
