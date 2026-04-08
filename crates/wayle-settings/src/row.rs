//! One row per config field. Shows label, source badge, reset button,
//! and a control slot for the widget.

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ValueSource;
use wayle_i18n::{t, t_attr};

use crate::property_handle::PropertyHandle;

pub struct SettingRowInit {
    pub i18n_key: &'static str,
    pub handle: PropertyHandle,
    pub control_widget: Option<gtk4::Widget>,
}

pub struct SettingRow {
    handle: PropertyHandle,
    label: String,
    description: String,
    source: ValueSource,
    source_label: String,
    source_tooltip: String,
    config_matches_default: bool,
    dirty: bool,
}

#[derive(Debug)]
pub enum SettingRowMsg {
    ClearOverride,
    Refresh,
}

impl SettingRow {
    fn has_runtime_override(&self) -> bool {
        matches!(
            self.source,
            ValueSource::RuntimeOnly | ValueSource::Overridden
        )
    }

    fn has_source_badge(&self) -> bool {
        self.source != ValueSource::Default && !self.config_matches_default
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

#[relm4::component(pub)]
impl SimpleComponent for SettingRow {
    type Init = SettingRowInit;
    type Input = SettingRowMsg;
    type Output = ();

    view! {
        gtk4::Box {
            add_css_class: "setting-row",
            set_orientation: gtk4::Orientation::Horizontal,
            set_vexpand: false,

            #[name = "info"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
                set_hexpand: true,
                set_valign: gtk4::Align::Center,

                gtk4::Box {
                    set_orientation: gtk4::Orientation::Horizontal,

                    #[name = "row_label"]
                    gtk4::Label {
                        set_halign: gtk4::Align::Start,
                        add_css_class: "setting-label",
                        #[watch]
                        set_label: &model.label,
                    },

                    #[name = "source_badge"]
                    gtk4::Label {
                        add_css_class: "badge-subtle",
                        set_hexpand: false,
                        set_vexpand: false,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Start,
                        #[watch]
                        set_css_classes: &["badge-subtle", model.source_css_class()],
                        #[watch]
                        set_label: &model.source_label,
                        #[watch]
                        set_tooltip_text: Some(&model.source_tooltip),
                        #[watch]
                        set_visible: model.has_source_badge(),
                    },

                    #[name = "dirty_badge"]
                    gtk4::Label {
                        add_css_class: "badge-subtle",
                        add_css_class: "warning",
                        set_hexpand: false,
                        set_vexpand: false,
                        set_valign: gtk4::Align::Center,
                        set_halign: gtk4::Align::Start,
                        #[watch]
                        set_visible: model.dirty,
                    },
                },

                #[name = "row_description"]
                gtk4::Label {
                    set_halign: gtk4::Align::Start,
                    add_css_class: "setting-description",
                    #[watch]
                    set_label: &model.description,
                    #[watch]
                    set_visible: !model.description.is_empty(),
                },
            },

            #[name = "reset_button"]
            gtk4::Button {
                add_css_class: "setting-reset",
                set_icon_name: "ld-rotate-ccw-symbolic",
                set_valign: gtk4::Align::Center,
                set_hexpand: false,
                #[watch]
                set_visible: model.has_runtime_override(),
                connect_clicked => SettingRowMsg::ClearOverride,
            },

            #[name = "control_slot"]
            gtk4::Box {
                add_css_class: "setting-control",
                set_valign: gtk4::Align::Center,
                set_hexpand: false,
            },
        }
    }

    fn init(
        mut init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let label = t(init.i18n_key);
        let description = t_attr(init.i18n_key, "description");

        let watcher = init.handle.watch_changes.take();

        let mut model = Self {
            handle: init.handle,
            label,
            description,
            source: ValueSource::Default,
            source_label: String::new(),
            source_tooltip: String::new(),
            config_matches_default: true,
            dirty: false,
        };

        model.update_source_info();

        let widgets = view_output!();

        if let Some(control) = init.control_widget {
            control.set_hexpand(false);
            control.set_valign(gtk4::Align::Center);
            widgets.control_slot.append(&control);
        }

        widgets.dirty_badge.set_label(&t("settings-source-unsaved"));
        widgets
            .dirty_badge
            .set_tooltip_text(Some(&t_attr("settings-source-unsaved", "description")));

        if let Some(watch) = watcher {
            let sender = sender.input_sender().clone();
            watch(Box::new(move || {
                let _ = sender.send(SettingRowMsg::Refresh);
            }));
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SettingRowMsg::ClearOverride => {
                self.handle.clear_runtime();
                self.dirty = false;
                self.update_source_info();
            }

            SettingRowMsg::Refresh => {
                self.dirty = false;
                self.update_source_info();
            }
        }
    }
}
