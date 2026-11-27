use nvml_wrapper::Nvml;
use tracing::warn;

// ── NVIDIA ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub(in crate::shell::bar::dropdowns) struct NvidiaData {
    pub name: String,
    pub usage_percent: f32,
    pub temperature_celsius: Option<f32>,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub gpu_clock_mhz: Option<u32>,
    pub mem_clock_mhz: Option<u32>,
    pub fan_speed_percent: Option<u32>,
    pub power_watts: Option<f32>,
    pub power_limit_watts: Option<f32>,
}

pub(super) fn init_nvml() -> Option<Nvml> {
    match Nvml::init() {
        Ok(nvml) => Some(nvml),
        Err(e) => {
            tracing::debug!("NVML init failed: {e}");
            None
        }
    }
}

pub(super) fn read_nvidia(nvml: &Nvml, device_index: u32) -> NvidiaData {
    let device = match nvml.device_by_index(device_index) {
        Ok(d) => d,
        Err(e) => {
            warn!("NVML device {device_index}: {e}");
            return NvidiaData::default();
        }
    };

    use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};

    NvidiaData {
        name: device.name().unwrap_or_default(),
        usage_percent: device
            .utilization_rates()
            .map(|u| u.gpu as f32)
            .unwrap_or(0.0),
        temperature_celsius: device
            .temperature(TemperatureSensor::Gpu)
            .ok()
            .map(|t| t as f32),
        vram_used_mb: device
            .memory_info()
            .map(|m| m.used / (1024 * 1024))
            .unwrap_or(0),
        vram_total_mb: device
            .memory_info()
            .map(|m| m.total / (1024 * 1024))
            .unwrap_or(0),
        gpu_clock_mhz: device.clock_info(Clock::Graphics).ok(),
        mem_clock_mhz: device.clock_info(Clock::Memory).ok(),
        fan_speed_percent: device.fan_speed(0).ok(),
        power_watts: device.power_usage().ok().map(|mw| mw as f32 / 1000.0),
        power_limit_watts: device
            .power_management_limit()
            .ok()
            .map(|mw| mw as f32 / 1000.0),
    }
}

// ── AMD (sysfs) ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub(in crate::shell::bar::dropdowns) struct AmdData {
    pub name: String,
    pub usage_percent: f32,
    pub temperature_celsius: Option<f32>,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
}

pub(super) fn detect_amd_card() -> Option<u32> {
    for entry in std::fs::read_dir("/sys/class/drm/").ok()?.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") || name.contains('-') {
            continue;
        }
        let path = entry.path().join("device/gpu_busy_percent");
        if path.exists() {
            return name.strip_prefix("card")?.parse().ok();
        }
    }
    None
}

pub(super) fn read_amd(card_index: u32) -> AmdData {
    let card = format!("/sys/class/drm/card{card_index}/device");

    let usage = read_sysfs_u64(&format!("{card}/gpu_busy_percent")).unwrap_or(0);
    let vram_used = read_sysfs_u64(&format!("{card}/mem_info_vram_used")).unwrap_or(0);
    let vram_total = read_sysfs_u64(&format!("{card}/mem_info_vram_total")).unwrap_or(0);
    let temp = find_hwmon_temp(&card);

    let name = read_sysfs_string(&format!("{card}/product_name"))
        .or_else(|| read_sysfs_string(&format!("{card}/marketing_name")))
        .unwrap_or_else(|| format!("AMD GPU (card{card_index})"));

    AmdData {
        name,
        usage_percent: usage as f32,
        temperature_celsius: temp,
        vram_used_mb: vram_used / (1024 * 1024),
        vram_total_mb: vram_total / (1024 * 1024),
    }
}

// ── CPU ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub(in crate::shell::bar::dropdowns) struct CpuSummary {
    pub name: String,
    pub usage_percent: f32,
    pub temperature_celsius: Option<f32>,
    pub avg_freq_ghz: f64,
    pub max_freq_ghz: f64,
    pub core_count: usize,
}

/// Read CPU model name from /proc/cpuinfo.
pub(super) fn read_cpu_name() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| String::from("CPU"))
}

// ── Memory ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub(in crate::shell::bar::dropdowns) struct MemoryInfo {
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub ram_percent: f32,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
    pub swap_percent: f32,
}

// ── sysfs helpers ───────────────────────────────────────────────────────

fn find_hwmon_temp(card_device_path: &str) -> Option<f32> {
    let hwmon_dir = format!("{card_device_path}/hwmon");
    for entry in std::fs::read_dir(&hwmon_dir).ok()?.flatten() {
        let temp_path = entry.path().join("temp1_input");
        if let Some(millideg) = read_sysfs_u64(&temp_path.to_string_lossy()) {
            return Some(millideg as f32 / 1000.0);
        }
    }
    None
}

fn read_sysfs_u64(path: &str) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn read_sysfs_string(path: &str) -> Option<String> {
    let s = std::fs::read_to_string(path).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

// ── Formatting helpers ──────────────────────────────────────────────────

pub(super) fn fmt_temp(t: Option<f32>) -> String {
    t.map(|v| format!("{v:.0}°C")).unwrap_or_else(|| "—".into())
}

pub(super) fn fmt_vram(used: u64, total: u64) -> String {
    if total == 0 {
        return "—".into();
    }
    let pct = (used as f64 / total as f64) * 100.0;
    format!("{used} / {total} MB ({pct:.0}%)")
}

pub(super) fn fmt_clock(mhz: Option<u32>) -> String {
    mhz.map(|v| format!("{v} MHz")).unwrap_or_else(|| "—".into())
}

pub(super) fn fmt_fan(pct: Option<u32>) -> String {
    match pct {
        Some(0) => "Off".into(),
        Some(v) => format!("{v}%"),
        None => "—".into(),
    }
}

pub(super) fn fmt_power(watts: Option<f32>, limit: Option<f32>) -> String {
    match (watts, limit) {
        (Some(w), Some(l)) => format!("{w:.0} / {l:.0} W"),
        (Some(w), None) => format!("{w:.0} W"),
        (None, Some(l)) => format!("{l:.0} W (limit)"),
        (None, None) => "—".into(),
    }
}

pub(super) fn fmt_gb(gb: f64, total_gb: f64, pct: f32) -> String {
    format!("{gb:.1} / {total_gb:.1} GB ({pct:.0}%)")
}
