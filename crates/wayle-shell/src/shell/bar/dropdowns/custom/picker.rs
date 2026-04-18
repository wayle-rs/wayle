use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use serde::Deserialize;
use tokio::process::Command;
use wayle_config::schemas::dropdowns::PickerSectionConfig;
use wayle_widgets::primitives::popover::PopoverItem;

/// Parsed item from the list command output.
#[derive(Debug, Clone, Default, Deserialize)]
pub(super) struct ListItem {
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    active: bool,
}

pub(super) struct PickerSection {
    config: PickerSectionConfig,
    items: FactoryVecDeque<PopoverItem>,
    /// Parsed item values, parallel to factory indices, for selection lookup.
    values: Vec<String>,
    load_generation: u64,
}

pub(super) struct PickerSectionInit {
    pub config: PickerSectionConfig,
}

#[derive(Debug)]
pub(super) enum PickerMsg {
    /// Popover became visible — reload the list.
    Reload,
    /// User activated a row.
    RowActivated(u32),
}

#[derive(Debug)]
pub(super) enum PickerCmd {
    /// List command completed.
    ListLoaded {
        items: Vec<ListItem>,
        generation: u64,
    },
    /// Select command completed — refresh the list.
    SelectDone,
}

#[relm4::component(pub(super))]
impl Component for PickerSection {
    type Init = PickerSectionInit;
    type Input = PickerMsg;
    type Output = ();
    type CommandOutput = PickerCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "custom-picker-section",

            #[local_ref]
            list_box -> gtk::ListBox {
                add_css_class: "custom-picker-list",
                set_selection_mode: gtk::SelectionMode::None,
                set_activate_on_single_click: true,
                connect_row_activated[sender] => move |_, row: &gtk::ListBoxRow| {
                    sender.input(PickerMsg::RowActivated(row.index() as u32));
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let items = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .detach();

        let model = Self {
            config: init.config,
            items,
            values: Vec::new(),
            load_generation: 0,
        };

        let list_box = model.items.widget();

        let widgets = view_output!();

        // Pre-load items so the dropdown has content on first show.
        sender.input(PickerMsg::Reload);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            PickerMsg::Reload => {
                self.load_generation += 1;
                let generation = self.load_generation;

                if let Some(cmd) = &self.config.list_command {
                    tracing::debug!(command = %cmd, "picker: loading list");
                    let cmd = cmd.clone();
                    sender.oneshot_command(async move {
                        let items = run_list_command(&cmd).await;
                        tracing::debug!(count = items.len(), "picker: list loaded");
                        PickerCmd::ListLoaded { items, generation }
                    });
                } else {
                    tracing::warn!("picker: no list-command configured");
                }
            }
            PickerMsg::RowActivated(index) => {
                let Some(value) = self.values.get(index as usize) else {
                    return;
                };
                let value = value.clone();

                if let Some(cmd) = &self.config.select_command {
                    let cmd = cmd.clone();
                    sender.oneshot_command(async move {
                        run_select_command(&cmd, &value).await;
                        PickerCmd::SelectDone
                    });
                }
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
            PickerCmd::ListLoaded { items, generation } => {
                // Discard stale results from previous loads.
                if generation != self.load_generation {
                    return;
                }
                self.apply_items(items);
            }
            PickerCmd::SelectDone => {
                // Close the parent popover, which triggers on_next_close
                // in the custom module to refresh the bar label.
                if let Some(popover) = root
                    .ancestor(gtk::Popover::static_type())
                    .and_then(|w| w.downcast::<gtk::Popover>().ok())
                {
                    popover.popdown();
                }
            }
        }
    }
}

impl PickerSection {
    fn apply_items(&mut self, items: Vec<ListItem>) {
        let mut guard = self.items.guard();
        guard.clear();
        self.values.clear();

        for item in items {
            let ListItem {
                value,
                label,
                subtitle,
                icon,
                active,
            } = item;

            let label = label.or_else(|| value.clone()).unwrap_or_default();
            let value = value.unwrap_or_else(|| label.clone());

            self.values.push(value);

            guard.push_back(PopoverItem {
                icon,
                label,
                subtitle,
                active_icon: Some("tb-check-symbolic".to_string()),
                is_active: active,
            });
        }
    }
}

async fn run_list_command(command: &str) -> Vec<ListItem> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    let Ok(output) = output else {
        tracing::warn!(command, "custom picker list command failed");
        return Vec::new();
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_list_output(&stdout)
}

fn parse_list_output(output: &str) -> Vec<ListItem> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let trimmed = line.trim();
            // Try JSON first.
            if trimmed.starts_with('{')
                && let Ok(item) = serde_json::from_str::<ListItem>(trimmed)
            {
                return item;
            }
            // Plain text fallback.
            ListItem {
                value: Some(trimmed.to_string()),
                label: Some(trimmed.to_string()),
                ..Default::default()
            }
        })
        .collect()
}

async fn run_select_command(command: &str, selected: &str) {
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("WAYLE_SELECTED", selected)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .status()
        .await;

    if let Err(err) = result {
        tracing::warn!(command, %err, "custom picker select command failed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_json_lines() {
        let output = r#"{"value": "1.0", "label": "1.0x", "subtitle": "96 DPI", "active": true}
{"value": "1.5", "label": "1.5x", "subtitle": "64 DPI", "active": false}
"#;
        let items = parse_list_output(output);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].value.as_deref(), Some("1.0"));
        assert_eq!(items[0].label.as_deref(), Some("1.0x"));
        assert!(items[0].active);
        assert!(!items[1].active);
    }

    #[test]
    fn parse_plain_text_lines() {
        let output = "context-a\ncontext-b\ncontext-c\n";
        let items = parse_list_output(output);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].value.as_deref(), Some("context-a"));
        assert_eq!(items[0].label.as_deref(), Some("context-a"));
        assert!(!items[0].active);
    }

    #[test]
    fn parse_skips_empty_lines() {
        let output = "a\n\n  \nb\n";
        let items = parse_list_output(output);
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn parse_mixed_json_and_text() {
        let output = r#"{"value": "x", "label": "X", "active": true}
plain-text-item
"#;
        let items = parse_list_output(output);
        assert_eq!(items.len(), 2);
        assert!(items[0].active);
        assert_eq!(items[1].label.as_deref(), Some("plain-text-item"));
    }
}
