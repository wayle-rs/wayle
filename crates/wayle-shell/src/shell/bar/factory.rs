//! Bar item factory for creating modules and groups.

use std::rc::Rc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::schemas::bar::{BarItem, BarModule};
use wayle_widgets::prelude::BarSettings;

use crate::shell::{
    bar::{
        dropdowns::{DropdownOpener, DropdownRegistry, OPENER_CSS_CLASS},
        modules::{ModuleInstance, create_module},
    },
    services::ShellServices,
};

pub(crate) struct BarItemFactoryInit {
    pub(crate) item: BarItem,
    pub(crate) settings: BarSettings,
    pub(crate) services: ShellServices,
    pub(crate) dropdowns: Rc<DropdownRegistry>,
}

pub(crate) struct BarItemFactory {
    item: BarItem,
    settings: BarSettings,
    #[allow(dead_code)]
    services: ShellServices,
    #[allow(dead_code)]
    dropdowns: Rc<DropdownRegistry>,
    /// Each created module paired with its layout type and the dropdown opener it
    /// published (if any), in layout order. The type + opener let a CLI dropdown
    /// identifier be resolved back to the module's opener (see
    /// [`Self::dropdown_targets`]).
    modules: Vec<(BarModule, ModuleInstance, Option<DropdownOpener>)>,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for BarItemFactory {
    type Init = BarItemFactoryInit;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "bar-item",
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let build = |module_ref: &wayle_config::schemas::bar::ModuleRef| {
            create_module(module_ref, &init.settings, &init.services, &init.dropdowns)
                .map(|(instance, opener)| (module_ref.module().clone(), instance, opener))
        };
        let modules: Vec<(BarModule, ModuleInstance, Option<DropdownOpener>)> = match &init.item {
            BarItem::Module(module_ref) => build(module_ref).into_iter().collect(),
            BarItem::Group(group) => group.modules.iter().filter_map(build).collect(),
        };

        Self {
            item: init.item,
            settings: init.settings,
            services: init.services,
            dropdowns: init.dropdowns,
            modules,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        let orientation = if self.settings.is_vertical.get() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        };
        root.set_orientation(orientation);

        if let BarItem::Group(group) = &self.item {
            root.set_widget_name(&group.name);
            root.add_css_class("bar-group");
        }

        for (_, instance, opener) in &self.modules {
            let widget = instance.controller.widget();
            widget.add_css_class("module");
            // Mark dropdown-capable modules as openers on their OUTER widget (which
            // the module owns), so the bar-click gesture never pre-dismisses a press
            // on them. `handle_bar_click` walks ancestors, so this shields the inner
            // button too — and unlike the `BarButton` itself, this widget's classes
            // aren't rewritten out from under us.
            if opener.is_some() {
                widget.add_css_class(OPENER_CSS_CLASS);
            }
            if let Some(class) = &instance.class {
                widget.add_css_class(class);
            }
            root.append(widget);

            let container = root.clone();
            widget.connect_notify_local(Some("visible"), move |_, _| {
                sync_container_visibility(&container);
            });
        }

        sync_container_visibility(&root);

        widgets
    }
}

impl BarItemFactory {
    pub(crate) fn matches(&self, item: &BarItem) -> bool {
        self.item == *item
    }

    /// Each contained module's type paired with its dropdown opener (if any), in
    /// layout order. Used to resolve a CLI dropdown identifier (e.g.
    /// `audio@microphone`) back to the opener that toggles it — the same opener the
    /// module's own click uses. Opener-less modules yield `None` (kept in place to
    /// preserve layout-order indexing).
    pub(crate) fn dropdown_targets(&self) -> Vec<(BarModule, Option<DropdownOpener>)> {
        self.modules
            .iter()
            .map(|(module, _, opener)| (module.clone(), opener.clone()))
            .collect()
    }
}

fn sync_container_visibility(container: &gtk::Box) {
    let has_visible_child = container
        .observe_children()
        .into_iter()
        .filter_map(|obj| obj.ok()?.downcast::<gtk::Widget>().ok())
        .any(|widget| widget.get_visible());

    container.set_visible(has_visible_child);
}
