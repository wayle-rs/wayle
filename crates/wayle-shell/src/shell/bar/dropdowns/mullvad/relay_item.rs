use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_mullvad::NetworkTarget;

/// One relay (leaf) row in the country -> city -> relay tree.
pub(super) struct RelayItem {
    hostname: String,
    target: NetworkTarget,
    active: bool,
}

pub(super) struct RelayItemInit {
    pub hostname: String,
    pub target: NetworkTarget,
    pub active: bool,
}

#[derive(Debug)]
pub(super) enum RelayItemOutput {
    Selected(NetworkTarget),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for RelayItem {
    type Init = RelayItemInit;
    type Input = ();
    type Output = RelayItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            add_css_class: "mullvad-relay-item",
            set_cursor_from_name: Some("pointer"),
            #[watch]
            set_sensitive: self.active,

            gtk::Label {
                add_css_class: "mullvad-relay-name",
                set_halign: gtk::Align::Start,
                set_ellipsize: gtk::pango::EllipsizeMode::End,
                set_label: &self.hostname,
            },
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            hostname: init.hostname,
            target: init.target,
            active: init.active,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let target = self.target.clone();
        let out = sender.output_sender().clone();
        root.connect_clicked(move |_| {
            out.emit(RelayItemOutput::Selected(target.clone()));
        });

        let widgets = view_output!();
        widgets
    }
}
