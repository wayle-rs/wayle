//! On-screen display domain logic.
//!
//! Resolves the device named on the command line before publishing an
//! [`OsdRequest`], so a bad name fails the D-Bus call instead of silently
//! doing nothing, and so [`OsdControl::devices`] can list exactly the
//! identifiers the show methods accept.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use tracing::{debug, instrument};
use wayle_audio::{
    AudioService,
    core::device::{input::InputDevice, output::OutputDevice},
};
use wayle_brightness::{BacklightDevice, BrightnessService};
use wayle_config::ConfigService;
use wayle_ipc::shell_ipc::OsdDeviceInfo;
use zbus::fdo;

use super::state::{OsdDevice, OsdRequest, ShellIpcState};

const KIND_SPEAKER: &str = "speaker";
const KIND_MICROPHONE: &str = "microphone";
const KIND_BRIGHTNESS: &str = "brightness";

/// One targetable device, flattened for matching and error messages.
struct Candidate {
    /// PulseAudio sink or source index. `None` for backlights.
    index: Option<u32>,

    /// Stable identifier: node name for audio, sysfs name for backlights.
    id: String,

    /// Human-readable name, as shown on the overlay.
    label: String,
}

/// Why a query didn't resolve to exactly one device.
enum Unresolved {
    NoMatch,
    Ambiguous(Vec<String>),
}

/// OSD display logic. Resolves device queries against the live service state
/// and publishes [`ShellIpcState::osd_request`].
pub(crate) struct OsdControl {
    state: ShellIpcState,
    config: Arc<ConfigService>,
    audio: Option<Arc<AudioService>>,
    brightness: Option<Arc<BrightnessService>>,
    seq: AtomicU64,
}

impl OsdControl {
    pub(crate) fn new(
        state: ShellIpcState,
        config: Arc<ConfigService>,
        audio: Option<Arc<AudioService>>,
        brightness: Option<Arc<BrightnessService>>,
    ) -> Self {
        Self {
            state,
            config,
            audio,
            brightness,
            seq: AtomicU64::new(0),
        }
    }

    /// Shows the speaker OSD. Empty query targets the default output.
    #[instrument(skip(self))]
    pub(crate) fn show_speaker(&self, query: &str) -> fdo::Result<String> {
        self.ensure_enabled()?;

        let device = self.resolve_output(query)?;
        let label = device.description.get();

        self.publish(OsdDevice::Speaker(device));

        Ok(label)
    }

    /// Shows the microphone OSD. Empty query targets the default input.
    #[instrument(skip(self))]
    pub(crate) fn show_microphone(&self, query: &str) -> fdo::Result<String> {
        self.ensure_enabled()?;

        let device = self.resolve_input(query)?;
        let label = device.description.get();

        self.publish(OsdDevice::Microphone(device));

        Ok(label)
    }

    /// Shows the brightness OSD. Empty query targets the primary backlight.
    #[instrument(skip(self))]
    pub(crate) fn show_brightness(&self, query: &str) -> fdo::Result<String> {
        self.ensure_enabled()?;

        let device = self.resolve_brightness(query)?;
        let label = device.name.to_string();

        self.publish(OsdDevice::Brightness(device));

        Ok(label)
    }

    /// Every device the show methods can target.
    pub(crate) fn devices(&self) -> Vec<OsdDeviceInfo> {
        let mut devices = Vec::new();

        if let Some(audio) = &self.audio {
            let default = audio.default_output.get().map(|device| device.key);

            devices.extend(audio.output_devices.get().iter().map(|device| OsdDeviceInfo {
                kind: String::from(KIND_SPEAKER),
                id: device.name.get(),
                label: device.description.get(),
                is_default: default == Some(device.key),
            }));

            let default = audio.default_input.get().map(|device| device.key);

            devices.extend(input_devices(audio).iter().map(|device| OsdDeviceInfo {
                kind: String::from(KIND_MICROPHONE),
                id: device.name.get(),
                label: device.description.get(),
                is_default: default == Some(device.key),
            }));
        }

        if let Some(brightness) = &self.brightness {
            let primary = brightness.primary.get().map(|device| device.name.clone());

            devices.extend(brightness.devices.get().iter().map(|device| OsdDeviceInfo {
                kind: String::from(KIND_BRIGHTNESS),
                id: device.name.to_string(),
                label: device.name.to_string(),
                is_default: primary.as_ref() == Some(&device.name),
            }));
        }

        devices
    }

    fn ensure_enabled(&self) -> fdo::Result<()> {
        if self.config.config().osd.enabled.get() {
            return Ok(());
        }

        Err(fdo::Error::Failed(String::from(
            "OSD is disabled; enable it with `wayle config set osd.enabled true`",
        )))
    }

    fn publish(&self, device: OsdDevice) {
        let seq = self.seq.fetch_add(1, Ordering::Relaxed).saturating_add(1);

        debug!(seq, "publishing OSD request");

        self.state
            .osd_request
            .replace(Some(OsdRequest { seq, device }));
    }

    fn audio(&self) -> fdo::Result<&Arc<AudioService>> {
        self.audio
            .as_ref()
            .ok_or_else(|| fdo::Error::Failed(String::from("audio service unavailable")))
    }

    fn brightness(&self) -> fdo::Result<&Arc<BrightnessService>> {
        self.brightness
            .as_ref()
            .ok_or_else(|| fdo::Error::Failed(String::from("no backlight devices available")))
    }

    fn resolve_output(&self, query: &str) -> fdo::Result<Arc<OutputDevice>> {
        let audio = self.audio()?;

        if query.is_empty() {
            return audio
                .default_output
                .get()
                .ok_or_else(|| fdo::Error::Failed(String::from("no default output device")));
        }

        resolve(
            query,
            "output device",
            audio.output_devices.get(),
            |device| Candidate {
                index: Some(device.key.index),
                id: device.name.get(),
                label: device.description.get(),
            },
        )
    }

    fn resolve_input(&self, query: &str) -> fdo::Result<Arc<InputDevice>> {
        let audio = self.audio()?;

        if query.is_empty() {
            return audio
                .default_input
                .get()
                .ok_or_else(|| fdo::Error::Failed(String::from("no default input device")));
        }

        resolve(query, "input device", input_devices(audio), |device| {
            Candidate {
                index: Some(device.key.index),
                id: device.name.get(),
                label: device.description.get(),
            }
        })
    }

    fn resolve_brightness(&self, query: &str) -> fdo::Result<Arc<BacklightDevice>> {
        let brightness = self.brightness()?;

        if query.is_empty() {
            return brightness
                .primary
                .get()
                .ok_or_else(|| fdo::Error::Failed(String::from("no backlight devices available")));
        }

        resolve(
            query,
            "backlight device",
            brightness.devices.get(),
            |device| Candidate {
                index: None,
                id: device.name.to_string(),
                label: device.name.to_string(),
            },
        )
    }
}

/// Input devices excluding monitor sources, which mirror an output and aren't
/// meaningful microphone targets.
fn input_devices(audio: &AudioService) -> Vec<Arc<InputDevice>> {
    audio
        .input_devices
        .get()
        .into_iter()
        .filter(|device| !device.is_monitor.get())
        .collect()
}

/// Picks the one device in `devices` that `query` names.
///
/// `kind` names the device class in error messages; `describe` flattens a
/// device into the fields the query is matched against.
fn resolve<T>(
    query: &str,
    kind: &str,
    devices: Vec<T>,
    describe: impl Fn(&T) -> Candidate,
) -> fdo::Result<T> {
    let candidates: Vec<Candidate> = devices.iter().map(describe).collect();

    let position =
        match_query(query, &candidates).map_err(|reason| error(kind, query, reason, &candidates))?;

    devices
        .into_iter()
        .nth(position)
        .ok_or_else(|| error(kind, query, Unresolved::NoMatch, &candidates))
}

/// Matches a query against candidates, most specific rule first: PulseAudio
/// index, exact id, case-insensitive label, then a substring of either.
///
/// A numeric query only ever means an index. Letting it fall through to
/// substring matching would resolve a stale index to whichever unrelated
/// device happens to have those digits in its node name.
fn match_query(query: &str, candidates: &[Candidate]) -> Result<usize, Unresolved> {
    if let Ok(index) = query.parse::<u32>()
        && candidates.iter().any(|it| it.index.is_some())
    {
        return candidates
            .iter()
            .position(|it| it.index == Some(index))
            .ok_or(Unresolved::NoMatch);
    }

    if let Some(position) = candidates.iter().position(|it| it.id == query) {
        return Ok(position);
    }

    let query = query.to_lowercase();

    let exact = positions(candidates, |it| it.label.to_lowercase() == query);

    if !exact.is_empty() {
        return only(exact, candidates);
    }

    let substring = positions(candidates, |it| {
        it.id.to_lowercase().contains(&query) || it.label.to_lowercase().contains(&query)
    });

    only(substring, candidates)
}

fn positions(candidates: &[Candidate], matches: impl Fn(&Candidate) -> bool) -> Vec<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| matches(candidate))
        .map(|(position, _)| position)
        .collect()
}

/// Accepts a match set only when it names one device.
fn only(matched: Vec<usize>, candidates: &[Candidate]) -> Result<usize, Unresolved> {
    match matched.as_slice() {
        [position] => Ok(*position),
        [] => Err(Unresolved::NoMatch),
        _ => Err(Unresolved::Ambiguous(
            matched
                .iter()
                .filter_map(|position| candidates.get(*position))
                .map(label)
                .collect(),
        )),
    }
}

fn error(kind: &str, query: &str, reason: Unresolved, candidates: &[Candidate]) -> fdo::Error {
    match reason {
        Unresolved::Ambiguous(matched) => fdo::Error::Failed(format!(
            "\"{query}\" matches several devices: {}",
            matched.join(", ")
        )),

        Unresolved::NoMatch if candidates.is_empty() => {
            fdo::Error::Failed(format!("no {kind} available"))
        }

        Unresolved::NoMatch => fdo::Error::Failed(format!(
            "no {kind} matching \"{query}\". Available: {}",
            candidates
                .iter()
                .map(label)
                .collect::<Vec<String>>()
                .join(", ")
        )),
    }
}

fn label(candidate: &Candidate) -> String {
    if candidate.id == candidate.label {
        return candidate.id.clone();
    }

    format!("{} ({})", candidate.label, candidate.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audio(index: u32, id: &str, label: &str) -> Candidate {
        Candidate {
            index: Some(index),
            id: String::from(id),
            label: String::from(label),
        }
    }

    fn backlight(name: &str) -> Candidate {
        Candidate {
            index: None,
            id: String::from(name),
            label: String::from(name),
        }
    }

    fn sinks() -> Vec<Candidate> {
        vec![
            audio(0, "alsa_output.pci-0000_00_1f.3.analog-stereo", "Built-in Audio"),
            audio(55, "bluez_output.AC_12_2F_5D_00_01.1", "WH-1000XM4"),
        ]
    }

    #[test]
    fn numeric_query_matches_index() {
        assert!(matches!(match_query("55", &sinks()), Ok(1)));
    }

    #[test]
    fn numeric_query_never_falls_through_to_substring() {
        // "3" appears in `pci-0000_00_1f.3`; a stale index must not silently
        // resolve to whichever device happens to have those digits.
        assert!(matches!(
            match_query("3", &sinks()),
            Err(Unresolved::NoMatch)
        ));
    }

    #[test]
    fn numeric_query_matches_by_name_when_no_device_has_an_index() {
        let backlights = vec![backlight("amdgpu_bl1"), backlight("acpi_video0")];

        assert!(matches!(match_query("1", &backlights), Ok(0)));
    }

    #[test]
    fn exact_id_wins_over_substring() {
        let candidates = vec![
            audio(0, "usb", "First"),
            audio(1, "usb-extended", "Second"),
        ];

        assert!(matches!(match_query("usb", &candidates), Ok(0)));
    }

    #[test]
    fn label_match_ignores_case() {
        assert!(matches!(match_query("wh-1000xm4", &sinks()), Ok(1)));
    }

    #[test]
    fn duplicate_labels_are_ambiguous() {
        let candidates = vec![
            audio(0, "alsa_output.usb-first", "USB Audio"),
            audio(1, "alsa_output.usb-second", "USB Audio"),
        ];

        assert!(matches!(
            match_query("usb audio", &candidates),
            Err(Unresolved::Ambiguous(_))
        ));
    }

    #[test]
    fn unique_substring_resolves() {
        assert!(matches!(match_query("xm4", &sinks()), Ok(1)));
    }

    #[test]
    fn shared_substring_is_ambiguous() {
        assert!(matches!(
            match_query("output", &sinks()),
            Err(Unresolved::Ambiguous(_))
        ));
    }

    #[test]
    fn unknown_query_does_not_match() {
        assert!(matches!(
            match_query("nonexistent", &sinks()),
            Err(Unresolved::NoMatch)
        ));
    }
}
