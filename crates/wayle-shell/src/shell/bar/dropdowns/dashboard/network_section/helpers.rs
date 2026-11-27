use wayle_sysinfo::types::NetworkData;

const BYTES_PER_KB: f64 = 1024.0;

pub(super) struct FormattedSpeed {
    pub value: String,
    pub is_megabytes: bool,
}

/// Infers whether the machine has an active network connection from sysinfo
/// interface data.
///
/// Used as a fallback when NetworkManager is unavailable (for example on
/// systemd-networkd systems), where connection state cannot be queried over
/// D-Bus. Any non-loopback interface that has carried traffic is treated as a
/// live connection.
pub(super) fn infer_connected(interfaces: &[NetworkData]) -> bool {
    interfaces.iter().any(|iface| {
        !iface.interface.starts_with("lo") && (iface.rx_bytes > 0 || iface.tx_bytes > 0)
    })
}

/// Formats bytes per second into a human-readable speed value and unit flag.
pub(super) fn format_speed(bytes_per_sec: u64) -> FormattedSpeed {
    let kbps = bytes_per_sec as f64 / BYTES_PER_KB;
    if kbps < BYTES_PER_KB {
        FormattedSpeed {
            value: format!("{kbps:.1}"),
            is_megabytes: false,
        }
    } else {
        let mbps = kbps / BYTES_PER_KB;
        FormattedSpeed {
            value: format!("{mbps:.1}"),
            is_megabytes: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net_data(interface: &str, rx_bytes: u64, tx_bytes: u64) -> NetworkData {
        NetworkData {
            interface: interface.to_string(),
            rx_bytes,
            tx_bytes,
            rx_bytes_per_sec: 0,
            tx_bytes_per_sec: 0,
        }
    }

    #[test]
    fn infer_connected_true_for_interface_with_traffic() {
        let interfaces = vec![net_data("lo", 5000, 5000), net_data("eth0", 1000, 0)];
        assert!(infer_connected(&interfaces));
    }

    #[test]
    fn infer_connected_counts_transmit_only_interface() {
        let interfaces = vec![net_data("wlan0", 0, 42)];
        assert!(infer_connected(&interfaces));
    }

    #[test]
    fn infer_connected_ignores_loopback() {
        let interfaces = vec![net_data("lo", 999_999, 999_999)];
        assert!(!infer_connected(&interfaces));
    }

    #[test]
    fn infer_connected_false_for_idle_interfaces() {
        let interfaces = vec![net_data("eth0", 0, 0), net_data("wg0", 0, 0)];
        assert!(!infer_connected(&interfaces));
    }

    #[test]
    fn infer_connected_false_for_empty_list() {
        assert!(!infer_connected(&[]));
    }
}
