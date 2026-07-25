use bytesize::ByteSize;
use serde_json::json;
use wayle_sysinfo::types::MemoryData;

pub(super) fn format_label(format: &str, mem: &MemoryData) -> String {
    let ctx = json!({
        "percent": format!("{:.0}", mem.usage_percent),
        "used_gib": gib(mem.used_bytes),
        "total_gib": gib(mem.total_bytes),
        "available_gib": gib(mem.available_bytes),
        "swap_percent": format!("{:.0}", mem.swap_percent),
        "swap_used_gib": gib(mem.swap_used_bytes),
        "swap_total_gib": gib(mem.swap_total_bytes),
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

fn gib(bytes: u64) -> String {
    format!("{:.1}", ByteSize::b(bytes).as_gib())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1024 * 1024 * 1024;

    fn mem_data(
        used: u64,
        total: u64,
        available: u64,
        usage_percent: f32,
        swap_used: u64,
        swap_total: u64,
        swap_percent: f32,
    ) -> MemoryData {
        MemoryData {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
            swap_total_bytes: swap_total,
            swap_used_bytes: swap_used,
            swap_percent,
        }
    }

    #[test]
    fn format_label_replaces_percent_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 0, 4 * GIB, 0.0);
        let result = format_label("{{ percent }}%", &mem);
        assert_eq!(result, "50%");
    }

    #[test]
    fn format_label_percent_minimal_digits() {
        let mem = mem_data(GIB, 16 * GIB, 15 * GIB, 6.25, 0, 4 * GIB, 0.0);
        let result = format_label("{{ percent }}", &mem);
        assert_eq!(result, "6");
    }

    #[test]
    fn format_label_replaces_used_gib_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 0, 4 * GIB, 0.0);
        let result = format_label("{{ used_gib }}G", &mem);
        assert_eq!(result, "8.0G");
    }

    #[test]
    fn format_label_replaces_total_gib_placeholder() {
        let mem = mem_data(8 * GIB, 32 * GIB, 24 * GIB, 25.0, 0, 4 * GIB, 0.0);
        let result = format_label("{{ total_gib }}G", &mem);
        assert_eq!(result, "32.0G");
    }

    #[test]
    fn format_label_replaces_available_gib_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 7 * GIB, 50.0, 0, 4 * GIB, 0.0);
        let result = format_label("{{ available_gib }}G free", &mem);
        assert_eq!(result, "7.0G free");
    }

    #[test]
    fn format_label_replaces_swap_percent_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 2 * GIB, 4 * GIB, 50.0);
        let result = format_label("Swap: {{ swap_percent }}%", &mem);
        assert_eq!(result, "Swap: 50%");
    }

    #[test]
    fn format_label_replaces_swap_used_gib_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 2 * GIB, 4 * GIB, 50.0);
        let result = format_label("{{ swap_used_gib }}G", &mem);
        assert_eq!(result, "2.0G");
    }

    #[test]
    fn format_label_replaces_swap_total_gib_placeholder() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 2 * GIB, 8 * GIB, 25.0);
        let result = format_label("{{ swap_total_gib }}G", &mem);
        assert_eq!(result, "8.0G");
    }

    #[test]
    fn format_label_with_zero_swap() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 0, 0, 0.0);
        let result = format_label("{{ swap_used_gib }}/{{ swap_total_gib }}", &mem);
        assert_eq!(result, "0.0/0.0");
    }

    #[test]
    fn format_label_with_multiple_placeholders() {
        let mem = mem_data(12 * GIB, 32 * GIB, 20 * GIB, 37.5, GIB, 8 * GIB, 12.5);
        let result = format_label("{{ used_gib }}/{{ total_gib }}G ({{ percent }}%)", &mem);
        assert_eq!(result, "12.0/32.0G (38%)");
    }

    #[test]
    fn format_label_with_fractional_gib() {
        let bytes = (1.5 * GIB as f64) as u64;
        let mem = mem_data(bytes, 16 * GIB, 14 * GIB, 9.4, 0, 4 * GIB, 0.0);
        let result = format_label("{{ used_gib }}", &mem);
        assert_eq!(result, "1.5");
    }

    #[test]
    fn format_label_with_no_placeholders_returns_unchanged() {
        let mem = mem_data(8 * GIB, 16 * GIB, 8 * GIB, 50.0, 0, 4 * GIB, 0.0);
        let result = format_label("RAM", &mem);
        assert_eq!(result, "RAM");
    }
}
