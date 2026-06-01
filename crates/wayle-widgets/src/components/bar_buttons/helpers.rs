use glib::{Propagation, object::IsA};
use gtk4::{
    EventControllerScroll, EventControllerScrollFlags, GestureClick, Widget,
    prelude::{EventControllerExt, GestureSingleExt, WidgetExt},
};
use relm4::Sender;
use tracing::debug;

use super::types::BarButtonOutput;

pub(super) fn attach_click_gesture(widget: &impl IsA<Widget>, sender: Sender<BarButtonOutput>) {
    let click = GestureClick::new();
    click.set_button(0);

    click.connect_pressed(move |gesture, _n_press, _x, _y| {
        let button = gesture.current_button();
        let event = match button {
            1 => BarButtonOutput::LeftClick,
            2 => BarButtonOutput::MiddleClick,
            3 => BarButtonOutput::RightClick,
            _ => return,
        };
        debug!(
            button,
            classes = ?gesture.widget().map(|widget| widget.css_classes()),
            "bar button click"
        );
        let _ = sender.send(event);
    });

    widget.add_controller(click);
}

pub(super) fn attach_scroll_controller(
    widget: &impl IsA<Widget>,
    sender: Sender<BarButtonOutput>,
    sensitivity: f64,
) {
    let scroll = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
    let threshold = 0.5 / sensitivity.max(0.1);

    scroll.connect_scroll(move |controller, _dx, dy| {
        if dy.abs() < threshold {
            return Propagation::Proceed;
        }
        let event = if dy < 0.0 {
            BarButtonOutput::ScrollUp
        } else {
            BarButtonOutput::ScrollDown
        };
        debug!(
            dy,
            classes = ?controller.widget().map(|widget| widget.css_classes()),
            "bar button scroll"
        );
        let _ = sender.send(event);
        Propagation::Stop
    });

    widget.add_controller(scroll);
}

pub(super) fn setup_event_controllers(
    widget: &impl IsA<Widget>,
    sender: Sender<BarButtonOutput>,
    scroll_sensitivity: f64,
) {
    attach_click_gesture(widget, sender.clone());
    attach_scroll_controller(widget, sender, scroll_sensitivity);
}
