//! One row per config field. Shows label, source badge, reset button,
//! and a control slot for the widget.

use relm4::{
    gtk,
    gtk::{pango, prelude::*},
    prelude::*,
};
use wayle_config::ValueSource;
use wayle_i18n::{t, t_attr};

use crate::{
    editors::WatcherHandle,
    pages::spec::{Keepalive, SettingRowInit},
    property_handle::PropertyHandle,
};

const DESCRIPTION_MAX_CHARS: i32 = 60;
const DESCRIPTION_TOOLTIP_THRESHOLD: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowBehavior {
    /// Bound to a config value. Shows the source badge and reset button
    /// when a runtime override is present.
    Setting,
    /// One-shot trigger with no value to track, like "Apply theme preset".
    /// No source badge, no reset.
    Action,
}

pub(crate) struct SettingRow {
    handle: PropertyHandle,
    label: String,
    description: String,
    source: ValueSource,
    source_label: String,
    source_tooltip: String,
    config_matches_default: bool,
    behavior: RowBehavior,
    #[allow(dead_code)]
    keepalive: Keepalive,
    _watcher: Option<WatcherHandle>,
}

#[derive(Debug)]
pub(crate) enum SettingRowMsg {
    ClearOverride,
    Refresh,
}

#[relm4::component(pub(crate))]
impl SimpleComponent for SettingRow {
    type Init = SettingRowInit;
    type Input = SettingRowMsg;
    type Output = ();

    view! {
        gtk::Box {
            add_css_class: "setting-row",
            set_orientation: gtk::Orientation::Horizontal,
            set_vexpand: false,

            #[name = "info"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_valign: gtk::Align::Center,

                #[name = "label_row"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    #[name = "row_label"]
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        add_css_class: "setting-label",
                        #[watch]
                        set_label: &model.label,
                    },

                    #[name = "source_badge"]
                    gtk::Label {
                        add_css_class: "badge-subtle",
                        set_hexpand: false,
                        set_vexpand: false,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_css_classes: &["badge-subtle", model.source_css_class()],
                        #[watch]
                        set_label: &model.source_label,
                        #[watch]
                        set_tooltip_text: Some(&model.source_tooltip),
                        #[watch]
                        set_visible: model.has_source_badge(),
                    },
                },

                #[name = "row_description"]
                gtk::Label {
                    set_halign: gtk::Align::Start,
                    add_css_class: "setting-description",
                    set_ellipsize: pango::EllipsizeMode::End,
                    set_max_width_chars: DESCRIPTION_MAX_CHARS,
                    set_single_line_mode: true,
                    #[watch]
                    set_label: &model.description,
                    #[watch]
                    set_visible: !model.description.is_empty(),
                    #[watch]
                    set_tooltip_text: if model.description.len() > DESCRIPTION_TOOLTIP_THRESHOLD {
                        Some(&model.description)
                    } else {
                        None
                    },
                },
            },

            #[name = "reset_button"]
            gtk::Button {
                add_css_class: "setting-reset",
                set_icon_name: "ld-rotate-ccw-symbolic",
                set_valign: gtk::Align::Center,
                set_hexpand: false,
                set_cursor_from_name: Some("pointer"),
                #[watch]
                set_visible: model.has_runtime_override(),
                connect_clicked => SettingRowMsg::ClearOverride,
            },

            #[name = "control_slot"]
            gtk::Box {
                add_css_class: "setting-control",
                set_valign: gtk::Align::Center,
                set_hexpand: false,
            },
        }
    }

    fn init(
        mut init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let i18n_key = init.i18n_key.unwrap_or("missing-i18n-key");
        let label = t(i18n_key);
        let description = t_attr(i18n_key, "description");

        let watch_fn = init.handle.watch_changes.take();

        let mut model = Self {
            handle: init.handle,
            label,
            description,
            source: ValueSource::Default,
            source_label: String::new(),
            source_tooltip: String::new(),
            behavior: init.behavior,
            config_matches_default: true,
            keepalive: init.keepalive,
            _watcher: None,
        };

        model.update_source_info();

        let widgets = view_output!();

        if let Some(badge) = init.dirty_badge {
            widgets.label_row.append(&badge);
        }

        apply_control_layout(&init.control, &root, &widgets.control_slot, init.full_width);
        widgets.control_slot.append(&init.control);

        if init.full_width {
            root.remove(&widgets.reset_button);
            widgets.label_row.append(&widgets.reset_button);
        }

        if let Some(watch) = watch_fn {
            let sender = sender.input_sender().clone();
            model._watcher = Some(watch(Box::new(move || {
                sender.send(SettingRowMsg::Refresh).is_ok()
            })));
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SettingRowMsg::ClearOverride => {
                self.handle.clear_runtime();
                self.update_source_info();
            }

            SettingRowMsg::Refresh => {
                self.update_source_info();
            }
        }
    }
}

fn apply_control_layout(control: &gtk::Widget, root: &gtk::Box, slot: &gtk::Box, full_width: bool) {
    if full_width {
        control.set_hexpand(true);
        root.set_orientation(gtk::Orientation::Vertical);
        root.add_css_class("vertical");
        slot.set_hexpand(true);
        return;
    }

    control.set_hexpand(false);
    control.set_valign(gtk::Align::Center);
}

impl SettingRow {
    fn has_runtime_override(&self) -> bool {
        self.behavior == RowBehavior::Setting
            && matches!(
                self.source,
                ValueSource::RuntimeOnly | ValueSource::Overridden
            )
    }

    fn has_source_badge(&self) -> bool {
        self.behavior == RowBehavior::Setting
            && self.source != ValueSource::Default
            && !self.config_matches_default
    }

    fn source_css_class(&self) -> &'static str {
        match self.source {
            ValueSource::Default => "",
            ValueSource::Config => "info",
            ValueSource::RuntimeOnly => "success",
            ValueSource::Overridden => "warning",
        }
    }

    fn update_source_info(&mut self) {
        self.source = self.handle.source();

        self.source_label = match self.source {
            ValueSource::Default => String::new(),
            ValueSource::Config => t("settings-source-config"),
            ValueSource::RuntimeOnly => t("settings-source-custom"),
            ValueSource::Overridden => t("settings-source-override"),
        };

        self.source_tooltip = match self.source {
            ValueSource::Default => String::new(),
            ValueSource::Config => t_attr("settings-source-config", "description"),
            ValueSource::RuntimeOnly => t_attr("settings-source-custom", "description"),
            ValueSource::Overridden => t_attr("settings-source-override", "description"),
        };

        self.config_matches_default = self.source == ValueSource::Config
            && self.handle.config_display() == Some(self.handle.default_display());
    }
}
