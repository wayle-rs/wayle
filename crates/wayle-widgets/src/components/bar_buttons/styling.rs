//! CSS variable generation for bar button styling.

use relm4::{ComponentSender, gtk};
use wayle_config::schemas::styling::{ThemeProvider, ThresholdColors};

use super::component::{BarButton, BarButtonCmd};
use crate::{
    styling::{InlineStyling, resolve_color},
    watch,
};

impl InlineStyling for BarButton {
    type Sender = ComponentSender<Self>;
    type Cmd = BarButtonCmd;

    fn css_provider(&self) -> &gtk::CssProvider {
        &self.css_provider
    }

    fn spawn_style_watcher(&self, sender: &Self::Sender) {
        let show_icon = self.behavior.show_icon.clone();
        let show_label = self.behavior.show_label.clone();
        let show_border = self.behavior.show_border.clone();
        let visible = self.behavior.visible.clone();
        let label_max_chars = self.behavior.label_max_chars.clone();
        let icon_color = self.colors.icon_color.clone();
        let label_color = self.colors.label_color.clone();
        let icon_background = self.colors.icon_background.clone();
        let button_background = self.colors.button_background.clone();
        let border_color = self.colors.border_color.clone();
        let border_location = self.settings.border_location.clone();
        let border_width = self.settings.border_width.clone();
        let theme_provider = self.settings.theme_provider.clone();
        let is_vertical = self.settings.is_vertical.clone();

        watch!(
            sender,
            [
                show_icon.watch(),
                show_label.watch(),
                show_border.watch(),
                visible.watch(),
                label_max_chars.watch(),
                icon_color.watch(),
                label_color.watch(),
                icon_background.watch(),
                button_background.watch(),
                border_color.watch(),
                border_location.watch(),
                border_width.watch(),
                theme_provider.watch(),
                is_vertical.watch(),
            ],
            |out| {
                let _ = out.send(BarButtonCmd::ConfigChanged);
            }
        );
    }

    fn build_css(&self) -> String {
        let is_wayle = matches!(self.settings.theme_provider.get(), ThemeProvider::Wayle);
        let t = &self.threshold_overrides;

        let icon_color =
            ThresholdColors::resolve_or(&t.icon_color, self.resolve_icon_color(is_wayle));
        let label_color = ThresholdColors::resolve_or(
            &t.label_color,
            resolve_color(&self.colors.label_color, is_wayle),
        );
        let icon_bg = ThresholdColors::resolve_or(
            &t.icon_background,
            resolve_color(&self.colors.icon_background, is_wayle),
        );
        let button_bg = ThresholdColors::resolve_or(
            &t.button_background,
            resolve_color(&self.colors.button_background, is_wayle),
        );
        let border_color = ThresholdColors::resolve_or(
            &t.border_color,
            resolve_color(&self.colors.border_color, is_wayle),
        );
        let border_width = self.settings.border_width.get();

        format!(
            "* {{ \
             --bar-btn-icon-color: {}; \
             --bar-btn-label-color: {}; \
             --bar-btn-icon-bg: {}; \
             --bar-btn-bg: {}; \
             --bar-btn-border-color: {}; \
             --bar-btn-border-width: {}px; \
             }}",
            icon_color, label_color, icon_bg, button_bg, border_color, border_width
        )
    }
}
