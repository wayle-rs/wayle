use clap::Subcommand;

use crate::styled_header;

/// On-screen display subcommands.
#[derive(Subcommand, Debug)]
#[command(after_long_help = OSD_EXAMPLES)]
pub enum OsdCommands {
    /// Show the speaker OSD
    #[command(after_long_help = DEVICE_MATCHING)]
    Speaker {
        /// Output device (omit for the default sink)
        #[arg(value_name = "DEVICE")]
        device: Option<String>,
    },

    /// Show the microphone OSD
    #[command(after_long_help = DEVICE_MATCHING)]
    Mic {
        /// Input device (omit for the default source)
        #[arg(value_name = "DEVICE")]
        device: Option<String>,
    },

    /// Show the display brightness OSD
    #[command(after_long_help = DEVICE_MATCHING)]
    Brightness {
        /// Backlight device (omit for the primary backlight)
        #[arg(value_name = "DEVICE")]
        device: Option<String>,
    },

    /// List devices the show commands can target
    Devices,
}

const OSD_EXAMPLES: &str = concat!(
    styled_header!("Examples:"),
    "
  wayle osd speaker                 Show the default sink's volume
  wayle osd mic                     Show the default source's volume
  wayle osd brightness              Show the primary backlight
  wayle osd devices                 List targetable devices

The OSD appears even when the value has not changed, so these pair well with
keybinds. To stop an OSD appearing on every change, turn off its automatic
trigger:

  wayle config set osd.auto-brightness false"
);

const DEVICE_MATCHING: &str = concat!(
    styled_header!("Device matching:"),
    "
Rules are tried in order: PulseAudio index (audio only, from `pactl list short
sinks`), exact node name, exact description, then a substring of either that
matches one device. A device name that is only digits is always read as an
index, never as a substring.

Run `wayle osd devices` for the node names and descriptions.

  wayle osd speaker \"HDMI Audio\"
  wayle osd speaker alsa_output.pci-0000_00_1f.3.analog-stereo
  wayle osd speaker 3"
);
