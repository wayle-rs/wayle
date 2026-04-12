use bytesize::ByteSize;
use serde_json::json;
use wayle_sysinfo::types::{GpuData, GpuDeviceData};

/// Formats a GPU label using Jinja2 template syntax.
///
/// ## Aggregate Variables
///
/// - `{{ count }}` - Number of detected GPUs
/// - `{{ active_count }}` - Number of GPUs reporting utilization
/// - `{{ percent }}` - Average GPU core utilization (00-100, zero-padded)
/// - `{{ mem_percent }}` - Average GPU memory utilization (00-100, zero-padded)
/// - `{{ temp_c }}` - Maximum GPU temperature across devices (zero-padded)
///
/// ## Per-device Variables (first two GPUs)
///
/// - `{{ gpu0_percent }}`, `{{ gpu1_percent }}`
/// - `{{ gpu0_mem_percent }}`, `{{ gpu1_mem_percent }}`
/// - `{{ gpu0_temp_c }}`, `{{ gpu1_temp_c }}`
/// - `{{ gpu0_mem_used_gib }}`, `{{ gpu1_mem_used_gib }}`
/// - `{{ gpu0_mem_total_gib }}`, `{{ gpu1_mem_total_gib }}`
pub(super) fn format_label(format: &str, gpu: &GpuData) -> String {
    let gpu0 = gpu.devices.iter().find(|device| device.index == 0);
    let gpu1 = gpu.devices.iter().find(|device| device.index == 1);

    let max_temp_c = gpu
        .devices
        .iter()
        .filter_map(|d| d.temperature_celsius)
        .fold(0.0_f32, f32::max);

    let ctx = json!({
        "count": gpu.total_count,
        "active_count": gpu.active_count,
        "percent": format!("{:02.0}", gpu.average_utilization_percent),
        "mem_percent": format!("{:02.0}", gpu.average_memory_utilization_percent),
        "temp_c": format!("{max_temp_c:02.0}"),

        "gpu0_percent": format_percent(gpu0.and_then(|d| d.utilization_percent)),
        "gpu1_percent": format_percent(gpu1.and_then(|d| d.utilization_percent)),

        "gpu0_mem_percent": format_percent(gpu0.and_then(|d| d.memory_utilization_percent)),
        "gpu1_mem_percent": format_percent(gpu1.and_then(|d| d.memory_utilization_percent)),

        "gpu0_temp_c": format_percent(gpu0.and_then(|d| d.temperature_celsius)),
        "gpu1_temp_c": format_percent(gpu1.and_then(|d| d.temperature_celsius)),

        "gpu0_mem_used_gib": gib(gpu0.and_then(|d| d.memory_used_bytes)),
        "gpu1_mem_used_gib": gib(gpu1.and_then(|d| d.memory_used_bytes)),
        "gpu0_mem_total_gib": gib(gpu0.and_then(|d| d.memory_total_bytes)),
        "gpu1_mem_total_gib": gib(gpu1.and_then(|d| d.memory_total_bytes)),
    });

    crate::template::render(format, ctx).unwrap_or_default()
}

fn format_percent(value: Option<f32>) -> String {
    format!("{:02.0}", value.unwrap_or(0.0))
}

fn gib(bytes: Option<u64>) -> String {
    format!("{:.1}", ByteSize::b(bytes.unwrap_or(0)).as_gib())
}

#[cfg(test)]
mod tests {
    use super::*;
    const GIB: u64 = 1024 * 1024 * 1024;

    fn device(
        index: u32,
        util: Option<f32>,
        mem_used: u64,
        mem_total: u64,
        temp: Option<f32>,
    ) -> GpuDeviceData {
        GpuDeviceData {
            index,
            name: format!("GPU {index}"),
            uuid: format!("uuid-{index}"),
            utilization_percent: util,
            memory_used_bytes: Some(mem_used),
            memory_total_bytes: Some(mem_total),
            memory_utilization_percent: if mem_total > 0 {
                Some((mem_used as f32 / mem_total as f32) * 100.0)
            } else {
                Some(0.0)
            },
            temperature_celsius: temp,
            power_watts: None,
            power_limit_watts: None,
            fan_speed_percent: None,
            graphics_clock_mhz: None,
            memory_clock_mhz: None,
        }
    }

    fn gpu_data(devices: Vec<GpuDeviceData>, avg_util: f32, avg_mem_util: f32) -> GpuData {
        let active_count = devices
            .iter()
            .filter(|d| d.utilization_percent.is_some())
            .count();
        let total_count = devices
            .iter()
            .map(|d| d.index as usize)
            .max()
            .map(|max_index| max_index + 1)
            .unwrap_or(0);
        GpuData {
            total_count,
            active_count,
            average_utilization_percent: avg_util,
            average_memory_utilization_percent: avg_mem_util,
            devices,
        }
    }

    #[test]
    fn format_label_replaces_aggregate_placeholders() {
        let gpu = gpu_data(vec![], 37.2, 42.1);
        let out = format_label("{{ percent }}% VRAM {{ mem_percent }}% ({{ count }})", &gpu);
        assert_eq!(out, "37% VRAM 42% (0)");
    }

    #[test]
    fn format_label_uses_max_temperature() {
        let gpu = gpu_data(
            vec![
                device(0, Some(10.0), 2 * GIB, 8 * GIB, Some(61.0)),
                device(1, Some(20.0), 1 * GIB, 8 * GIB, Some(73.0)),
            ],
            15.0,
            19.0,
        );
        let out = format_label("{{ temp_c }}C", &gpu);
        assert_eq!(out, "73C");
    }

    #[test]
    fn format_label_replaces_per_gpu_placeholders() {
        let gpu = gpu_data(
            vec![
                device(0, Some(52.0), 3 * GIB, 8 * GIB, Some(65.0)),
                device(1, Some(11.0), 1 * GIB, 8 * GIB, Some(49.0)),
            ],
            31.5,
            25.0,
        );
        let out = format_label("{{ gpu0_percent }}% | {{ gpu1_percent }}%", &gpu);
        assert_eq!(out, "52% | 11%");
    }

    #[test]
    fn format_label_missing_second_gpu_defaults_to_zero() {
        let gpu = gpu_data(
            vec![device(0, Some(40.0), 2 * GIB, 8 * GIB, Some(55.0))],
            40.0,
            25.0,
        );
        let out = format_label("{{ gpu1_percent }} {{ gpu1_mem_total_gib }}", &gpu);
        assert_eq!(out, "00 0.0");
    }

    #[test]
    fn format_label_uses_device_index_not_vector_position() {
        let gpu = gpu_data(vec![device(1, Some(11.0), 1 * GIB, 8 * GIB, Some(49.0))], 11.0, 12.5);
        let out = format_label("{{ gpu0_percent }}% | {{ gpu1_percent }}%", &gpu);
        assert_eq!(out, "00% | 11%");
    }

    #[test]
    fn format_label_formats_memory_gib() {
        let gpu = gpu_data(
            vec![device(
                0,
                Some(40.0),
                (1.5 * GIB as f64) as u64,
                12 * GIB,
                Some(55.0),
            )],
            40.0,
            12.5,
        );
        let out = format_label("{{ gpu0_mem_used_gib }}/{{ gpu0_mem_total_gib }}", &gpu);
        assert_eq!(out, "1.5/12.0");
    }
}
