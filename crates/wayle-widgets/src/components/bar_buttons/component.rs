//! Bar button component with runtime-switchable visual variants.

use std::borrow::Cow;

use glib::prelude::CastNone;
#[allow(deprecated)]
use gtk4::prelude::StyleContextExt;
use gtk4::prelude::{BoxExt, ListModelExt, OrientableExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, gtk, prelude::*};
use wayle_config::schemas::{bar::IconPosition, styling::{CssToken, ThresholdColors}};

use super::{
    helpers::setup_event_controllers,
    types::{
        BarButtonBehavior, BarButtonClass, BarButtonColors, BarButtonOutput, BarButtonVariant,
        BarSettings,
    },
    watchers::{spawn_icon_position_watcher, spawn_variant_watcher},
};
use crate::{styling::InlineStyling, utils::force_window_resize};

/// Initialization data for BarButton.
#[derive(Debug, Clone)]
pub struct BarButtonInit {
    /// Icon name (symbolic icon).
    pub icon: String,
    /// Button label text.
    pub label: String,
    /// Optional tooltip.
    pub tooltip: Option<String>,
    /// Module-specific color configuration.
    pub colors: BarButtonColors,
    /// Module-specific behavior configuration.
    pub behavior: BarButtonBehavior,
    /// Bar-wide settings.
    pub settings: BarSettings,
}

/// Input messages for BarButton.
#[derive(Debug)]
pub enum BarButtonInput {
    /// Update the icon.
    SetIcon(String),
    /// Update the label.
    SetLabel(String),
    /// Update the tooltip.
    SetTooltip(Option<String>),
    /// Lock label width to prevent resize while a popover is open.
    FreezeSize,
    /// Unlock label width, restoring normal sizing.
    ThawSize,
    /// Config property changed.
    ConfigChanged,
    /// Apply threshold-based color overrides.
    SetThresholdColors(ThresholdColors),
}

/// Command outputs from async watchers.
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BarButtonCmd {
    VariantChanged(BarButtonVariant),
    IconPositionChanged(IconPosition),
    ConfigChanged,
}

/// Bar button with switchable visual variants.
pub struct BarButton {
    icon: String,
    label: String,
    tooltip: Option<String>,
    size_frozen: bool,
    pending_label: Option<String>,
    pub(super) variant: BarButtonVariant,
    pub(super) colors: BarButtonColors,
    pub(super) behavior: BarButtonBehavior,
    pub(super) settings: BarSettings,
    pub(super) css_provider: gtk::CssProvider,
    /// Threshold-based color overrides. Applied on top of config colors.
    pub(super) threshold_overrides: ThresholdColors,
}

#[relm4::component(pub)]
impl Component for BarButton {
    type Init = BarButtonInit;
    type Input = BarButtonInput;
    type Output = BarButtonOutput;
    type CommandOutput = BarButtonCmd;

    view! {
        #[root]
        gtk::MenuButton {
            set_always_show_arrow: false,
            set_cursor_from_name: Some("pointer"),

            set_css_classes: &model.css_classes(),

            #[watch]
            set_visible: model.behavior.visible.get(),

            #[watch]
            set_tooltip_text: model.tooltip.as_deref(),

            #[watch]
            set_hexpand: model.settings.is_vertical.get(),

            #[wrap(Some)]
            #[name = "content"]
            set_child = &gtk::Box {
                add_css_class: "bar-button-content",

                #[watch]
                set_hexpand: model.settings.is_vertical.get(),

                #[watch]
                set_orientation: model.orientation(),

                #[name = "icon_container"]
                gtk::Box {
                    add_css_class: "icon-container",

                    #[watch]
                    set_visible: model.behavior.show_icon.get(),

                    #[watch]
                    set_hexpand: model.icon_should_center(),

                    gtk::Image {
                        set_halign: gtk::Align::Center,

                        #[watch]
                        set_hexpand: model.icon_should_center(),

                        #[watch]
                        set_icon_name: Some(&model.icon),
                    },
                },

                #[name = "label_container"]
                gtk::Box {
                    add_css_class: "label-container",

                    #[watch]
                    set_visible: model.behavior.show_label.get(),

                    #[watch]
                    set_hexpand: model.settings.is_vertical.get(),

                    gtk::Label {
                        add_css_class: "bar-button-label",
                        set_align: gtk::Align::Center,
                        set_justify: gtk::Justification::Center,

                        #[watch]
                        set_hexpand: model.settings.is_vertical.get(),

                        #[watch]
                        set_label: &model.label,

                        #[watch]
                        set_ellipsize: model.ellipsize(),

                        #[watch]
                        set_max_width_chars: model.max_width_chars(),
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let css_provider = gtk::CssProvider::new();
        let scroll_sensitivity = init.settings.scroll_sensitivity;

        let model = BarButton {
            icon: init.icon,
            label: init.label,
            tooltip: init.tooltip,
            size_frozen: false,
            pending_label: None,
            variant: init.settings.variant.get(),
            colors: init.colors,
            behavior: init.behavior,
            settings: init.settings,
            css_provider,
            threshold_overrides: ThresholdColors::default(),
        };

        #[allow(deprecated)]
        root.style_context()
            .add_provider(&model.css_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
        model.reload_css();
        model.apply_css_classes(&root);

        let widgets = view_output!();

        if model.settings.icon_position.get() == IconPosition::End {
            widgets
                .content
                .reorder_child_after(&widgets.icon_container, Some(&widgets.label_container));
        }

        setup_event_controllers(&root, sender.output_sender().clone(), scroll_sensitivity);
        spawn_variant_watcher(&model.settings.variant, &sender);
        spawn_icon_position_watcher(&model.settings.icon_position, &sender);
        model.spawn_style_watcher(&sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BarButtonInput::SetIcon(icon) => self.icon = icon,
            BarButtonInput::SetLabel(label) => {
                if self.size_frozen {
                    self.pending_label = Some(label);
                } else if self.label != label {
                    self.label = label;
                }
            }
            BarButtonInput::SetTooltip(tooltip) => self.tooltip = tooltip,
            BarButtonInput::FreezeSize => {
                self.size_frozen = true;
            }
            BarButtonInput::ThawSize => {
                self.size_frozen = false;
                if let Some(label) = self.pending_label.take() {
                    self.label = label;
                }
            }
            BarButtonInput::ConfigChanged => {}
            BarButtonInput::SetThresholdColors(colors) => {
                self.threshold_overrides = colors;
                self.reload_css();
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            BarButtonCmd::VariantChanged(variant) => {
                self.variant = variant;
                self.reload_css();
                self.apply_css_classes(root);
            }
            BarButtonCmd::IconPositionChanged(position) => {
                Self::reorder_children(root, position);
                self.apply_css_classes(root);
            }
            BarButtonCmd::ConfigChanged => {
                self.reload_css();
                self.apply_css_classes(root);
                force_window_resize(root);
            }
        }
    }
}
impl BarButton {
    fn apply_css_classes(&self, root: &gtk::MenuButton) {
        root.set_css_classes(&self.css_classes());
    }

    fn css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec![BarButtonClass::BASE];

        classes.push(match self.variant {
            BarButtonVariant::Basic => "basic",
            BarButtonVariant::BlockPrefix => "block-prefix",
            BarButtonVariant::IconSquare => "icon-square",
        });

        if !self.behavior.show_label.get() {
            classes.push(BarButtonClass::ICON_ONLY);
        }
        if !self.behavior.show_icon.get() {
            classes.push(BarButtonClass::LABEL_ONLY);
        }
        if self.settings.is_vertical.get() {
            classes.push(BarButtonClass::VERTICAL);
        }
        if self.behavior.show_border.get()
            && let Some(border_class) = self.settings.border_location.get().css_class()
        {
            classes.push(border_class);
        }
        if let Some(icon_position_class) = self.settings.icon_position.get().css_class() {
            classes.push(icon_position_class);
        }
        classes
    }

    fn orientation(&self) -> gtk::Orientation {
        if self.settings.is_vertical.get() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    fn ellipsize(&self) -> gtk::pango::EllipsizeMode {
        if self.behavior.label_max_chars.get() > 0 {
            gtk::pango::EllipsizeMode::End
        } else {
            gtk::pango::EllipsizeMode::None
        }
    }

    fn max_width_chars(&self) -> i32 {
        let max = self.behavior.label_max_chars.get();
        if max > 0 { max as i32 } else { -1 }
    }

    fn is_icon_only(&self) -> bool {
        !self.behavior.show_label.get()
    }

    fn icon_should_center(&self) -> bool {
        self.is_icon_only() || self.settings.is_vertical.get()
    }

    fn reorder_children(root: &gtk::MenuButton, position: IconPosition) {
        let Some(content) = root.child().and_downcast::<gtk::Box>() else {
            return;
        };

        let list_model = content.observe_children();
        let mut icon_container: Option<gtk::Widget> = None;
        let mut label_container: Option<gtk::Widget> = None;

        for i in 0..list_model.n_items() {
            let Some(widget) = list_model.item(i).and_downcast::<gtk::Widget>() else {
                continue;
            };
            if widget.has_css_class("icon-container") {
                icon_container = Some(widget);
            } else if widget.has_css_class("label-container") {
                label_container = Some(widget);
            }
        }

        let (Some(icon_container), Some(label_container)) = (icon_container, label_container)
        else {
            return;
        };

        match position {
            IconPosition::Start => {
                content.reorder_child_after(&icon_container, gtk::Widget::NONE);
            }
            IconPosition::End => {
                content.reorder_child_after(&icon_container, Some(&label_container));
            }
        }
    }

    pub(super) fn resolve_icon_color(&self, is_wayle_themed: bool) -> Cow<'static, str> {
        let color = if is_wayle_themed {
            self.colors.icon_color.get()
        } else {
            self.colors.icon_color.default().clone()
        };

        if color.is_auto() {
            let token = match self.variant {
                BarButtonVariant::Basic => self.colors.auto_icon_color,
                BarButtonVariant::BlockPrefix | BarButtonVariant::IconSquare => {
                    CssToken::FgOnAccent
                }
            };
            Cow::Borrowed(token.css_var())
        } else {
            color.to_css()
        }
    }
}
