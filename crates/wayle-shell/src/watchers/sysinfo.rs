//! Sysinfo polling interval hot-reload watcher.

use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use wayle_sysinfo::SysinfoService;

use crate::shell::ShellServices;

/// Spawns watchers for sysinfo polling interval configuration.
///
/// Updates the service's polling intervals when config properties change.
pub fn spawn(services: &ShellServices) {
    let config = services.config.config().clone();
    let sysinfo = &services.sysinfo;
    let modules = &config.modules;

    spawn_cpu_watcher(&modules.cpu, sysinfo);
    spawn_memory_watcher(&modules.ram, sysinfo);
    spawn_disk_watcher(&modules.storage, sysinfo);
    spawn_network_watcher(&modules.netstat, sysinfo);
    spawn_gpu_watcher(&modules.gpu, sysinfo);
}

fn spawn_cpu_watcher(
    config: &wayle_config::schemas::modules::CpuConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let mut stream = config.poll_interval_ms.watch();
    let sysinfo = sysinfo.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_ms) = stream.next().await {
            sysinfo.set_cpu_interval(Duration::from_millis(interval_ms));
        }
    });
}

fn spawn_memory_watcher(
    config: &wayle_config::schemas::modules::RamConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let mut stream = config.poll_interval_ms.watch();
    let sysinfo = sysinfo.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_ms) = stream.next().await {
            sysinfo.set_memory_interval(Duration::from_millis(interval_ms));
        }
    });
}

fn spawn_disk_watcher(
    config: &wayle_config::schemas::modules::StorageConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let mut stream = config.poll_interval_ms.watch();
    let sysinfo = sysinfo.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_ms) = stream.next().await {
            sysinfo.set_disk_interval(Duration::from_millis(interval_ms));
        }
    });
}

fn spawn_network_watcher(
    config: &wayle_config::schemas::modules::NetstatConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let mut stream = config.poll_interval_ms.watch();
    let sysinfo = sysinfo.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_ms) = stream.next().await {
            sysinfo.set_network_interval(Duration::from_millis(interval_ms));
        }
    });
}

fn spawn_gpu_watcher(
    config: &wayle_config::schemas::modules::GpuConfig,
    sysinfo: &Arc<SysinfoService>,
) {
    let mut stream = config.poll_interval_ms.watch();
    let sysinfo = sysinfo.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(interval_ms) = stream.next().await {
            sysinfo.set_gpu_interval(Duration::from_millis(interval_ms));
        }
    });
}
