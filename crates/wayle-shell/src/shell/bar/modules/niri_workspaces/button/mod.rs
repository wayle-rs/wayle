//! Per-workspace button used by the [`NiriWorkspaces`] factory.
//!
//! [`NiriWorkspaces`]: super::NiriWorkspaces

mod methods;

use std::{collections::HashSet, mem};

use gtk::prelude::*;
use relm4::{factory::FactoryComponent, prelude::*};
use wayle_config::schemas::modules::{ActiveIndicator, DisplayMode, UrgentMode};

use self::methods::compute_css_classes;

/// Initial input describing one app icon to render inside a workspace button.
///
/// `window_ids` lists the niri window ids this icon represents — usually one,
/// or multiple when [`app-icons-dedupe`] is on.
///
/// [`app-icons-dedupe`]: wayle_config::schemas::modules::NiriWorkspacesConfig::app_icons_dedupe
#[derive(Debug, Clone)]
pub(crate) struct AppIconInit {
    pub icon_name: String,
    pub window_ids: Vec<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct NiriWorkspaceButtonInit {
    pub id: u64,
    pub name: Option<String>,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_urgent: bool,
    pub has_windows: bool,
    pub is_vertical: bool,
    pub display_mode: DisplayMode,
    pub active_indicator: ActiveIndicator,
    pub urgent_show: bool,
    pub urgent_mode: UrgentMode,

    pub show_app_icons: bool,
    pub app_icons: Vec<AppIconInit>,
    pub urgent_window_ids: HashSet<u64>,
    pub divider: String,
    pub icon_gap_px: i32,
    pub empty_icon: String,
}

pub(crate) struct NiriWorkspaceButton {
    pub(super) id: u64,
    pub(super) label: Option<String>,
    pub(super) icon: Option<String>,
    pub(super) is_vertical: bool,
    pub(super) display_mode: DisplayMode,
    pub(super) classes: Vec<String>,

    pub(super) show_app_icons: bool,
    pub(super) divider: String,
    pub(super) icon_gap_px: i32,
    pub(super) empty_icon: String,
    app_icon_inits: Vec<AppIconInit>,
    initial_urgent_window_ids: HashSet<u64>,
}

#[derive(Debug)]
pub(crate) enum NiriWorkspaceButtonInput {}

#[derive(Debug)]
pub(crate) enum NiriWorkspaceButtonOutput {
    LeftClick(u64),
    MiddleClick(u64),
    RightClick(u64),
    ScrollUp,
    ScrollDown,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for NiriWorkspaceButton {
    type Init = NiriWorkspaceButtonInit;
    type Input = NiriWorkspaceButtonInput;
    type Output = NiriWorkspaceButtonOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            set_css_classes: &self.classes.iter().map(String::as_str).collect::<Vec<_>>(),

            connect_clicked[sender, id = self.id] => move |_| {
                let _ = sender.output(NiriWorkspaceButtonOutput::LeftClick(id));
            },

            gtk::Box {
                add_css_class: "workspace-content",
                #[watch]
                set_orientation: self.orientation(),
                #[watch]
                set_halign: self.content_halign(),
                #[watch]
                set_valign: self.content_valign(),

                #[name = "identity_row"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    #[watch]
                    set_visible: self.show_identity_row(),

                    gtk::Label {
                        add_css_class: "workspace-label",
                        #[watch]
                        set_visible: self.show_label(),
                        #[watch]
                        set_label: self.label_text(),
                        set_valign: gtk::Align::Center,
                    },

                    gtk::Image {
                        add_css_class: "workspace-icon",
                        #[watch]
                        set_visible: self.show_icon(),
                        #[watch]
                        set_icon_name: self.icon.as_deref(),
                        set_valign: gtk::Align::Center,
                    },

                    #[name = "divider"]
                    gtk::Label {
                        add_css_class: "workspace-divider",
                        #[watch]
                        set_visible: self.show_divider(),
                        #[watch]
                        set_label: &self.divider,
                        set_valign: gtk::Align::Center,
                    },
                },

                #[name = "app_icons_container"]
                gtk::Box {
                    add_css_class: "workspace-icons",
                    #[watch]
                    set_visible: self.show_app_icons,
                    #[watch]
                    set_orientation: self.orientation(),
                    #[watch]
                    set_spacing: self.icon_gap_px,
                    #[watch]
                    set_halign: self.icons_halign(),
                    set_valign: gtk::Align::Fill,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let classes = compute_css_classes(&init);
        Self {
            id: init.id,
            label: init.label,
            icon: init.icon,
            is_vertical: init.is_vertical,
            display_mode: init.display_mode,
            classes,
            show_app_icons: init.show_app_icons,
            divider: init.divider,
            icon_gap_px: init.icon_gap_px,
            empty_icon: init.empty_icon,
            app_icon_inits: init.app_icons,
            initial_urgent_window_ids: init.urgent_window_ids,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        attach_middle_click(&root, &sender, self.id);
        attach_right_click(&root, &sender, self.id);
        attach_scroll(&root, &sender);

        let urgent = mem::take(&mut self.initial_urgent_window_ids);
        self.populate_app_icons(&widgets.app_icons_container, &urgent);

        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {}
    }
}

fn attach_middle_click(button: &gtk::Button, sender: &FactorySender<NiriWorkspaceButton>, id: u64) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_MIDDLE);
    let sender = sender.clone();
    gesture.connect_released(move |_, _, _, _| {
        let _ = sender.output(NiriWorkspaceButtonOutput::MiddleClick(id));
    });
    button.add_controller(gesture);
}

fn attach_right_click(button: &gtk::Button, sender: &FactorySender<NiriWorkspaceButton>, id: u64) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
    let sender = sender.clone();
    gesture.connect_released(move |_, _, _, _| {
        let _ = sender.output(NiriWorkspaceButtonOutput::RightClick(id));
    });
    button.add_controller(gesture);
}

fn attach_scroll(button: &gtk::Button, sender: &FactorySender<NiriWorkspaceButton>) {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
    );
    let sender = sender.clone();
    controller.connect_scroll(move |_, _dx, dy| {
        if dy > 0.0 {
            let _ = sender.output(NiriWorkspaceButtonOutput::ScrollDown);
        } else if dy < 0.0 {
            let _ = sender.output(NiriWorkspaceButtonOutput::ScrollUp);
        }
        gtk::glib::Propagation::Stop
    });
    button.add_controller(controller);
}
