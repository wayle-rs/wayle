use std::{sync::Arc, time::Duration};

use gtk4_layer_shell::{Edge, LayerShell};
use relm4::{ComponentSender, gtk, gtk::prelude::*};
use wayle_audio::core::device::{input::InputDevice, output::OutputDevice};
use wayle_brightness::BacklightDevice;
use wayle_config::schemas::osd::{OsdMonitor, OsdPosition};

use super::{
    BRIGHTNESS_ICON, Osd, messages,
    messages::{OsdCmd, OsdEvent},
    watchers,
};
use crate::{
    i18n::t,
    shell::helpers::layer_shell::{
        apply_layer as apply_window_layer, apply_monitor_by_connector, apply_primary_monitor,
        reset_anchors,
    },
};

impl Osd {
    pub(super) fn show_event(
        &mut self,
        event: OsdEvent,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        if !self.ready {
            return;
        }

        self.current_event = Some(event);
        self.dismiss_id = self.dismiss_id.wrapping_add(1);

        root.set_visible(true);

        let duration = self.config.config().osd.duration.get();
        Self::schedule_dismiss(sender, duration, self.dismiss_id);
    }

    pub(super) fn handle_device_changed(
        &mut self,
        device: Option<Arc<OutputDevice>>,
        sender: &ComponentSender<Self>,
    ) {
        let token = self.device_watcher.reset();

        if let Some(device) = &device {
            watchers::spawn_device_watchers(sender, device, token);
        }
    }

    pub(super) fn handle_volume_changed(
        &mut self,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        let Some(audio) = &self.audio else {
            return;
        };

        let Some(device) = audio.default_output.get() else {
            return;
        };

        let percentage = device.volume.get().average_percentage();
        let muted = device.muted.get();
        let rounded = percentage.round() as u32;

        let snapshot = (rounded, muted);

        if self.last_volume == Some(snapshot) {
            return;
        }

        self.last_volume = Some(snapshot);

        let description = device.description.get();
        let icon = volume_icon(percentage, muted);

        let event = OsdEvent::Slider {
            label: description,
            icon: icon.to_string(),
            percentage,
            muted,
        };

        self.show_event(event, sender, root);
    }

    pub(super) fn handle_brightness_device_changed(
        &mut self,
        device: Option<Arc<BacklightDevice>>,
        sender: &ComponentSender<Self>,
    ) {
        let token = self.brightness_watcher.reset();

        if let Some(device) = &device {
            watchers::spawn_brightness_watcher(sender, device, token);
        }
    }

    pub(super) fn handle_brightness_changed(
        &mut self,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        let Some(brightness) = &self.brightness else {
            return;
        };

        let Some(device) = brightness.primary.get() else {
            return;
        };

        let percentage = device.percentage().value();
        let rounded = percentage.round() as u32;

        if self.last_brightness == Some(rounded) {
            return;
        }

        self.last_brightness = Some(rounded);

        let event = OsdEvent::Slider {
            label: device.name.to_string(),
            icon: BRIGHTNESS_ICON.to_string(),
            percentage,
            muted: false,
        };

        self.show_event(event, sender, root);
    }

    pub(super) fn handle_input_device_changed(
        &mut self,
        device: Option<Arc<InputDevice>>,
        sender: &ComponentSender<Self>,
    ) {
        let token = self.input_device_watcher.reset();

        if let Some(device) = &device {
            watchers::spawn_input_device_watchers(sender, device, token);
        }
    }

    pub(super) fn handle_input_volume_changed(
        &mut self,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        let Some(audio) = &self.audio else {
            return;
        };

        let Some(device) = audio.default_input.get() else {
            return;
        };

        let percentage = device.volume.get().average_percentage();
        let muted = device.muted.get();
        let rounded = percentage.round() as u32;

        let snapshot = (rounded, muted);

        if self.last_input_volume == Some(snapshot) {
            return;
        }

        self.last_input_volume = Some(snapshot);

        let description = device.description.get();

        let icon = if muted {
            "ld-mic-off-symbolic"
        } else {
            "ld-mic-symbolic"
        };

        let event = OsdEvent::Slider {
            label: description,
            icon: icon.to_string(),
            percentage,
            muted,
        };

        self.show_event(event, sender, root);
    }

    pub(super) fn handle_toggle_changed(
        &mut self,
        toggle: messages::ToggleEvent,
        sender: &ComponentSender<Self>,
        root: &gtk::Window,
    ) {
        let (label, icon) = match toggle.key {
            messages::ToggleKey::CapsLock => (t!("osd-caps-lock"), "ld-a-large-small-symbolic"),
            messages::ToggleKey::NumLock => (t!("osd-num-lock"), "ld-hash-symbolic"),
            messages::ToggleKey::ScrollLock => (t!("osd-scroll-lock"), "ld-arrow-up-down-symbolic"),
        };

        let event = OsdEvent::Toggle {
            label,
            icon: icon.to_string(),
            active: toggle.active,
        };

        self.show_event(event, sender, root);
    }

    pub(super) fn apply_position(&self, root: &gtk::Window) {
        let config = self.config.config();
        let osd_config = &config.osd;
        let position = osd_config.position.get();
        let scale = config.styling.scale.get().value();
        let margin = (osd_config.margin.get().value() * scale) as i32;

        reset_anchors(root);

        match position {
            OsdPosition::TopLeft => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Top, margin);
                root.set_margin(Edge::Left, margin);
            }

            OsdPosition::Top => {
                root.set_anchor(Edge::Top, true);
                root.set_margin(Edge::Top, margin);
            }

            OsdPosition::TopRight => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Top, margin);
                root.set_margin(Edge::Right, margin);
            }

            OsdPosition::Right => {
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Right, margin);
            }

            OsdPosition::BottomRight => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Right, true);
                root.set_margin(Edge::Bottom, margin);
                root.set_margin(Edge::Right, margin);
            }

            OsdPosition::Bottom => {
                root.set_anchor(Edge::Bottom, true);
                root.set_margin(Edge::Bottom, margin);
            }

            OsdPosition::BottomLeft => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Bottom, margin);
                root.set_margin(Edge::Left, margin);
            }

            OsdPosition::Left => {
                root.set_anchor(Edge::Left, true);
                root.set_margin(Edge::Left, margin);
            }
        }

        let monitor = osd_config.monitor.get();

        match &monitor {
            OsdMonitor::Primary => apply_primary_monitor(root),
            OsdMonitor::Connector(name) => {
                apply_monitor_by_connector(root, name);
            }
        }
    }

    pub(super) fn apply_layer(&self, root: &gtk::Window) {
        let configured = self.config.config().osd.layer.get();
        apply_window_layer(root, configured, &self.config);
    }

    pub(super) fn schedule_dismiss(
        sender: &ComponentSender<Osd>,
        duration_ms: u32,
        dismiss_id: u32,
    ) {
        sender.oneshot_command(async move {
            tokio::time::sleep(Duration::from_millis(duration_ms as u64)).await;
            OsdCmd::Dismiss(dismiss_id)
        });
    }
}

pub(super) fn osd_classes(model: &Osd) -> Vec<&'static str> {
    let mut classes = vec!["osd"];

    if model
        .current_event
        .as_ref()
        .is_some_and(|event| matches!(event, OsdEvent::Slider { muted: true, .. }))
    {
        classes.push("muted");
    }

    if model
        .current_event
        .as_ref()
        .is_some_and(|event| matches!(event, OsdEvent::Toggle { active: false, .. }))
    {
        classes.push("toggle-off");
    }

    if model.config.config().osd.border.get() {
        classes.push("bordered");
    }

    classes
}

pub(super) fn is_slider(event: &Option<OsdEvent>) -> bool {
    event
        .as_ref()
        .is_some_and(|event| matches!(event, OsdEvent::Slider { .. }))
}

pub(super) fn is_toggle(event: &Option<OsdEvent>) -> bool {
    event
        .as_ref()
        .is_some_and(|event| matches!(event, OsdEvent::Toggle { .. }))
}

pub(super) fn event_icon(event: &Option<OsdEvent>) -> Option<&str> {
    match event {
        Some(OsdEvent::Slider { icon, .. }) | Some(OsdEvent::Toggle { icon, .. }) => {
            Some(icon.as_str())
        }
        None => None,
    }
}

pub(super) fn event_slider_label(event: &Option<OsdEvent>) -> String {
    match event {
        Some(OsdEvent::Slider { label, .. }) => label.clone(),
        _ => String::new(),
    }
}

pub(super) fn event_label(event: &Option<OsdEvent>) -> String {
    match event {
        Some(OsdEvent::Slider { label, .. }) => label.clone(),

        Some(OsdEvent::Toggle {
            label,
            active: true,
            ..
        }) => t!("osd-toggle-on", label = label.clone()),

        Some(OsdEvent::Toggle {
            label,
            active: false,
            ..
        }) => t!("osd-toggle-off", label = label.clone()),

        None => String::new(),
    }
}

pub(super) fn event_value(event: &Option<OsdEvent>) -> String {
    match event {
        Some(OsdEvent::Slider { percentage, .. }) => format!("{}%", percentage.round() as u32),
        _ => String::new(),
    }
}

pub(super) fn event_fraction(event: &Option<OsdEvent>) -> f64 {
    match event {
        Some(OsdEvent::Slider { percentage, .. }) => (*percentage / 100.0).clamp(0.0, 1.0),
        _ => 0.0,
    }
}

fn volume_icon(percentage: f64, muted: bool) -> &'static str {
    if muted || percentage <= 0.0 {
        "ld-volume-x-symbolic"
    } else if percentage < 34.0 {
        "ld-volume-symbolic"
    } else if percentage < 67.0 {
        "ld-volume-1-symbolic"
    } else {
        "ld-volume-2-symbolic"
    }
}
