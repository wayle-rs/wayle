use nvml_wrapper::Nvml;
use serde_json::json;
use tracing::warn;

/// Raw GPU data from sysfs or NVML.
#[derive(Debug, Clone, Default)]
pub(super) struct GpuData {
    pub usage_percent: f32,
    pub temperature_celsius: Option<f32>,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub name: String,
}

impl GpuData {
    pub fn vram_percent(&self) -> f32 {
        if self.vram_total_mb == 0 {
            return 0.0;
        }
        (self.vram_used_mb as f32 / self.vram_total_mb as f32) * 100.0
    }
}

/// Formats a GPU label using Jinja2 template syntax.
///
/// ## Variables
///
/// - `{{ percent }}` - GPU usage (00-100, zero-padded)
/// - `{{ temp_c }}` - Temperature in Celsius (zero-padded)
/// - `{{ temp_f }}` - Temperature in Fahrenheit (zero-padded)
/// - `{{ vram_used_mb }}` - VRAM used in MB
/// - `{{ vram_total_mb }}` - VRAM total in MB
/// - `{{ vram_percent }}` - VRAM usage percentage
/// - `{{ name }}` - GPU model name
pub(super) fn format_label(format: &str, gpu: &GpuData) -> String {
    let temp_c = gpu.temperature_celsius.unwrap_or(0.0);
    let temp_f = temp_c * 9.0 / 5.0 + 32.0;

    let ctx = json!({
        "percent": format!("{:02.0}", gpu.usage_percent),
        "temp_c": format!("{temp_c:02.0}"),
        "temp_f": format!("{temp_f:02.0}"),
        "vram_used_mb": gpu.vram_used_mb,
        "vram_total_mb": gpu.vram_total_mb,
        "vram_percent": format!("{:.0}", gpu.vram_percent()),
        "name": &gpu.name,
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

/// Detected GPU vendor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GpuVendor {
    Nvidia,
    Amd,
    Unknown,
}

/// Attempt to initialise the NVML library.
///
/// Returns `Some(Nvml)` when the NVIDIA driver is installed and
/// `libnvidia-ml.so` can be loaded.  Returns `None` otherwise.
pub(super) fn init_nvml() -> Option<Nvml> {
    match Nvml::init() {
        Ok(nvml) => Some(nvml),
        Err(e) => {
            tracing::debug!("NVML init failed (expected on non-NVIDIA systems): {e}");
            None
        }
    }
}

/// Auto-detect GPU vendor.
///
/// When `vendor_config` is `"auto"`, the presence of a successfully
/// initialised NVML handle indicates NVIDIA; otherwise AMD sysfs is probed.
pub(super) fn detect_vendor(vendor_config: &str, nvml: &Option<Nvml>) -> GpuVendor {
    match vendor_config {
        "nvidia" => GpuVendor::Nvidia,
        "amd" => GpuVendor::Amd,
        "auto" | _ => {
            // NVML was already initialised — NVIDIA driver is present
            if nvml.is_some() {
                return GpuVendor::Nvidia;
            }

            // Check AMD (sysfs gpu_busy_percent exists)
            for entry in std::fs::read_dir("/sys/class/drm/").into_iter().flatten() {
                let Ok(entry) = entry else { continue };
                let path = entry.path().join("device/gpu_busy_percent");
                if path.exists() {
                    return GpuVendor::Amd;
                }
            }

            GpuVendor::Unknown
        }
    }
}

/// Read GPU data from NVIDIA via NVML.
pub(super) fn read_nvidia(nvml: &Nvml, device_index: u32) -> GpuData {
    let device = match nvml.device_by_index(device_index) {
        Ok(d) => d,
        Err(e) => {
            warn!("NVML: failed to get device {device_index}: {e}");
            return GpuData::default();
        }
    };

    let usage_percent = device
        .utilization_rates()
        .map(|u| u.gpu as f32)
        .unwrap_or(0.0);

    let temperature_celsius = device
        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
        .ok()
        .map(|t| t as f32);

    let (vram_used_mb, vram_total_mb) = device
        .memory_info()
        .map(|m| (m.used / (1024 * 1024), m.total / (1024 * 1024)))
        .unwrap_or((0, 0));

    let name = device.name().unwrap_or_default();

    GpuData {
        usage_percent,
        temperature_celsius,
        vram_used_mb,
        vram_total_mb,
        name,
    }
}

/// Read GPU data from AMD sysfs/hwmon.
pub(super) fn read_amd(card_index: u32) -> GpuData {
    let card = format!("/sys/class/drm/card{card_index}/device");

    let usage = read_sysfs_u64(&format!("{card}/gpu_busy_percent")).unwrap_or(0);
    let vram_used = read_sysfs_u64(&format!("{card}/mem_info_vram_used")).unwrap_or(0);
    let vram_total = read_sysfs_u64(&format!("{card}/mem_info_vram_total")).unwrap_or(0);

    // Temperature from hwmon (millidegrees Celsius)
    let temp = find_hwmon_temp(&card);

    // GPU name from product info or marketing name
    let name = read_sysfs_string(&format!("{card}/product_name"))
        .or_else(|| read_sysfs_string(&format!("{card}/marketing_name")))
        .unwrap_or_default();

    GpuData {
        usage_percent: usage as f32,
        temperature_celsius: temp,
        vram_used_mb: vram_used / (1024 * 1024),
        vram_total_mb: vram_total / (1024 * 1024),
        name,
    }
}

fn find_hwmon_temp(card_device_path: &str) -> Option<f32> {
    let hwmon_dir = format!("{card_device_path}/hwmon");
    let entries = std::fs::read_dir(&hwmon_dir).ok()?;

    for entry in entries.flatten() {
        let temp_path = entry.path().join("temp1_input");
        if let Some(millideg) = read_sysfs_u64(&temp_path.to_string_lossy()) {
            return Some(millideg as f32 / 1000.0);
        }
    }
    None
}

fn read_sysfs_u64(path: &str) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn read_sysfs_string(path: &str) -> Option<String> {
    let s = std::fs::read_to_string(path).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gpu_data(usage: f32, temp: Option<f32>, vram_used: u64, vram_total: u64) -> GpuData {
        GpuData {
            usage_percent: usage,
            temperature_celsius: temp,
            vram_used_mb: vram_used,
            vram_total_mb: vram_total,
            name: String::from("Test GPU"),
        }
    }

    #[test]
    fn format_label_replaces_percent() {
        let gpu = gpu_data(45.7, Some(55.0), 2048, 8192);
        let result = format_label("{{ percent }}%", &gpu);
        assert_eq!(result, "46%");
    }

    #[test]
    fn format_label_percent_pads_single_digits() {
        let gpu = gpu_data(5.2, Some(55.0), 2048, 8192);
        let result = format_label("{{ percent }}", &gpu);
        assert_eq!(result, "05");
    }

    #[test]
    fn format_label_replaces_temp_c() {
        let gpu = gpu_data(50.0, Some(72.5), 2048, 8192);
        let result = format_label("{{ temp_c }}°C", &gpu);
        assert_eq!(result, "72°C");
    }

    #[test]
    fn format_label_replaces_vram() {
        let gpu = gpu_data(50.0, Some(55.0), 2048, 8192);
        let result = format_label("{{ vram_used_mb }}/{{ vram_total_mb }}MB", &gpu);
        assert_eq!(result, "2048/8192MB");
    }

    #[test]
    fn format_label_replaces_vram_percent() {
        let gpu = gpu_data(50.0, Some(55.0), 4096, 8192);
        let result = format_label("{{ vram_percent }}%", &gpu);
        assert_eq!(result, "50%");
    }

    #[test]
    fn format_label_replaces_name() {
        let gpu = gpu_data(50.0, Some(55.0), 2048, 8192);
        let result = format_label("{{ name }}", &gpu);
        assert_eq!(result, "Test GPU");
    }

    #[test]
    fn format_label_with_no_temp_uses_zero() {
        let gpu = gpu_data(50.0, None, 2048, 8192);
        let result = format_label("{{ temp_c }}°C", &gpu);
        assert_eq!(result, "00°C");
    }

    #[test]
    fn format_label_multiple_placeholders() {
        let gpu = gpu_data(75.0, Some(65.0), 4096, 8192);
        let result = format_label("{{ percent }}% {{ temp_c }}°C {{ vram_percent }}%V", &gpu);
        assert_eq!(result, "75% 65°C 50%V");
    }

    #[test]
    fn vram_percent_zero_total() {
        let gpu = gpu_data(0.0, None, 0, 0);
        assert_eq!(gpu.vram_percent(), 0.0);
    }
}
