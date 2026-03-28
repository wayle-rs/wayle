//! Parser for WireGuard `.conf` files.
//!
//! Converts INI-style WireGuard configuration into NetworkManager connection
//! settings for import.

use crate::error::Error;

/// Parsed WireGuard configuration.
#[derive(Debug, Clone)]
pub struct WireGuardConfig {
    /// Interface section fields.
    pub interface: InterfaceConfig,
    /// Peer sections.
    pub peers: Vec<PeerConfig>,
}

/// The [Interface] section of a WireGuard config.
#[derive(Debug, Clone, Default)]
pub struct InterfaceConfig {
    /// Base64-encoded private key.
    pub private_key: String,
    /// UDP listen port (optional).
    pub listen_port: Option<u16>,
    /// Comma-separated DNS servers.
    pub dns: Vec<String>,
    /// Interface addresses (e.g. "10.0.0.2/24").
    pub addresses: Vec<String>,
    /// MTU (optional).
    pub mtu: Option<u32>,
}

/// A [Peer] section of a WireGuard config.
#[derive(Debug, Clone, Default)]
pub struct PeerConfig {
    /// Base64-encoded public key.
    pub public_key: String,
    /// Base64-encoded preshared key (optional).
    pub preshared_key: Option<String>,
    /// Endpoint address (e.g. "1.2.3.4:51820").
    pub endpoint: Option<String>,
    /// Allowed IPs (e.g. "0.0.0.0/0, ::/0").
    pub allowed_ips: Vec<String>,
    /// Persistent keepalive interval in seconds.
    pub persistent_keepalive: Option<u32>,
}

/// Parse a WireGuard `.conf` file from its text content.
///
/// # Errors
///
/// Returns `Error::DataConversionFailed` if the config is malformed.
pub fn parse_config(content: &str) -> Result<WireGuardConfig, Error> {
    let mut interface = InterfaceConfig::default();
    let mut peers = Vec::new();
    let mut current_peer: Option<PeerConfig> = None;
    let mut in_interface = false;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.eq_ignore_ascii_case("[interface]") {
            if let Some(peer) = current_peer.take() {
                peers.push(peer);
            }
            in_interface = true;
            continue;
        }

        if line.eq_ignore_ascii_case("[peer]") {
            if let Some(peer) = current_peer.take() {
                peers.push(peer);
            }
            in_interface = false;
            current_peer = Some(PeerConfig::default());
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim();

        if in_interface {
            parse_interface_field(&mut interface, key, value);
        } else if let Some(ref mut peer) = current_peer {
            parse_peer_field(peer, key, value);
        }
    }

    if let Some(peer) = current_peer.take() {
        peers.push(peer);
    }

    if interface.private_key.is_empty() {
        return Err(Error::DataConversionFailed {
            data_type: String::from("WireGuard config"),
            reason: String::from("missing required PrivateKey in [Interface] section"),
        });
    }

    Ok(WireGuardConfig { interface, peers })
}

fn parse_interface_field(interface: &mut InterfaceConfig, key: &str, value: &str) {
    match key.to_ascii_lowercase().as_str() {
        "privatekey" => interface.private_key = value.to_owned(),
        "listenport" => interface.listen_port = value.parse().ok(),
        "dns" => {
            interface.dns = value
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "address" => {
            interface.addresses = value
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "mtu" => interface.mtu = value.parse().ok(),
        _ => {}
    }
}

fn parse_peer_field(peer: &mut PeerConfig, key: &str, value: &str) {
    match key.to_ascii_lowercase().as_str() {
        "publickey" => peer.public_key = value.to_owned(),
        "presharedkey" => peer.preshared_key = Some(value.to_owned()),
        "endpoint" => peer.endpoint = Some(value.to_owned()),
        "allowedips" => {
            peer.allowed_ips = value
                .split(',')
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
                .collect();
        }
        "persistentkeepalive" => peer.persistent_keepalive = value.parse().ok(),
        _ => {}
    }
}


#[cfg(test)]
fn parse_ipv4_to_u32(ip: &str) -> Option<u32> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return None;
    }

    let octets: Vec<u8> = parts.iter().filter_map(|p| p.parse().ok()).collect();
    if octets.len() != 4 {
        return None;
    }

    // NetworkManager stores IPv4 DNS in network byte order (big-endian)
    Some(
        u32::from(octets[0])
            | (u32::from(octets[1]) << 8)
            | (u32::from(octets[2]) << 16)
            | (u32::from(octets[3]) << 24),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_config() {
        let config = "\
[Interface]
PrivateKey = yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk=
Address = 10.200.200.2/32
DNS = 10.200.200.1

[Peer]
PublicKey = xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg=
Endpoint = demo.wireguard.com:51820
AllowedIPs = 0.0.0.0/0, ::/0
PersistentKeepalive = 25
";
        let result = parse_config(config);
        assert!(result.is_ok());

        let wg = result.expect("config should parse");
        assert_eq!(
            wg.interface.private_key,
            "yAnz5TF+lXXJte14tji3zlMNq+hd2rYUIgJBgB3fBmk="
        );
        assert_eq!(wg.interface.addresses, vec!["10.200.200.2/32"]);
        assert_eq!(wg.interface.dns, vec!["10.200.200.1"]);
        assert_eq!(wg.peers.len(), 1);
        assert_eq!(
            wg.peers[0].public_key,
            "xTIBA5rboUvnH4htodjb6e697QjLERt1NAB4mZqp8Dg="
        );
        assert_eq!(
            wg.peers[0].endpoint,
            Some(String::from("demo.wireguard.com:51820"))
        );
        assert_eq!(wg.peers[0].allowed_ips, vec!["0.0.0.0/0", "::/0"]);
        assert_eq!(wg.peers[0].persistent_keepalive, Some(25));
    }

    #[test]
    fn parse_missing_private_key_fails() {
        let config = "\
[Interface]
Address = 10.0.0.1/24

[Peer]
PublicKey = abc123
";
        let result = parse_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn parse_multiple_peers() {
        let config = "\
[Interface]
PrivateKey = aGVsbG8=
Address = 10.0.0.1/24

[Peer]
PublicKey = peer1key
Endpoint = 1.2.3.4:51820
AllowedIPs = 10.0.0.0/24

[Peer]
PublicKey = peer2key
Endpoint = 5.6.7.8:51820
AllowedIPs = 10.1.0.0/24
PresharedKey = psk123
";
        let wg = parse_config(config).expect("should parse");
        assert_eq!(wg.peers.len(), 2);
        assert_eq!(wg.peers[0].public_key, "peer1key");
        assert_eq!(wg.peers[1].public_key, "peer2key");
        assert_eq!(
            wg.peers[1].preshared_key,
            Some(String::from("psk123"))
        );
    }

    #[test]
    fn parse_ipv4_to_u32_valid() {
        // 8.8.8.8 in NM byte order
        let result = parse_ipv4_to_u32("8.8.8.8");
        assert_eq!(result, Some(0x08_08_08_08));
    }

    #[test]
    fn parse_ipv4_to_u32_localhost() {
        // 127.0.0.1 in NM byte order (little-endian storage)
        let result = parse_ipv4_to_u32("127.0.0.1");
        assert_eq!(result, Some(0x0100_007F));
    }
}
