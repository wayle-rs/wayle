pub(crate) mod messages;
mod methods;
mod toggles;
mod watchers;

use std::{sync::Arc, time::Duration};

use gtk::{pango::EllipsizeMode, prelude::*};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use relm4::{gtk, prelude::*};
use tracing::debug;
use wayle_audio::AudioService;
use wayle_brightness::BrightnessService;
use wayle_config::ConfigService;
use wayle_widgets::WatcherToken;

pub(crate) use self::messages::OsdInit;
use self::{
    messages::{OsdCmd, OsdEvent},
    methods::{
        event_fraction, event_icon, event_label, event_slider_label, event_value, is_slider,
        is_toggle, osd_classes,
    },
};
use crate::services::shell_ipc::OsdDevice;

const BRIGHTNESS_ICON: &str = "ld-sun-symbolic";

pub(crate) struct Osd {
    config: Arc<ConfigService>,
    audio: Option<Arc<AudioService>>,
    brightness: Option<Arc<BrightnessService>>,
    dismiss_id: u32,
    ready: bool,
    device_watcher: WatcherToken,
    input_device_watcher: WatcherToken,
    brightness_watcher: WatcherToken,

    /// Device the visible overlay is reporting on, watched while it stays up.
    displayed: Option<OsdDevice>,
    displayed_watcher: WatcherToken,

    current_event: Option<OsdEvent>,
    last_volume: Option<(u32, bool)>,
    last_input_volume: Option<(u32, bool)>,
    last_brightness: Option<u32>,

    /// Highest IPC request sequence already handled.
    seen_seq: u64,
}

#[allow(clippy::needless_borrow)]
#[relm4::component(pub(crate))]
impl Component for Osd {
    type Init = OsdInit;
    type Input = ();
    type Output = ();
    type CommandOutput = OsdCmd;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
            add_css_class: "osd-host",
            set_default_size: (1, 1),
            set_visible: false,

            #[name = "osd_container"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[watch]
                set_css_classes: &osd_classes(&model),

                #[name = "slider_header"]
                gtk::Box {
                    add_css_class: "osd-header",

                    #[watch]
                    set_visible: is_slider(&model.current_event),

                    #[name = "slider_icon"]
                    gtk::Image {
                        add_css_class: "osd-icon",
                        set_valign: gtk::Align::Center,

                        #[watch]
                        set_icon_name: event_icon(&model.current_event),
                    },

                    #[name = "slider_label"]
                    gtk::Label {
                        add_css_class: "osd-label",
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        set_ellipsize: EllipsizeMode::End,

                        #[watch]
                        set_label: &event_slider_label(&model.current_event),
                    },

                    #[name = "value"]
                    gtk::Label {
                        add_css_class: "osd-value",
                        set_valign: gtk::Align::Center,

                        #[watch]
                        set_label: &event_value(&model.current_event),
                    },
                },

                #[name = "toggle_header"]
                gtk::Box {
                    add_css_class: "osd-header",

                    #[watch]
                    set_visible: is_toggle(&model.current_event),

                    #[name = "toggle_icon"]
                    gtk::Image {
                        add_css_class: "osd-icon",
                        set_valign: gtk::Align::Center,

                        #[watch]
                        set_icon_name: event_icon(&model.current_event),
                    },

                    #[name = "toggle_label"]
                    gtk::Label {
                        add_css_class: "osd-label",
                        set_valign: gtk::Align::Center,

                        #[watch]
                        set_label: &event_label(&model.current_event),
                    },
                },

                #[name = "bar"]
                gtk::ProgressBar {
                    add_css_class: "osd-bar",

                    #[watch]
                    set_fraction: event_fraction(&model.current_event),

                    #[watch]
                    set_visible: is_slider(&model.current_event),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_namespace(Some("wayle-osd"));

        let model = Self {
            config: init.config.clone(),
            audio: init.audio.clone(),
            brightness: init.brightness.clone(),
            dismiss_id: 0,
            ready: false,
            device_watcher: WatcherToken::new(),
            input_device_watcher: WatcherToken::new(),
            brightness_watcher: WatcherToken::new(),
            displayed: None,
            displayed_watcher: WatcherToken::new(),
            current_event: None,
            last_volume: None,
            last_input_volume: None,
            last_brightness: None,
            seen_seq: init.seen_seq,
        };

        model.apply_position(&root);
        model.apply_layer(&root);

        sender.oneshot_command(async {
            tokio::time::sleep(Duration::from_millis(500)).await;
            OsdCmd::Ready
        });

        let widgets = view_output!();

        watchers::spawn(
            &sender,
            &init.config,
            &init.audio,
            &init.brightness,
            &init.shell_ipc,
        );

        ComponentParts { model, widgets }
    }

    fn update_cmd(&mut self, msg: OsdCmd, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            OsdCmd::Ready => {
                self.ready = true;
            }

            OsdCmd::Dismiss(dismiss_id) => {
                if dismiss_id == self.dismiss_id {
                    root.set_visible(false);
                    self.clear_displayed();
                    debug!("OSD dismissed");
                }
            }

            OsdCmd::ConfigChanged => {
                self.apply_position(root);
                self.apply_layer(root);
            }

            OsdCmd::DeviceChanged(device) => {
                self.handle_device_changed(device, &sender);
            }

            OsdCmd::VolumeChanged => {
                self.handle_volume_changed(&sender, root);
            }

            OsdCmd::BrightnessDeviceChanged(device) => {
                self.handle_brightness_device_changed(device, &sender);
            }

            OsdCmd::BrightnessChanged => {
                self.handle_brightness_changed(&sender, root);
            }

            OsdCmd::InputDeviceChanged(device) => {
                self.handle_input_device_changed(device, &sender);
            }

            OsdCmd::InputVolumeChanged => {
                self.handle_input_volume_changed(&sender, root);
            }

            OsdCmd::ToggleChanged(toggle) => {
                self.handle_toggle_changed(toggle, &sender, root);
            }

            OsdCmd::ShowRequested(request) => {
                self.handle_show_requested(request, &sender, root);
            }

            OsdCmd::DisplayedValueChanged => {
                self.handle_displayed_value_changed();
            }
        }
    }
}
