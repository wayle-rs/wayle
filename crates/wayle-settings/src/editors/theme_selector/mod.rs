//! "Apply theme preset" action button. Opens a popover listing available
//! themes; clicking one overwrites the current palette colors with that
//! theme's values. The button has no persistent selection state.

mod helpers;
mod methods;
mod row;

use std::{cell::RefCell, collections::BTreeSet};

use relm4::{gtk, gtk::prelude::*, prelude::*};
pub(crate) use row::theme_selector;
use wayle_config::{
    ConfigProperty,
    schemas::styling::{PaletteConfig, ThemeEntry},
};
use wayle_i18n::t;

use self::methods::populate_list;
use super::{WatcherHandle, spawn_property_watcher};

struct SwatchStyles {
    provider: gtk::CssProvider,
    registered_hexes: BTreeSet<String>,
}

thread_local! {
    static SWATCH_PROVIDER: RefCell<Option<SwatchStyles>> = const { RefCell::new(None) };
}

pub(crate) struct ThemeSelectorControl {
    pub(super) available: ConfigProperty<Vec<ThemeEntry>>,
    pub(super) palette: PaletteConfig,
    pub(super) popover: gtk::Popover,
    pub(super) list_box: gtk::ListBox,
    _watcher: WatcherHandle,
}

pub(crate) struct ThemeSelectorInit {
    pub(crate) available: ConfigProperty<Vec<ThemeEntry>>,
    pub(crate) palette: PaletteConfig,
}

#[derive(Debug)]
pub(crate) enum ThemeSelectorMsg {
    Apply(String),
    RebuildList,
}

impl SimpleComponent for ThemeSelectorControl {
    type Init = ThemeSelectorInit;
    type Input = ThemeSelectorMsg;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .hexpand(false)
            .valign(gtk::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let button_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .valign(gtk::Align::Center)
            .build();

        let button_icon = gtk::Image::from_icon_name("ld-palette-symbolic");
        button_icon.add_css_class("theme-preset-button-icon");
        button_content.append(&button_icon);

        let button_label = gtk::Label::new(Some(&t("settings-theme-preset-apply")));
        button_label.add_css_class("theme-preset-button-label");
        button_content.append(&button_label);

        let button_chevron = gtk::Image::from_icon_name("ld-chevron-down-symbolic");
        button_chevron.add_css_class("theme-preset-button-chevron");
        button_content.append(&button_chevron);

        let button = gtk::Button::builder().child(&button_content).build();
        button.add_css_class("theme-preset-button");
        button.set_cursor_from_name(Some("pointer"));

        let list_box = gtk::ListBox::new();
        list_box.add_css_class("theme-preset-list");
        list_box.set_selection_mode(gtk::SelectionMode::None);

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&list_box)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .build();
        scrolled.add_css_class("theme-preset-scroll");

        let popover = gtk::Popover::builder()
            .child(&scrolled)
            .has_arrow(false)
            .build();
        popover.add_css_class("theme-preset-popover");
        popover.set_parent(&button);

        let popover_ref = popover.clone();
        button.connect_clicked(move |_| popover_ref.popup());

        populate_list(&list_box, &init.available.get(), &sender);

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&init.available, move || {
            input_sender.send(ThemeSelectorMsg::RebuildList).is_ok()
        });

        let popover_cleanup = popover.clone();
        button.connect_destroy(move |_| popover_cleanup.unparent());

        root.append(&button);

        let model = Self {
            available: init.available,
            palette: init.palette,
            popover,
            list_box,
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ThemeSelectorMsg::Apply(name) => self.on_apply(name),
            ThemeSelectorMsg::RebuildList => self.on_rebuild_list(&sender),
        }
    }
}
