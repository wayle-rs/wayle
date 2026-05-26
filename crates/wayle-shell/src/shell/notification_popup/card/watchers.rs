use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_widgets::watch;

use super::{CardCmd, NotificationPopupCard};

pub(super) fn spawn(sender: &ComponentSender<NotificationPopupCard>, config: &Arc<ConfigService>) {
    let notif_config = config.config().modules.notifications.clone();
    let shadow = notif_config.popup_shadow.clone();
    let urgency_bar = notif_config.popup_urgency_bar.clone();

    watch!(sender, [shadow.watch(), urgency_bar.watch()], |out| {
        let _ = out.send(CardCmd::ConfigChanged {
            shadow: shadow.get(),
            urgency_bar: urgency_bar.get(),
        });
    });
}
