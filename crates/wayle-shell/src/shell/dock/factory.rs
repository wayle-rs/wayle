use gtk::prelude::*;
use relm4::prelude::*;

use super::settings::DockSettings;
use crate::shell::bar::icons::resolve_app_icon;

pub(crate) struct DockItem {
    app_id: String,
    is_pinned: bool,
    is_running: bool,
    is_active: bool,
    settings: DockSettings,
    _root: gtk::Button,
}

#[derive(Clone)]
pub(crate) struct DockItemInit {
    pub(crate) app_id: String,
    pub(crate) is_pinned: bool,
    pub(crate) is_running: bool,
    pub(crate) is_active: bool,
    pub(crate) settings: DockSettings,
}

#[derive(Debug)]
pub(crate) enum DockItemInput {
    Click,
    RightClick,
    HoverEnter,
    HoverLeave,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for DockItem {
    type Init = DockItemInit;
    type Input = DockItemInput;
    type Output = (String, DockItemInput);
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            add_css_class: "dock-item",
            set_focusable: false,

            #[name = "icon_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,
                // set_homogeneous: true,

                #[name = "icon_image"]
                gtk::Image {
                    set_pixel_size: 24,
                    set_halign: gtk::Align::Center,
                },

                #[name = "indicator_revealer"]
                gtk::Revealer {
                    set_reveal_child: false,
                    set_transition_type: gtk::RevealerTransitionType::Crossfade,

                    #[name = "indicator_dot"]
                    gtk::Box {
                        add_css_class: "dock-indicator",
                        set_hexpand: true,
                        set_vexpand: false,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::End,
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            app_id: init.app_id,
            is_pinned: init.is_pinned,
            is_running: init.is_running,
            is_active: init.is_active,
            settings: init.settings,
            _root: gtk::Button::new(),
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

        self._root = root.clone();
        self.update_model(&widgets);

        let sender_clone = sender.clone();
        root.connect_clicked(move |_| {
            sender_clone.input(DockItemInput::Click);
        });

        let right_click = gtk::GestureClick::builder().button(3).build();
        right_click.connect_released({
            let sender = sender.clone();
            move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender.input(DockItemInput::RightClick);
            }
        });
        root.add_controller(right_click);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            DockItemInput::Click => {
                let app_id = self.app_id.clone();
                let _ = sender.output((app_id, DockItemInput::Click));
            }
            DockItemInput::RightClick => {
                let app_id = self.app_id.clone();
                let _ = sender.output((app_id, DockItemInput::RightClick));
            }
            DockItemInput::HoverEnter => {
                let _ = sender.output((self.app_id.clone(), DockItemInput::HoverEnter));
            }
            DockItemInput::HoverLeave => {
                let _ = sender.output((self.app_id.clone(), DockItemInput::HoverLeave));
            }
        }
    }
}

impl DockItem {
    fn update_model(&self, widgets: &<Self as relm4::factory::FactoryComponent>::Widgets) {
        widgets
            .icon_image
            .set_pixel_size(self.settings.size.get() as i32);

        let icon_name = resolve_app_icon(&self.app_id);

        widgets.icon_image.set_icon_name(Some(&icon_name));

        widgets.indicator_revealer.set_reveal_child(self.is_running);

        if self.is_active {
            self._root.add_css_class("active");
        } else {
            self._root.remove_css_class("active");
        }

        if self.is_running {
            self._root.add_css_class("running");
        } else {
            self._root.remove_css_class("running");
        }
    }
}
