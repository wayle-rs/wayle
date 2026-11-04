pub(crate) mod messages;
mod methods;

use gtk::{pango, prelude::*};
use relm4::{gtk, prelude::*};
use wayle_widgets::prelude::*;

use self::messages::{DeviceItemInit, DeviceItemInput};
pub(crate) use self::methods::ActionsLayout;
use crate::{
    i18n::t,
    shell::bar::dropdowns::bluetooth::helpers::{DeviceSnapshot, battery_level_icon},
};

const DETAIL_SEPARATOR: &str = "\u{2022}";
const HOVER_TRANSITION_MS: u32 = 150;

pub(crate) struct DeviceItem {
    /// What the row shows, and the device its actions act on.
    snapshot: DeviceSnapshot,
    hovered: bool,
    /// This row's position in its list, kept current by the factory; lets the
    /// parent locate a row in O(1) while reordering.
    pub(crate) index: DynamicIndex,
    /// Guards the row (and its buttons) after it moves under the pointer; set
    /// once the row's widget exists.
    click_guard: Option<ClickGuard>,
    /// Guards just the buttons, which take Dismiss's place when an error is
    /// dismissed (see [`ActionsLayout::only_dismissed`]).
    actions_guard: Option<ClickGuard>,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for DeviceItem {
    type Init = DeviceItemInit;
    type Input = DeviceItemInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            add_css_class: "bluetooth-device",
            #[watch]
            set_cursor_from_name: self.row_clickable().then_some("pointer"),
            #[watch]
            set_css_classes: &self.root_css_classes(),

            #[name = "icon_container"]
            gtk::Box {
                #[watch]
                set_css_classes: &self.icon_css_classes(),
                set_hexpand: false,

                #[name = "device_icon"]
                gtk::Image {
                    add_css_class: "bluetooth-icon",
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_icon_name: Some(self.snapshot.icon),
                },
            },

            #[name = "info_column"]
            gtk::Box {
                add_css_class: "bluetooth-device-info",
                set_orientation:
                    gtk::Orientation::Vertical,
                set_hexpand: true,
                set_valign: gtk::Align::Center,

                #[name = "device_name"]
                gtk::Label {
                    add_css_class:
                        "bluetooth-device-name",
                    set_halign: gtk::Align::Start,
                    set_ellipsize:
                        pango::EllipsizeMode::End,
                    #[watch]
                    set_label: &self.snapshot.name,
                },

                #[name = "detail_row"]
                gtk::Box {
                    add_css_class:
                        "bluetooth-device-detail-row",
                    set_halign: gtk::Align::Start,

                    #[name = "device_type_label"]
                    gtk::Label {
                        #[watch]
                        set_css_classes: &self.detail_css_classes(),
                        #[watch]
                        set_label: &self.detail_text(),
                        #[watch]
                        set_tooltip_text: self.error_tooltip(),
                    },

                    #[name = "battery_separator"]
                    gtk::Label {
                        add_css_class:
                            "bluetooth-detail-separator",
                        set_label: DETAIL_SEPARATOR,
                        #[watch]
                        set_visible: self.snapshot.battery.is_some(),
                    },

                    #[name = "battery_icon"]
                    gtk::Image {
                        add_css_class:
                            "bluetooth-battery-icon",
                        #[watch]
                        set_visible: self.snapshot.battery.is_some(),
                        #[watch]
                        set_icon_name: self.snapshot.battery.map(battery_level_icon),
                    },

                    #[name = "battery_label"]
                    gtk::Label {
                        add_css_class:
                            "bluetooth-device-detail",
                        #[watch]
                        set_visible: self.snapshot.battery.is_some(),
                        #[watch]
                        set_label: &self.battery_text(),
                    },
                },
            },

            gtk::Stack {
                add_css_class: "bluetooth-hover-stack",
                set_transition_type:
                    gtk::StackTransitionType::Crossfade,
                set_transition_duration: HOVER_TRANSITION_MS,
                set_valign: gtk::Align::Center,
                set_hexpand: false,
                #[watch]
                set_visible: self.is_my_device()
                    || self.is_busy()
                    || self.shown_error().is_some(),
                add_named[Some("status")] = &gtk::Box {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    // Settled states are plain text; in-progress and error
                    // states are a badge, as in the network dropdowns.
                    gtk::Label {
                        add_css_class: "bluetooth-device-status",
                        set_vexpand: false,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_label: &self.status_label(),
                        #[watch]
                        set_visible: self.status_visible() && self.status_badge().is_none(),
                    },

                    #[template]
                    SubtleBadge {
                        #[watch]
                        set_css_classes: &self.status_badge_css_classes(),
                        #[watch]
                        set_label: &self.status_label(),
                        set_vexpand: false,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_visible: self.status_badge().is_some(),
                    },
                },

                add_named[Some("error-actions")] = &gtk::Box {
                    add_css_class:
                        "bluetooth-device-actions",
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    // A failure can leave the device connected (e.g. a
                    // connect whose profiles the device then connected
                    // itself), and it can still be disconnected.
                    #[template]
                    GhostButton {
                        add_css_class:
                            "bluetooth-action-toggle",
                        #[watch]
                        set_visible: self.toggle_visible(),
                        #[template_child]
                        label {
                            #[watch]
                            set_label: &self.toggle_label(),
                        },
                        connect_clicked =>
                            DeviceItemInput::ToggleClicked,
                    },

                    #[template]
                    GhostButton {
                        add_css_class:
                            "bluetooth-action-dismiss",
                        #[template_child]
                        label {
                            set_label: &t!(
                                "dropdown-bluetooth-dismiss"
                            ),
                        },
                        connect_clicked =>
                            DeviceItemInput::DismissClicked,
                    },
                },

                #[name = "actions_box"]
                add_named[Some("actions")] = &gtk::Box {
                    add_css_class:
                        "bluetooth-device-actions",
                    // The stack sizes its pages to the widest one (e.g. the
                    // "Connecting…" status); keep the buttons at the right.
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,

                    #[template]
                    GhostButton {
                        add_css_class:
                            "bluetooth-action-toggle",
                        #[watch]
                        set_visible: self.toggle_visible(),
                        #[template_child]
                        label {
                            #[watch]
                            set_label: &self.toggle_label(),
                        },
                        connect_clicked =>
                            DeviceItemInput::ToggleClicked,
                    },

                    #[template]
                    GhostButton {
                        add_css_class:
                            "bluetooth-forget",
                        #[watch]
                        set_visible: self.forget_visible(),
                        #[template_child]
                        label {
                            set_label: &t!(
                                "dropdown-bluetooth-forget"
                            ),
                        },
                        connect_clicked =>
                            DeviceItemInput::ForgetClicked,
                    },
                },

                #[watch]
                set_visible_child_name: self.hover_page(),
            },
        }
    }

    fn init_model(init: Self::Init, index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            snapshot: init.snapshot,
            hovered: false,
            index: index.clone(),
            click_guard: None,
            actions_guard: None,
        }
    }

    fn update(&mut self, msg: DeviceItemInput, _sender: FactorySender<Self>) {
        // A guarded row can't be targeted by the pointer; this also covers a
        // click that was already queued when the row moved.
        let clicked_row = matches!(
            msg,
            DeviceItemInput::Clicked
                | DeviceItemInput::ToggleClicked
                | DeviceItemInput::ForgetClicked
                | DeviceItemInput::DismissClicked
        );
        let clicked_button = matches!(
            msg,
            DeviceItemInput::ToggleClicked | DeviceItemInput::ForgetClicked
        );
        let armed = |guard: &Option<ClickGuard>| guard.as_ref().is_some_and(ClickGuard::is_armed);
        if (clicked_row && armed(&self.click_guard))
            || (clicked_button && armed(&self.actions_guard))
        {
            return;
        }

        // Actions go straight to the device; their progress and outcome come
        // back through its state (`activity`, `connected`, `last_error`, ...).
        match msg {
            DeviceItemInput::Clicked => self.handle_click(),
            DeviceItemInput::ToggleClicked => self.handle_toggle(),
            DeviceItemInput::Hovered(hovered) => self.hovered = hovered,
            DeviceItemInput::DismissClicked => self.snapshot.device.dismiss_error(),
            DeviceItemInput::ForgetClicked => self.handle_forget(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();
        self.click_guard = Some(ClickGuard::new(&root));
        self.actions_guard = Some(ClickGuard::new(&widgets.actions_box));

        let click = gtk::GestureClick::new();
        let click_sender = sender.input_sender().clone();
        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            click_sender.emit(DeviceItemInput::Clicked);
        });
        root.add_controller(click);

        // Every row tracks hover: available rows show their actions too, to
        // offer Cancel while connecting.
        let hover = gtk::EventControllerMotion::new();
        let hover_sender = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            hover_sender.emit(DeviceItemInput::Hovered(true));
        });
        let leave_sender = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave_sender.emit(DeviceItemInput::Hovered(false));
        });
        root.add_controller(hover);

        widgets
    }
}
