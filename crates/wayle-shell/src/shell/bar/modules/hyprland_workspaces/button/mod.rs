mod methods;

use std::{collections::HashSet, mem, sync::Arc};

use gtk::prelude::*;
use relm4::{factory::FactoryComponent, prelude::*};
use wayle_config::schemas::modules::{ActiveIndicator, DisplayMode, HyprlandWorkspacesConfig};
use wayle_hyprland::{Address, Client, WorkspaceId};

use crate::shell::bar::modules::hyprland_workspaces::helpers::{
    IconContext, WorkspaceState, compute_static_css_classes, determine_workspace_state,
    resolve_workspace_icons, workspace_id_css_class, workspace_name_css_class,
};

const WORKSPACE_LABEL_CSS: &str = "workspace-label";
const WORKSPACE_CUSTOM_ICON_CSS: &str = "workspace-custom-icon";
const WORKSPACE_DIVIDER_CSS: &str = "workspace-divider";
const WORKSPACE_ICON_CSS: &str = "workspace-icon";
const WORKSPACE_ICON_EMPTY_CSS: &str = "workspace-icon-empty";
const WORKSPACE_ICONS_CSS: &str = "workspace-icons";

/// Context for building a workspace button.
#[derive(Debug, Clone)]
pub(crate) struct ButtonBuildContext<'a> {
    pub id: WorkspaceId,
    pub display_id: WorkspaceId,
    pub name: &'a str,
    pub windows: u16,
    pub is_active: bool,
    pub is_urgent: bool,
    pub is_vertical: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AppIconInit {
    pub icon_name: String,
    pub addresses: Vec<Address>,
}

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceButtonInit {
    pub id: WorkspaceId,
    pub display_id: WorkspaceId,
    pub name: String,
    pub windows: u16,
    pub is_active: bool,
    pub is_urgent: bool,
    pub is_vertical: bool,

    pub display_mode: DisplayMode,
    pub active_indicator: ActiveIndicator,
    pub label_use_name: bool,
    pub mapped_icon: Option<String>,
    pub divider: String,

    pub show_app_icons: bool,
    pub app_icons: Vec<AppIconInit>,
    pub urgent_addresses: HashSet<Address>,
    pub empty_icon: String,
    pub icon_gap_px: i32,
}

pub(super) struct AppIcon {
    pub addresses: Vec<Address>,
    pub widget: gtk::Image,
}

pub(crate) struct WorkspaceButton {
    id: WorkspaceId,
    pub(super) state: WorkspaceState,
    pub(super) is_urgent: bool,
    pub(super) css_id_class: String,
    pub(super) css_name_class: String,
    pub(super) static_classes: Vec<&'static str>,

    pub(super) display_id: WorkspaceId,
    pub(super) name: String,
    pub(super) display_mode: DisplayMode,
    pub(super) label_use_name: bool,
    pub(super) mapped_icon: Option<String>,
    pub(super) divider: String,
    pub(super) is_vertical: bool,

    pub(super) show_app_icons: bool,
    app_icon_inits: Vec<AppIconInit>,
    pub(super) app_icons: Vec<AppIcon>,
    initial_urgent_addrs: HashSet<Address>,
    pub(super) empty_icon: String,
    pub(super) icon_gap_px: i32,
}

#[derive(Debug)]
pub(crate) enum WorkspaceButtonInput {
    UpdateState {
        windows: u16,
        is_active: bool,
        is_urgent: bool,
        urgent_addresses: HashSet<Address>,
    },
}

#[derive(Debug)]
pub(crate) enum WorkspaceButtonOutput {
    Clicked(WorkspaceId),
    ScrollUp,
    ScrollDown,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for WorkspaceButton {
    type Init = WorkspaceButtonInit;
    type Input = WorkspaceButtonInput;
    type Output = WorkspaceButtonOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_css_classes: &self.current_css_classes(),

            connect_clicked[sender, id = self.id] => move |_| {
                sender.output(WorkspaceButtonOutput::Clicked(id)).ok();
            },

            #[name = "content"]
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

                    #[name = "identity"]
                    gtk::Box {},

                    #[name = "divider"]
                    gtk::Label {
                        add_css_class: WORKSPACE_DIVIDER_CSS,
                        #[watch]
                        set_visible: self.show_divider(),
                        #[watch]
                        set_label: &self.divider,
                        set_valign: gtk::Align::Center,
                    },
                },

                #[name = "app_icons_container"]
                gtk::Box {
                    add_css_class: WORKSPACE_ICONS_CSS,
                    #[watch]
                    set_visible: self.show_app_icons,
                    #[watch]
                    set_orientation: self.orientation(),
                    #[watch]
                    set_spacing: self.icon_gap_px,
                    #[watch]
                    set_halign: self.icons_halign(),
                    #[watch]
                    set_valign: gtk::Align::Fill,
                },
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let state = determine_workspace_state(init.is_active, init.windows);
        let static_classes = compute_static_css_classes(
            init.id,
            init.active_indicator.css_class(),
            init.is_vertical,
        );

        Self {
            id: init.id,
            state,
            is_urgent: init.is_urgent,
            css_id_class: workspace_id_css_class(init.id),
            css_name_class: workspace_name_css_class(&init.name),
            static_classes,

            display_id: init.display_id,
            name: init.name,
            display_mode: init.display_mode,
            label_use_name: init.label_use_name,
            mapped_icon: init.mapped_icon,
            divider: init.divider,
            is_vertical: init.is_vertical,

            show_app_icons: init.show_app_icons,
            app_icon_inits: init.app_icons,
            app_icons: Vec::new(),
            initial_urgent_addrs: init.urgent_addresses,
            empty_icon: init.empty_icon,
            icon_gap_px: init.icon_gap_px,
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

        let scroll_controller = gtk::EventControllerScroll::new(
            gtk::EventControllerScrollFlags::VERTICAL | gtk::EventControllerScrollFlags::DISCRETE,
        );
        scroll_controller.connect_scroll({
            let sender = sender.clone();
            move |_, _dx, dy| {
                if dy > 0.0 {
                    sender.output(WorkspaceButtonOutput::ScrollDown).ok();
                } else if dy < 0.0 {
                    sender.output(WorkspaceButtonOutput::ScrollUp).ok();
                }
                gtk::glib::Propagation::Stop
            }
        });
        root.add_controller(scroll_controller);

        self.populate_identity(&widgets.identity);
        let urgent_addrs = mem::take(&mut self.initial_urgent_addrs);
        self.populate_app_icons(&widgets.app_icons_container, &urgent_addrs);

        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            WorkspaceButtonInput::UpdateState {
                windows,
                is_active,
                is_urgent,
                urgent_addresses,
            } => {
                self.state = determine_workspace_state(is_active, windows);
                self.apply_urgency(is_urgent, &urgent_addresses);
            }
        }
    }
}

pub(crate) fn build_button_init(
    ctx: &ButtonBuildContext<'_>,
    config: &HyprlandWorkspacesConfig,
    clients: &[Arc<Client>],
    urgent_addresses: HashSet<Address>,
) -> WorkspaceButtonInit {
    let workspace_map = config.workspace_map.get();
    let mapped_style = i32::try_from(ctx.id)
        .ok()
        .and_then(|id| workspace_map.lookup(id, ctx.name));
    let mapped_icon = mapped_style.and_then(|style| style.icon.clone());

    let app_icons = if config.app_icons_show.get() {
        let user_map = config.app_icon_map.get();
        let fallback = config.app_icons_fallback.get();
        let icon_ctx = IconContext {
            user_map: &user_map,
            fallback: &fallback,
        };
        let resolved =
            resolve_workspace_icons(ctx.id, clients, &icon_ctx, config.app_icons_dedupe.get());
        resolved
            .into_iter()
            .map(|resolved| AppIconInit {
                icon_name: resolved.icon_name,
                addresses: resolved.addresses,
            })
            .collect()
    } else {
        Vec::new()
    };

    let icon_gap_px = (config.icon_gap.get().value() * 16.0).round() as i32;

    WorkspaceButtonInit {
        id: ctx.id,
        display_id: ctx.display_id,
        name: ctx.name.to_string(),
        windows: ctx.windows,
        is_active: ctx.is_active,
        is_urgent: ctx.is_urgent,
        is_vertical: ctx.is_vertical,

        display_mode: config.display_mode.get(),
        active_indicator: config.active_indicator.get(),
        label_use_name: config.label_use_name.get(),
        mapped_icon,
        divider: config.divider.get(),

        show_app_icons: config.app_icons_show.get(),
        app_icons,
        urgent_addresses,
        empty_icon: config.app_icons_empty.get(),
        icon_gap_px,
    }
}
