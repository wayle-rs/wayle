use std::collections::HashMap;

use libpulse_binding::time::MicroSeconds;

use super::format::{AudioFormat, ChannelMap, SampleSpec};
use crate::volume::types::Volume;

/// Device state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceState {
    /// Device is running and available
    Running,
    /// Device is idle
    Idle,
    /// Device is suspended
    Suspended,
    /// Device is offline or unavailable
    Offline,
}

/// Device port information
#[derive(Debug, Clone, PartialEq)]
pub struct DevicePort {
    /// Port name
    pub name: String,
    /// Port description
    pub description: String,
    /// Port priority
    pub priority: u32,
    /// Port availability
    pub available: bool,
}

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    /// Audio input device (microphone, line-in)
    Input,
    /// Audio output device (speakers, headphones)
    Output,
}

/// Device key for unique identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceKey {
    /// Device index
    pub index: u32,
    /// Device type
    pub device_type: DeviceType,
}

impl DeviceKey {
    /// Create a new device key
    pub fn new(index: u32, device_type: DeviceType) -> Self {
        Self { index, device_type }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeviceInfo {
    pub index: u32,
    pub name: String,
    pub description: String,
    pub card_index: Option<u32>,
    pub owner_module: Option<u32>,
    pub driver: String,
    pub state: DeviceState,
    pub volume: Volume,
    pub base_volume: Volume,
    pub n_volume_steps: u32,
    pub muted: bool,
    pub properties: HashMap<String, String>,
    pub ports: Vec<DevicePort>,
    pub active_port: Option<String>,
    pub formats: Vec<AudioFormat>,
    pub sample_spec: SampleSpec,
    pub channel_map: ChannelMap,
    pub latency: MicroSeconds,
    pub configured_latency: MicroSeconds,
    pub flags: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SinkInfo {
    pub device: DeviceInfo,
    pub monitor_source: u32,
    pub monitor_source_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SourceInfo {
    pub device: DeviceInfo,
    pub monitor_of_sink: Option<u32>,
    pub monitor_of_sink_name: Option<String>,
    pub is_monitor: bool,
}

impl DeviceInfo {
    pub(crate) fn key(&self, device_type: DeviceType) -> DeviceKey {
        DeviceKey {
            index: self.index,
            device_type,
        }
    }
}

impl SinkInfo {
    pub(crate) fn key(&self) -> DeviceKey {
        self.device.key(DeviceType::Output)
    }
}

impl SourceInfo {
    pub(crate) fn key(&self) -> DeviceKey {
        self.device.key(DeviceType::Input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Device {
    Sink(SinkInfo),
    Source(SourceInfo),
}

impl Device {
    pub(crate) fn key(&self) -> DeviceKey {
        match self {
            Device::Sink(sink) => sink.key(),
            Device::Source(source) => source.key(),
        }
    }
}
