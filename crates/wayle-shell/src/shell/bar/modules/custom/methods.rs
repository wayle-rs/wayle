use relm4::{ComponentController, gtk, gtk::prelude::*, prelude::*};
use tracing::debug;
use wayle_config::schemas::modules::{CustomModuleDefinition, ExecutionMode};
use wayle_widgets::{prelude::BarButtonInput, utils::force_window_resize};

use super::{
    CustomModule,
    dropdown::{CustomDropdownPicker, DropdownPickerInput, DropdownPickerOutput},
    helpers, messages::CustomCmd, messages::CustomMsg, watchers,
};

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
        self.cleanup_dropdown();
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

        // Recreate dropdown if its config changed.
        if self.definition.dropdown_list_command != new_definition.dropdown_list_command
            || self.definition.dropdown_select_command != new_definition.dropdown_select_command
        {
            self.cleanup_dropdown();
        }

        self.apply_visual_properties(&new_definition);
        self.definition = new_definition;
        self.definition_present = true;

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

        // Re-apply ellipsize mode so hot-reload works correctly.
        use wayle_config::schemas::modules::LabelEllipsize;
        let ellipsize = match definition.label_ellipsize {
            LabelEllipsize::End => gtk::pango::EllipsizeMode::End,
            LabelEllipsize::Middle => gtk::pango::EllipsizeMode::Middle,
            LabelEllipsize::Start => gtk::pango::EllipsizeMode::Start,
        };
        self.bar_button.emit(BarButtonInput::SetEllipsize(ellipsize));
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

    // --- Inline dropdown methods ---

    pub(super) fn toggle_inline_dropdown(&mut self, sender: &ComponentSender<Self>) {
        let Some(list_cmd) = &self.definition.dropdown_list_command else {
            return;
        };

        // Lazy-create the popover and picker on first use.
        if self.dropdown_popover.is_none() {
            let picker = CustomDropdownPicker::builder()
                .launch(())
                .forward(sender.input_sender(), |output| match output {
                    DropdownPickerOutput::Selected(item) => CustomMsg::DropdownItemSelected(item),
                });

            // Wrap the picker in a .dropdown container so it gets standard
            // dropdown styling (background, border-radius, shadow, etc.).
            let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
            wrapper.add_css_class("dropdown");
            wrapper.append(picker.widget());

            let popover = gtk::Popover::new();
            popover.set_child(Some(&wrapper));
            popover.set_has_arrow(false);
            popover.set_autohide(true);
            popover.add_css_class("shadow");

            let bar_widget = self.bar_button.widget().upcast_ref::<gtk::Widget>();
            popover.set_parent(bar_widget);

            // Position based on bar location (top → dropdown goes down, etc.).
            let position = Self::detect_popover_position(bar_widget);
            popover.set_position(position);

            self.dropdown_picker = Some(picker);
            self.dropdown_popover = Some(popover);
        }

        let popover = self.dropdown_popover.as_ref().unwrap_or_else(|| unreachable!());

        if popover.is_visible() {
            popover.popdown();
            return;
        }

        // Run dropdown-list-command asynchronously. The popover opens when
        // items arrive in show_dropdown_with_items().
        self.dropdown_load_gen = self.dropdown_load_gen.wrapping_add(1);
        let generation = self.dropdown_load_gen;
        let list_cmd = list_cmd.clone();
        let active_item = self.last_output.trim().to_string();
        sender.oneshot_command(async move {
            let output = watchers::run_command_for_output(&list_cmd).await;
            let items: Vec<String> = output
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            CustomCmd::DropdownListLoaded {
                items,
                active_item,
                generation,
            }
        });
    }

    pub(super) fn show_dropdown_with_items(
        &mut self,
        items: Vec<String>,
        active_item: String,
    ) {
        use wayle_config::schemas::modules::LabelEllipsize;
        let ellipsize = match self.definition.label_ellipsize {
            LabelEllipsize::End => gtk::pango::EllipsizeMode::End,
            LabelEllipsize::Middle => gtk::pango::EllipsizeMode::Middle,
            LabelEllipsize::Start => gtk::pango::EllipsizeMode::Start,
        };
        if let Some(picker) = &self.dropdown_picker {
            picker.emit(DropdownPickerInput::Refresh {
                items,
                active_item,
                ellipsize,
            });
        }
        // Defer popup() to the next idle cycle so the factory has time to
        // create the row widgets. Without this, the popover measures empty
        // content and opens with minimal height.
        if let Some(popover) = &self.dropdown_popover {
            let popover = popover.clone();
            gtk::glib::idle_add_local_once(move || {
                popover.popup();
            });
        }
    }

    pub(super) fn handle_dropdown_selection(
        &mut self,
        sender: &ComponentSender<Self>,
        selected: &str,
    ) {
        // Close the dropdown.
        if let Some(popover) = &self.dropdown_popover {
            popover.popdown();
        }

        // Run the select command, wait for it to complete, then refresh
        // the main command so the label updates immediately.
        //
        // The selected item is passed via the WAYLE_SELECTED environment
        // variable to avoid shell injection from item text containing
        // metacharacters. The command template can reference it as
        // $WAYLE_SELECTED.
        let select_cmd = self.definition.dropdown_select_command.clone();
        let selected = selected.to_string();
        let refresh_cmd = self.definition.command.clone();
        let token = self.command_token.reset();
        sender.oneshot_command(async move {
            // First: run the selection command and wait for it to finish.
            if let Some(cmd) = select_cmd {
                let _ = watchers::run_command_with_env(
                    &cmd,
                    "WAYLE_SELECTED",
                    &selected,
                )
                .await;
            }

            // Then: run the main command to get the updated value.
            if let Some(cmd) = refresh_cmd {
                let output = watchers::run_command_for_output(&cmd).await;
                // Check cancellation before delivering.
                if token.is_cancelled() {
                    return CustomCmd::CommandCancelled;
                }
                return CustomCmd::CommandOutput(output);
            }

            CustomCmd::CommandCancelled
        });
    }

    fn detect_popover_position(widget: &gtk::Widget) -> gtk::PositionType {
        let Some(window) = widget.root().and_then(|r| r.downcast::<gtk::Window>().ok()) else {
            return gtk::PositionType::Bottom;
        };

        if window.has_css_class("bottom") {
            gtk::PositionType::Top
        } else if window.has_css_class("left") {
            gtk::PositionType::Right
        } else if window.has_css_class("right") {
            gtk::PositionType::Left
        } else {
            gtk::PositionType::Bottom
        }
    }

    fn cleanup_dropdown(&mut self) {
        if let Some(popover) = self.dropdown_popover.take() {
            popover.unparent();
        }
        self.dropdown_picker = None;
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
