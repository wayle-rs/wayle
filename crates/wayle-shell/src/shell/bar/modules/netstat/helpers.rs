use bytesize::ByteSize;
use serde_json::json;
use wayle_sysinfo::types::NetworkData;

pub(super) fn format_label(format: &str, net: &NetworkData) -> String {
    let ctx = json!({
        "down_kib": kib(net.rx_bytes_per_sec),
        "down_mib": mib(net.rx_bytes_per_sec),
        "down_gib": gib(net.rx_bytes_per_sec),
        "down_auto": auto(net.rx_bytes_per_sec),
        "up_kib": kib(net.tx_bytes_per_sec),
        "up_mib": mib(net.tx_bytes_per_sec),
        "up_gib": gib(net.tx_bytes_per_sec),
        "up_auto": auto(net.tx_bytes_per_sec),
        "interface": &net.interface,
    });
    crate::template::render(format, ctx).unwrap_or_default()
}

fn kib(bytes: u64) -> String {
    format!("{:.0}", ByteSize::b(bytes).as_kib())
}

fn mib(bytes: u64) -> String {
    format!("{:.1}", ByteSize::b(bytes).as_mib())
}

fn gib(bytes: u64) -> String {
    format!("{:.2}", ByteSize::b(bytes).as_gib())
}

/// Auto-scaled rate with a stable shape: always one decimal place and a 3-char
/// unit (KiB/MiB/GiB/TiB). Flooring at KiB avoids the sub-KiB "B" tier, which
/// would otherwise vary the decimal structure and unit length as traffic idles.
fn auto(bytes: u64) -> String {
    const KIB: u64 = 1024;
    if bytes < KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        ByteSize::b(bytes).to_string()
    }
}

pub(super) fn select_interface<'a>(
    networks: &'a [NetworkData],
    interface_config: &str,
) -> Option<&'a NetworkData> {
    if interface_config == "auto" {
        networks
            .iter()
            .filter(|n| !n.interface.starts_with("lo"))
            .max_by_key(|n| n.rx_bytes_per_sec + n.tx_bytes_per_sec)
    } else {
        networks.iter().find(|n| n.interface == interface_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    const GIB: u64 = 1024 * 1024 * 1024;

    fn net_data(interface: &str, rx_per_sec: u64, tx_per_sec: u64) -> NetworkData {
        NetworkData {
            interface: interface.to_string(),
            rx_bytes: 0,
            tx_bytes: 0,
            rx_bytes_per_sec: rx_per_sec,
            tx_bytes_per_sec: tx_per_sec,
        }
    }

    #[test]
    fn format_label_replaces_down_kib_placeholder() {
        let net = net_data("eth0", 500 * KIB, 100 * KIB);
        let result = format_label("{{ down_kib }} KiB/s", &net);
        assert_eq!(result, "500 KiB/s");
    }

    #[test]
    fn format_label_replaces_down_mib_placeholder() {
        let net = net_data("eth0", 50 * MIB, 10 * MIB);
        let result = format_label("{{ down_mib }} MiB/s", &net);
        assert_eq!(result, "50.0 MiB/s");
    }

    #[test]
    fn format_label_replaces_down_gib_placeholder() {
        let net = net_data("eth0", GIB, 100 * MIB);
        let result = format_label("{{ down_gib }} GiB/s", &net);
        assert_eq!(result, "1.00 GiB/s");
    }

    #[test]
    fn format_label_replaces_down_auto_placeholder() {
        let net = net_data("eth0", 500 * KIB, 100 * KIB);
        let result = format_label("{{ down_auto }}", &net);
        assert_eq!(result, "500.0 KiB");
    }

    #[test]
    fn format_label_replaces_up_kib_placeholder() {
        let net = net_data("eth0", 100 * KIB, 250 * KIB);
        let result = format_label("{{ up_kib }} KiB/s", &net);
        assert_eq!(result, "250 KiB/s");
    }

    #[test]
    fn format_label_replaces_up_mib_placeholder() {
        let net = net_data("eth0", 10 * MIB, 25 * MIB);
        let result = format_label("{{ up_mib }} MiB/s", &net);
        assert_eq!(result, "25.0 MiB/s");
    }

    #[test]
    fn format_label_replaces_up_gib_placeholder() {
        let net = net_data("eth0", 100 * MIB, 2 * GIB);
        let result = format_label("{{ up_gib }} GiB/s", &net);
        assert_eq!(result, "2.00 GiB/s");
    }

    #[test]
    fn format_label_replaces_up_auto_placeholder() {
        let net = net_data("eth0", 100 * KIB, 2 * MIB);
        let result = format_label("{{ up_auto }}", &net);
        assert_eq!(result, "2.0 MiB");
    }

    #[test]
    fn format_label_auto_floors_sub_kib_at_kib() {
        // Idle / sub-KiB traffic keeps the 1-decimal + KiB shape instead of "0 B".
        let net = net_data("eth0", 0, 512);
        assert_eq!(format_label("{{ down_auto }}", &net), "0.0 KiB");
        assert_eq!(format_label("{{ up_auto }}", &net), "0.5 KiB");
    }

    #[test]
    fn format_label_replaces_interface_placeholder() {
        let net = net_data("wlan0", 100 * KIB, 50 * KIB);
        let result = format_label("[{{ interface }}]", &net);
        assert_eq!(result, "[wlan0]");
    }

    #[test]
    fn format_label_with_multiple_placeholders() {
        let net = net_data("eth0", 1024 * KIB, 512 * KIB);
        let result = format_label("{{ down_kib }}/{{ up_kib }} on {{ interface }}", &net);
        assert_eq!(result, "1024/512 on eth0");
    }

    #[test]
    fn format_label_with_zero_traffic() {
        let net = net_data("eth0", 0, 0);
        let result = format_label("{{ down_kib }}/{{ up_kib }}", &net);
        assert_eq!(result, "0/0");
    }

    #[test]
    fn format_label_with_no_placeholders_returns_unchanged() {
        let net = net_data("eth0", 100 * KIB, 50 * KIB);
        let result = format_label("Network", &net);
        assert_eq!(result, "Network");
    }

    #[test]
    fn select_interface_auto_excludes_loopback() {
        let networks = vec![
            net_data("lo", 1000 * MIB, 1000 * MIB),
            net_data("eth0", 10 * MIB, 5 * MIB),
        ];
        let result = select_interface(&networks, "auto");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "eth0");
    }

    #[test]
    fn select_interface_auto_excludes_localhost_variants() {
        let networks = vec![
            net_data("lo0", 1000 * MIB, 1000 * MIB),
            net_data("lo1", 500 * MIB, 500 * MIB),
            net_data("wlan0", 5 * MIB, 2 * MIB),
        ];
        let result = select_interface(&networks, "auto");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "wlan0");
    }

    #[test]
    fn select_interface_auto_selects_highest_throughput() {
        let networks = vec![
            net_data("eth0", 10 * MIB, 5 * MIB),
            net_data("wlan0", 50 * MIB, 20 * MIB),
            net_data("eth1", 5 * MIB, 2 * MIB),
        ];
        let result = select_interface(&networks, "auto");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "wlan0");
    }

    #[test]
    fn select_interface_auto_sums_rx_and_tx_for_throughput() {
        let networks = vec![
            net_data("eth0", 100 * MIB, 5 * MIB),
            net_data("wlan0", 50 * MIB, 60 * MIB),
        ];
        let result = select_interface(&networks, "auto");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "wlan0");
    }

    #[test]
    fn select_interface_auto_returns_none_for_empty_list() {
        let networks: Vec<NetworkData> = vec![];
        let result = select_interface(&networks, "auto");
        assert!(result.is_none());
    }

    #[test]
    fn select_interface_auto_returns_none_when_only_loopback() {
        let networks = vec![
            net_data("lo", 1000 * MIB, 1000 * MIB),
            net_data("lo0", 500 * MIB, 500 * MIB),
        ];
        let result = select_interface(&networks, "auto");
        assert!(result.is_none());
    }

    #[test]
    fn select_interface_explicit_finds_matching_interface() {
        let networks = vec![
            net_data("eth0", 10 * MIB, 5 * MIB),
            net_data("wlan0", 50 * MIB, 20 * MIB),
            net_data("eth1", 5 * MIB, 2 * MIB),
        ];
        let result = select_interface(&networks, "eth1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "eth1");
    }

    #[test]
    fn select_interface_explicit_returns_none_when_not_found() {
        let networks = vec![
            net_data("eth0", 10 * MIB, 5 * MIB),
            net_data("wlan0", 50 * MIB, 20 * MIB),
        ];
        let result = select_interface(&networks, "eth1");
        assert!(result.is_none());
    }

    #[test]
    fn select_interface_explicit_can_select_loopback() {
        let networks = vec![
            net_data("lo", 1000 * MIB, 1000 * MIB),
            net_data("eth0", 10 * MIB, 5 * MIB),
        ];
        let result = select_interface(&networks, "lo");
        assert!(result.is_some());
        assert_eq!(result.unwrap().interface, "lo");
    }

    #[test]
    fn select_interface_explicit_is_case_sensitive() {
        let networks = vec![
            net_data("ETH0", 10 * MIB, 5 * MIB),
            net_data("eth0", 50 * MIB, 20 * MIB),
        ];
        let result = select_interface(&networks, "ETH0");
        assert!(result.is_some());
        assert_eq!(result.unwrap().rx_bytes_per_sec, 10 * MIB);
    }
}
