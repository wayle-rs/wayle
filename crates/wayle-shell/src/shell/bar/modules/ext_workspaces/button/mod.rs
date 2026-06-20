//! Per-workspace button used by the [`ExtWorkspaces`] factory.
//!
//! [`ExtWorkspaces`]: super::ExtWorkspaces

mod methods;

use gtk::prelude::*;
use relm4::{factory::FactoryComponent, prelude::*};
use wayle_config::schemas::modules::{ActiveIndicator, DisplayMode};

use self::methods::compute_css_classes;

#[derive(Debug, Clone)]
pub(crate) struct ExtWorkspaceButtonInit {
    pub key: u32,
    pub name: Option<String>,
    pub label: Option<String>,
    pub icon: Option<String>,
    pub is_active: bool,
    pub is_urgent: bool,
    pub is_hidden: bool,
    pub is_vertical: bool,
    pub display_mode: DisplayMode,
    pub active_indicator: ActiveIndicator,
}

pub(crate) struct ExtWorkspaceButton {
    pub(super) key: u32,
    pub(super) label: Option<String>,
    pub(super) icon: Option<String>,
    pub(super) is_vertical: bool,
    pub(super) display_mode: DisplayMode,
    pub(super) classes: Vec<String>,
}

#[derive(Debug)]
pub(crate) enum ExtWorkspaceButtonInput {}

#[derive(Debug)]
pub(crate) enum ExtWorkspaceButtonOutput {
    LeftClick(u32),
    MiddleClick(u32),
    RightClick(u32),
    ScrollUp,
    ScrollDown,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for ExtWorkspaceButton {
    type Init = ExtWorkspaceButtonInit;
    type Input = ExtWorkspaceButtonInput;
    type Output = ExtWorkspaceButtonOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            set_css_classes: &self.classes.iter().map(String::as_str).collect::<Vec<_>>(),

            connect_clicked[sender, key = self.key] => move |_| {
                let _ = sender.output(ExtWorkspaceButtonOutput::LeftClick(key));
            },

            gtk::Box {
                add_css_class: "workspace-content",
                #[watch]
                set_orientation: self.orientation(),
                #[watch]
                set_halign: self.content_halign(),
                #[watch]
                set_valign: self.content_valign(),

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
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let classes = compute_css_classes(&init);
        Self {
            key: init.key,
            label: init.label,
            icon: init.icon,
            is_vertical: init.is_vertical,
            display_mode: init.display_mode,
            classes,
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

        attach_middle_click(&root, &sender, self.key);
        attach_right_click(&root, &sender, self.key);
        attach_scroll(&root, &sender);

        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {}
    }
}

fn attach_middle_click(
    button: &gtk::Button,
    sender: &FactorySender<ExtWorkspaceButton>,
    key: u32,
) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_MIDDLE);
    let sender = sender.clone();
    gesture.connect_released(move |_, _, _, _| {
        let _ = sender.output(ExtWorkspaceButtonOutput::MiddleClick(key));
    });
    button.add_controller(gesture);
}

fn attach_right_click(button: &gtk::Button, sender: &FactorySender<ExtWorkspaceButton>, key: u32) {
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::BUTTON_SECONDARY);
    let sender = sender.clone();
    gesture.connect_released(move |_, _, _, _| {
        let _ = sender.output(ExtWorkspaceButtonOutput::RightClick(key));
    });
    button.add_controller(gesture);
}

fn attach_scroll(button: &gtk::Button, sender: &FactorySender<ExtWorkspaceButton>) {
    let controller = gtk::EventControllerScroll::new(
        gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
    );
    let sender = sender.clone();
    controller.connect_scroll(move |_, _dx, dy| {
        if dy > 0.0 {
            let _ = sender.output(ExtWorkspaceButtonOutput::ScrollDown);
        } else if dy < 0.0 {
            let _ = sender.output(ExtWorkspaceButtonOutput::ScrollUp);
        }
        gtk::glib::Propagation::Stop
    });
    button.add_controller(controller);
}
