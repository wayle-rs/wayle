use std::collections::HashMap;

/// Sample specification
#[derive(Debug, Clone, PartialEq)]
pub struct SampleSpec {
    /// Sample rate in Hz
    pub rate: u32,
    /// Number of channels
    pub channels: u8,
    /// Sample format
    pub format: SampleFormat,
}

/// Sample format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// Unsigned 8-bit samples.
    U8,
    /// 8-bit a-Law encoded samples.
    ALaw,
    /// 8-bit mu-Law encoded samples.
    ULaw,
    /// Signed 16-bit little-endian samples.
    S16LE,
    /// Signed 16-bit big-endian samples.
    S16BE,
    /// Signed 24-bit little-endian samples.
    S24LE,
    /// Signed 24-bit big-endian samples.
    S24BE,
    /// Signed 24-bit samples in LSB of 32-bit words, little-endian.
    S24_32LE,
    /// Signed 24-bit samples in LSB of 32-bit words, big-endian.
    S24_32BE,
    /// Signed 32-bit little-endian samples.
    S32LE,
    /// Signed 32-bit big-endian samples.
    S32BE,
    /// Float 32-bit little-endian samples.
    F32LE,
    /// Float 32-bit big-endian samples.
    F32BE,
    /// Unknown format.
    Unknown,
}

/// Channel map for audio channels
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMap {
    /// Number of channels
    pub channels: u8,
    /// Channel positions
    pub positions: Vec<ChannelPosition>,
}

/// Channel position enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelPosition {
    /// Single mono channel.
    Mono,
    /// Front left.
    FrontLeft,
    /// Front right.
    FrontRight,
    /// Front center.
    FrontCenter,
    /// Rear left.
    RearLeft,
    /// Rear right.
    RearRight,
    /// Rear center (Dolby: "Surround Rear Center").
    RearCenter,
    /// Low frequency effects (subwoofer).
    LFE,
    /// Side left (Dolby: "Surround Left").
    SideLeft,
    /// Side right (Dolby: "Surround Right").
    SideRight,
    /// Front left of center (Dolby: "Left Center").
    FrontLeftOfCenter,
    /// Front right of center (Dolby: "Right Center").
    FrontRightOfCenter,
    /// Top center (Apple: "Top Center Surround").
    TopCenter,
    /// Top front left (Apple: "Vertical Height Left").
    TopFrontLeft,
    /// Top front right (Apple: "Vertical Height Right").
    TopFrontRight,
    /// Top front center (Apple: "Vertical Height Center").
    TopFrontCenter,
    /// Top rear left (Microsoft/Apple: "Top Back Left").
    TopRearLeft,
    /// Top rear right (Microsoft/Apple: "Top Back Right").
    TopRearRight,
    /// Top rear center (Microsoft/Apple: "Top Back Center").
    TopRearCenter,
    /// Unrecognized position from PulseAudio.
    Unknown,
}

/// Audio format information
#[derive(Debug, Clone, PartialEq)]
pub struct AudioFormat {
    /// Encoding type
    pub encoding: String,
    /// Properties of the format
    pub properties: HashMap<String, String>,
}
