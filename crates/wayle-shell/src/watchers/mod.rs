mod color_extractor;
mod css;
mod location;
mod monitors;
mod notification;
mod osd;
mod scss_dev;
mod sysinfo;
mod wallpaper;
mod weather;

use std::env;

pub(crate) use color_extractor::build_extractor_config;
use relm4::ComponentSender;
use wayle_styling::watcher as user_style;

use crate::shell::{Shell, ShellCmd, ShellServices};

pub(crate) fn init(sender: &ComponentSender<Shell>, services: &ShellServices) {
    css::spawn(sender, services);
    location::spawn(sender, services);
    monitors::spawn(sender);
    osd::spawn(sender, services);
    color_extractor::spawn(services);
    notification::spawn(services);
    sysinfo::spawn(services);
    user_style::spawn(sender, services.config.clone(), ShellCmd::CssRecompiled);
    wallpaper::spawn(services);
    weather::spawn(services);

    if env::var("WAYLE_DEV").is_ok_and(|value| value == "1") {
        scss_dev::spawn(sender, services);
    }
}
