use relm4::{ComponentController, gtk, gtk::prelude::*, prelude::*};
use tracing::debug;
use wayle_config::{
    DropdownSources,
    schemas::modules::{CustomModuleDefinition, ExecutionMode},
};
use wayle_widgets::{prelude::BarButtonInput, utils::force_window_resize};

use super::{CustomModule, helpers, watchers};

impl CustomModule {
    pub(super) fn handle_definition_removed(&mut self, root: &gtk::Box) {
        if !self.definition_present {
            return;
        }

        debug!(
            module_id = %self.definition.id,
            "custom module definition was removed; hiding module"
        );

        self.definition_present = false;
        self.stop_execution_watchers();
        self.cancel_inflight_commands();
        root.set_visible(false);
        force_window_resize(root);
    }

    pub(super) fn handle_definition_changed(
        &mut self,
        sender: &ComponentSender<Self>,
        root: &gtk::Box,
        new_definition: CustomModuleDefinition,
    ) {
        let was_removed = !self.definition_present;
        if !was_removed && self.definition == new_definition {
            return;
        }

        let needs_restart =
            was_removed || Self::execution_settings_changed(&self.definition, &new_definition);

        self.cancel_inflight_commands();

        self.apply_visual_properties(&new_definition);
        self.definition = new_definition;
        self.definition_present = true;
        // Keep the opener's (shared) dropdown names in sync with the re-bound clicks, so
        // `wayle dropdown list` and CLI addressing follow a runtime re-configuration.
        // Update the names FIRST, then ask the bar to republish — running the rebuild
        // after the names are in place, rather than racing the config-reload republish.
        *self.dropdown_names.borrow_mut() = self.definition.dropdown_names();
        self.opener.request_republish();

        if needs_restart {
            self.last_output.clear();
            self.restart_execution_watchers(sender);
        }

        self.refresh_from_last_output(root);
    }

    fn execution_settings_changed(
        current: &CustomModuleDefinition,
        next: &CustomModuleDefinition,
    ) -> bool {
        current.mode != next.mode
            || current.interval_ms != next.interval_ms
            || current.restart_policy != next.restart_policy
            || current.restart_interval_ms != next.restart_interval_ms
            || current.command != next.command
    }

    fn apply_visual_properties(&self, definition: &CustomModuleDefinition) {
        self.show_icon.set(definition.icon_show);
        self.show_label.set(definition.label_show);
        self.show_border.set(definition.border_show);
        self.label_max_chars.set(definition.label_max_length);
        self.icon_color.set(definition.icon_color.clone());
        self.label_color.set(definition.label_color.clone());
        self.icon_bg_color.set(definition.icon_bg_color.clone());
        self.button_bg_color.set(definition.button_bg_color.clone());
        self.border_color.set(definition.border_color.clone());
    }

    fn stop_execution_watchers(&mut self) {
        self.poller_token.reset();
        self.watcher_token.reset();
    }

    fn restart_execution_watchers(&mut self, sender: &ComponentSender<Self>) {
        match self.definition.mode {
            ExecutionMode::Poll => {
                self.watcher_token.reset();
                watchers::spawn_command_poller(sender, &self.definition, self.poller_token.reset());
                watchers::run_definition_command(
                    sender,
                    &self.definition,
                    self.command_token.reset(),
                );
            }
            ExecutionMode::Watch => {
                self.poller_token.reset();
                watchers::spawn_command_watcher(
                    sender,
                    &self.definition,
                    self.watcher_token.reset(),
                );
            }
        }
    }

    fn cancel_inflight_commands(&mut self) {
        self.command_token.reset();
        self.scroll_debounce_token.reset();
    }

    fn refresh_from_last_output(&mut self, root: &gtk::Box) {
        let last_output = self.last_output.clone();
        self.apply_output(&last_output, root);
        force_window_resize(root);
    }

    pub(super) fn apply_output(&mut self, output: &str, root: &gtk::Box) {
        self.last_output = output.to_string();

        let parsed = helpers::ParsedOutput::parse(output);
        let label = helpers::format_label(&self.definition, &parsed);
        let icon = helpers::resolve_icon(&self.definition, &parsed);
        let tooltip = helpers::format_tooltip(&self.definition, &parsed);
        let is_visible = !helpers::should_hide(&parsed.raw, self.definition.hide_if_empty);
        let new_classes = helpers::resolve_classes(&self.definition, &parsed);

        self.bar_button.emit(BarButtonInput::SetLabel(label));
        self.bar_button.emit(BarButtonInput::SetIcon(icon));
        self.bar_button.emit(BarButtonInput::SetTooltip(tooltip));
        root.set_visible(is_visible);

        for old_class in &self.dynamic_classes {
            if !new_classes.contains(old_class) {
                root.remove_css_class(old_class);
            }
        }
        for new_class in &new_classes {
            if !self.dynamic_classes.contains(new_class) {
                root.add_css_class(new_class);
            }
        }
        self.dynamic_classes = new_classes;
    }
}
