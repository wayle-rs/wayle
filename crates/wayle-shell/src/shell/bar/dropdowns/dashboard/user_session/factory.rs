//! Session action factory for creating session action buttons in dashboard dropdown.

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::schemas::bar::dropdowns::dashboard::user_session::SessionAction;
use wayle_widgets::primitives::buttons::IconButton;

use crate::{i18n::t, shell::bar::dropdowns::dashboard::user_session::messages::UserSessionInput};

pub(crate) struct SessionActionFactoryInit {
    pub(crate) session_action: SessionAction,
}

pub(crate) struct SessionActionFactory {
    #[allow(dead_code)]
    session_action: SessionAction,
    action_button: IconButton,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for SessionActionFactory {
    type Init = SessionActionFactoryInit;
    type Input = ();
    type Output = UserSessionInput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "bar-item",
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let action_button = match &init.session_action {
            SessionAction::Lock => make_session_btn(
                "ld-lock-symbolic",
                &t!("dropdown-dashboard-lock"),
                &["session-btn"],
                move || {
                    sender.output(UserSessionInput::Lock).ok();
                },
            ),
            SessionAction::Logout => make_session_btn(
                "ld-log-out-symbolic",
                &t!("dropdown-dashboard-logout"),
                &["session-btn"],
                move || {
                    sender.output(UserSessionInput::Logout).ok();
                },
            ),
            SessionAction::Reboot => make_session_btn(
                "ld-refresh-cw-symbolic",
                &t!("dropdown-dashboard-reboot"),
                &["session-btn"],
                move || {
                    sender.output(UserSessionInput::Reboot).ok();
                },
            ),
            SessionAction::PowerOff => make_session_btn(
                "ld-power-symbolic",
                &t!("dropdown-dashboard-power-off"),
                &["session-btn"],
                move || {
                    sender.output(UserSessionInput::PowerOff).ok();
                },
            ),
        };

        Self {
            session_action: init.session_action,
            action_button,
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

        root.append(self.action_button.widget_ref());

        widgets
    }
}

fn make_session_btn(
    icon: &str,
    tooltip: &str,
    extra_classes: &[&str],
    on_click: impl Fn() + 'static,
) -> IconButton {
    let btn_template = IconButton::init(());
    let btn: &gtk::Button = &btn_template;

    for class in extra_classes {
        btn.add_css_class(class);
    }
    btn.set_tooltip_text(Some(tooltip));

    btn.connect_clicked(move |_| on_click());

    let image = gtk::Image::new();
    image.set_icon_name(Some(icon));
    btn.set_child(Some(&image));

    btn_template
}
