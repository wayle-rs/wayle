//! Window switcher dropdown: lists every open window, click-to-activate,
//! or Mod1+Tab/Tab to cycle and Return to confirm (see `~/.config/sway/config`
//! for the bindings - a `mode` block is required because Sway never fires a
//! binding for a bare modifier press or release once another key has been
//! pressed while it was held).

mod factory;
mod messages;
mod methods;
mod row;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use wayle_config::ConfigService;
use wayle_widgets::prelude::*;
use wayle_wlr_toplevel::WlrToplevelService;

pub(super) use self::factory::Factory;
use self::{
    messages::{WindowSwitcherDropdownCmd, WindowSwitcherDropdownInit, WindowSwitcherDropdownMsg},
    row::WindowRow,
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 320.0;
const BASE_HEIGHT: f32 = 400.0;

pub(crate) struct WindowSwitcherDropdown {
    service: Arc<WlrToplevelService>,
    config: Arc<ConfigService>,
    scaled_width: i32,
    scaled_height: i32,
    rows: FactoryVecDeque<WindowRow>,
    ordered_keys: Vec<u32>,
    /// Selection driven by Mod+Tab cycling, separate from mouse clicks
    /// (which activate immediately). `None` when not cycling.
    highlighted_index: Option<usize>,
}

#[relm4::component(pub(crate))]
impl Component for WindowSwitcherDropdown {
    type Init = WindowSwitcherDropdownInit;
    type Input = WindowSwitcherDropdownMsg;
    type Output = ();
    type CommandOutput = WindowSwitcherDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "window-switcher-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("ld-app-window-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-window-switcher-title"),
                    },
                },

                #[template]
                DropdownContent {
                    add_css_class: "window-switcher-content",

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,

                        #[local_ref]
                        row_list -> gtk::ListBox {
                            add_css_class: "window-switcher-list",
                            set_activate_on_single_click: true,
                            set_selection_mode: gtk::SelectionMode::None,
                        },
                    },

                    #[name = "empty_state"]
                    #[template]
                    EmptyState {
                        #[watch]
                        set_visible: model.rows.is_empty(),
                        #[template_child]
                        icon {
                            set_icon_name: Some("ld-app-window-symbolic"),
                        },
                        #[template_child]
                        title {
                            set_label: &t!("dropdown-window-switcher-empty-title"),
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let row_list = gtk::ListBox::new();
        let row_sender = sender.input_sender().clone();
        row_list.connect_row_activated(move |_list_box, row| {
            if let Ok(index) = usize::try_from(row.index()) {
                row_sender.emit(WindowSwitcherDropdownMsg::RowClicked(index as u32));
            }
        });

        let rows = FactoryVecDeque::builder().launch(row_list).detach();

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn_watchers(&sender, &init.service, &init.config, &init.ipc_state);

        let mut model = Self {
            service: init.service,
            config: init.config,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            rows,
            ordered_keys: Vec::new(),
            highlighted_index: None,
        };
        model.rebuild_rows();

        let row_list = model.rows.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            WindowSwitcherDropdownMsg::RowClicked(index) => {
                self.activate_row(index as usize);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: WindowSwitcherDropdownCmd,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            WindowSwitcherDropdownCmd::ToplevelsChanged
            | WindowSwitcherDropdownCmd::ConfigChanged => {
                self.rebuild_rows();
            }
            WindowSwitcherDropdownCmd::CycleStep => {
                self.cycle_step();
            }
            WindowSwitcherDropdownCmd::CycleCommit => {
                self.cycle_commit(root);
            }
        }
    }
}
