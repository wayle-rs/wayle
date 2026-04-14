mod bar;
mod helpers;
mod notification_popup;
mod osd;
pub(crate) mod services;

use std::time::Instant;

use console::style;
use gdk4::Display;
use gtk4::{CssProvider, glib::idle_add_local_once};
use gtk4_layer_shell::{Layer, LayerShell};
use relm4::{gtk, gtk::prelude::*, prelude::*};
pub(crate) use services::ShellServices;
use tracing::{debug, info};

pub(crate) use self::bar::pomodoro::{
    PomodoroMode, PomodoroSharedState, PomodoroSnapshot, SharedPomodoroState, TimerState,
};
use self::{
    notification_popup::{NotificationPopupHost, PopupHostInit},
    osd::{Osd, OsdInit},
};
use crate::{startup::StartupTimer, watchers};

pub(crate) struct Shell {
    css_provider: CssProvider,
    bars: helpers::monitors::BarMap,
    services: ShellServices,
    _notification_popup: Option<Controller<NotificationPopupHost>>,
    _osd: Option<Controller<Osd>>,
}

pub(crate) struct ShellInit {
    pub(crate) timer: StartupTimer,
    pub(crate) services: ShellServices,
}

#[derive(Debug)]
pub(crate) enum ShellInput {
    ReloadCss(String),
}

#[derive(Debug)]
pub(crate) enum ShellCmd {
    CssRecompiled(String),
    LocationChanged,
    OsdEnabledChanged(bool),
    SyncMonitors { expected_count: u32, attempt: u32 },
}

#[relm4::component(pub(crate))]
impl Component for Shell {
    type Init = ShellInit;
    type Input = ShellInput;
    type Output = ();
    type CommandOutput = ShellCmd;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
        }
    }

    #[allow(clippy::expect_used)]
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        init.timer.print_gtk_overhead();
        let start = Instant::now();

        root.init_layer_shell();
        root.set_layer(Layer::Background);
        root.set_default_size(1, 1);
        root.set_visible(false);

        let display = Display::default().expect("No display");

        helpers::init_icons();
        helpers::register_app_actions();
        watchers::init(&sender, &init.services);

        let css_provider = helpers::init_css_provider(&display, &init.services.config);
        let bars = helpers::monitors::create_bars(&init.services);
        helpers::monitors::schedule_deferred_sync_if_needed(bars.len(), &sender);

        let elapsed = start.elapsed();
        eprintln!(
            "{} Shell ({}ms)",
            style("✓").green().bold(),
            elapsed.as_millis()
        );
        info!(elapsed_ms = elapsed.as_millis(), "Shell initialized");

        init.timer.finish();

        let notification_popup = init.services.notification.as_ref().map(|notification| {
            NotificationPopupHost::builder()
                .launch(PopupHostInit {
                    notification: notification.clone(),
                    config: init.services.config.clone(),
                })
                .detach()
        });

        let osd = create_osd(&init.services);

        let model = Shell {
            css_provider,
            bars,
            services: init.services,
            _notification_popup: notification_popup,
            _osd: osd,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: ShellInput, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ShellInput::ReloadCss(css) => {
                self.css_provider.load_from_string(&css);

                for bar in self.bars.values() {
                    let window = bar.widget().clone();

                    idle_add_local_once(move || {
                        trigger_layer_shell_reconfigure(&window);
                    });
                }

                info!("CSS reloaded");
            }
        }
    }

    fn update_cmd(&mut self, msg: ShellCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            ShellCmd::CssRecompiled(css) => {
                sender.input(ShellInput::ReloadCss(css));
            }

            ShellCmd::LocationChanged => {
                self.recreate_bars();
            }

            ShellCmd::OsdEnabledChanged(enabled) => {
                self.toggle_osd(enabled);
            }

            ShellCmd::SyncMonitors {
                expected_count,
                attempt,
            } => {
                helpers::monitors::sync(
                    &mut self.bars,
                    &self.services,
                    expected_count,
                    attempt,
                    |expected, attempt| {
                        helpers::monitors::schedule_retry(expected, attempt, &sender);
                    },
                );
            }
        }
    }
}

impl Shell {
    fn recreate_bars(&mut self) {
        for controller in self.bars.values() {
            controller.widget().destroy();
        }
        self.bars.clear();
        self.bars = helpers::monitors::create_bars(&self.services);
        info!("Bars recreated for location change");
    }

    fn toggle_osd(&mut self, enabled: bool) {
        if enabled && self._osd.is_none() {
            self._osd = create_osd(&self.services);
            debug!("OSD enabled");
        } else if !enabled && let Some(controller) = self._osd.take() {
            controller.widget().destroy();
            debug!("OSD disabled");
        }
    }
}

fn create_osd(services: &ShellServices) -> Option<Controller<Osd>> {
    let osd_enabled = services.config.config().osd.enabled.get();

    if !osd_enabled {
        return None;
    }

    Some(
        Osd::builder()
            .launch(OsdInit {
                config: services.config.clone(),
                audio: services.audio.clone(),
                brightness: services.brightness.clone(),
            })
            .detach(),
    )
}

/// Resets a layer-shell window's cached size so GTK recalculates from content.
fn trigger_layer_shell_reconfigure(window: &gtk::Window) {
    window.set_default_size(1, 1);
    window.set_default_size(0, 0);
}
