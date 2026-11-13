use std::sync::Arc;

use futures::stream::{BoxStream, SelectAll, StreamExt, select_all};
use relm4::ComponentSender;
use wayle_audio::{AudioService, core::device::output::OutputDevice, types::device::DevicePort};
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{
    DevicePicker,
    messages::DevicePickerCmd,
    methods::{build_input_device_list, build_output_device_list, visible_output_devices},
};
use crate::shell::bar::dropdowns::audio::VolumeSectionKind;

pub(super) fn spawn(
    sender: &ComponentSender<DevicePicker>,
    audio: &Arc<AudioService>,
    config: &Arc<ConfigService>,
    kind: VolumeSectionKind,
) {
    match kind {
        VolumeSectionKind::Output => spawn_output(sender, audio, config),
        VolumeSectionKind::Input => {
            let input_devices = audio.input_devices.clone();
            let default_input = audio.default_input.clone();
            watch!(
                sender,
                [input_devices.watch(), default_input.watch()],
                |out| {
                    let list = build_input_device_list(&input_devices.get(), &default_input.get());
                    let _ = out.send(DevicePickerCmd::DevicesChanged(list));
                }
            );
        }
    }
}

fn spawn_output(
    sender: &ComponentSender<DevicePicker>,
    audio: &Arc<AudioService>,
    config: &Arc<ConfigService>,
) {
    let output_devices = audio.output_devices.clone();
    let default_output = audio.default_output.clone();
    let hide_setting = config
        .config()
        .modules
        .volume
        .dropdown_hide_unavailable_outputs
        .clone();

    sender.command(move |out, shutdown| async move {
        let mut devices_changed = output_devices.watch().boxed();
        let mut default_changed = default_output.watch().boxed();
        let mut setting_changed = hide_setting.watch().skip(1).boxed();

        let mut hide_unavailable = hide_setting.get();
        let mut ports_changed = port_changes(&output_devices.get(), hide_unavailable);

        let watching = async {
            loop {
                tokio::select! {
                    Some(_) = devices_changed.next() => {
                        ports_changed = port_changes(&output_devices.get(), hide_unavailable);
                    }
                    Some(_) = setting_changed.next() => {
                        hide_unavailable = hide_setting.get();
                        ports_changed = port_changes(&output_devices.get(), hide_unavailable);
                    }
                    Some(_) = default_changed.next() => {}
                    Some(_) = ports_changed.next() => {}
                    else => return,
                }

                let list = build_output_device_list(
                    &visible_output_devices(&output_devices.get(), hide_unavailable),
                    &default_output.get(),
                );
                let _ = out.send(DevicePickerCmd::DevicesChanged(list));
            }
        };

        tokio::select! {
            () = shutdown.wait() => {}
            () = watching => {}
        }
    });
}

/// Changes to the ports of every output.
///
/// A port's availability can change without the list of outputs changing, so
/// it has to be watched separately. This is only needed when unavailable
/// outputs are hidden; otherwise the stream never yields.
fn port_changes(
    devices: &[Arc<OutputDevice>],
    hide_unavailable: bool,
) -> SelectAll<BoxStream<'static, Vec<DevicePort>>> {
    let streams = if hide_unavailable {
        devices
            .iter()
            .map(|device| device.ports.watch().skip(1).boxed())
            .collect()
    } else {
        Vec::new()
    };
    select_all(streams)
}
