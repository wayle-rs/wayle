use std::{borrow::Cow, collections::HashMap};

use libpulse_binding::{def::PortAvailable, format::Info as PulseFormatInfo, proplist::Proplist};

use crate::types::{device::DevicePort, format::AudioFormat, stream::MediaInfo};

pub(super) fn cow_to_string(cow: Option<&Cow<'_, str>>) -> String {
    cow.map(|s| s.to_string()).unwrap_or_default()
}

pub(super) fn collect_proplist(proplist: &Proplist) -> HashMap<String, String> {
    proplist
        .iter()
        .filter_map(|key| proplist.get_str(&key).map(|value| (key.to_string(), value)))
        .collect()
}

pub(super) fn convert_formats(formats: &[PulseFormatInfo]) -> Vec<AudioFormat> {
    formats
        .iter()
        .map(|format_info| AudioFormat {
            encoding: format!("{:?}", format_info.get_encoding()),
            properties: collect_proplist(format_info.get_properties()),
        })
        .collect()
}

pub(super) fn convert_ports(ports: &[impl PulsePort]) -> Vec<DevicePort> {
    ports
        .iter()
        .map(|port| DevicePort {
            name: cow_to_string(port.port_name()),
            description: cow_to_string(port.port_description()),
            priority: port.port_priority(),
            available: port.port_available() != PortAvailable::No,
        })
        .collect()
}

pub(super) fn active_port_name(active_port: &Option<Box<impl PulsePort>>) -> Option<String> {
    active_port
        .as_ref()
        .and_then(|port| port.port_name().map(|name| name.to_string()))
}

pub(super) fn extract_media_info(proplist: &Proplist) -> MediaInfo {
    MediaInfo {
        title: proplist.get_str("media.title"),
        artist: proplist.get_str("media.artist"),
        album: proplist.get_str("media.album"),
        icon_name: proplist.get_str("application.icon_name"),
    }
}

pub(super) trait PulsePort {
    fn port_name(&self) -> Option<&Cow<'_, str>>;
    fn port_description(&self) -> Option<&Cow<'_, str>>;
    fn port_priority(&self) -> u32;
    fn port_available(&self) -> PortAvailable;
}

impl PulsePort for libpulse_binding::context::introspect::SinkPortInfo<'_> {
    fn port_name(&self) -> Option<&Cow<'_, str>> {
        self.name.as_ref()
    }

    fn port_description(&self) -> Option<&Cow<'_, str>> {
        self.description.as_ref()
    }

    fn port_priority(&self) -> u32 {
        self.priority
    }

    fn port_available(&self) -> PortAvailable {
        self.available
    }
}

impl PulsePort for libpulse_binding::context::introspect::SourcePortInfo<'_> {
    fn port_name(&self) -> Option<&Cow<'_, str>> {
        self.name.as_ref()
    }

    fn port_description(&self) -> Option<&Cow<'_, str>> {
        self.description.as_ref()
    }

    fn port_priority(&self) -> u32 {
        self.priority
    }

    fn port_available(&self) -> PortAvailable {
        self.available
    }
}

#[cfg(test)]
mod tests {
    use libpulse_binding::format::{Encoding, Info as PulseFormatInfo};

    use super::*;

    fn format_with_encoding(encoding: Encoding) -> PulseFormatInfo {
        let mut format_info = PulseFormatInfo::new().expect("format info should allocate");
        format_info.set_encoding(encoding);
        format_info
    }

    #[test]
    fn convert_formats_supports_hdmi_encoded_formats() {
        let truehd = format_with_encoding(Encoding::TRUEHD_IEC61937);
        let dtshd = format_with_encoding(Encoding::DTSHD_IEC61937);

        let formats = convert_formats(&[truehd, dtshd]);

        assert_eq!(formats[0].encoding, "TRUEHD_IEC61937");
        assert_eq!(formats[1].encoding, "DTSHD_IEC61937");
    }
}
