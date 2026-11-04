use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use relm4::{
    factory::FactoryVecDequeGuard,
    gtk,
    prelude::{DynamicIndex, *},
};
use wayle_bluetooth::{
    core::device::Device,
    types::{RadioBlock, agent::PairingRequest},
};
use wayle_widgets::WatcherToken;
use zbus::zvariant::OwnedObjectPath;

use super::{
    BluetoothDropdown, SCAN_DURATION,
    device_item::{ActionsLayout, DeviceItem, messages::DeviceItemInit},
    helpers::{DeviceSnapshot, build_split_device_lists, is_listed},
    pairing_card::messages::PairingCardMsg,
    watchers,
};

impl BluetoothDropdown {
    /// Whether a Bluetooth adapter is present.
    pub(super) fn available(&self) -> bool {
        self.bluetooth
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.available.get())
    }

    /// Whether Bluetooth is on or turning on, which the switch shows.
    pub(super) fn enabled(&self) -> bool {
        self.bluetooth
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.enabled.get())
    }

    /// Whether Bluetooth is on: the lists show only while BlueZ can act on
    /// their devices.
    pub(super) fn powered(&self) -> bool {
        self.bluetooth
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.powered.get())
    }

    /// Whether a hardware switch blocks a Bluetooth radio, which keeps
    /// Bluetooth off (as KDE reckons it) whatever the switch does.
    pub(super) fn hardware_blocked(&self) -> bool {
        self.bluetooth
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.radio_block.get() == RadioBlock::Hardware)
    }

    /// Whether the adapter is discovering, for any client.
    pub(super) fn scanning(&self) -> bool {
        self.bluetooth
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.discovering.get())
    }

    /// Starts (or extends) a timed scan. `scanning` follows the service's
    /// `discovering`.
    pub(super) fn handle_scan_requested(&self) {
        if let Some(bluetooth) = &self.bluetooth {
            bluetooth.start_timed_discovery(SCAN_DURATION);
        }
    }

    /// A device list. Its rows act on their devices themselves.
    pub(super) fn build_device_list() -> FactoryVecDeque<DeviceItem> {
        FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach()
    }

    /// Keeps one property watcher per device: drops watchers for devices that
    /// are gone and spawns them only for new ones, so a device appearing or
    /// disappearing costs O(1) watchers rather than respawning all of them.
    ///
    /// Returns whether the lists need rebuilding: a listed device came, or a
    /// device with a row went. Unnamed devices coming and going don't affect
    /// them.
    pub(super) fn sync_device_watchers(&mut self, sender: &ComponentSender<Self>) -> bool {
        let Some(bluetooth) = &self.bluetooth else {
            return false;
        };

        let devices = bluetooth.devices.get();
        let current: HashMap<&OwnedObjectPath, &Arc<Device>> = devices
            .iter()
            .map(|device| (&device.object_path, device))
            .collect();

        // Also drop entries whose path now belongs to a different `Device`
        // (removed and re-added between syncs), so it gets a fresh watcher.
        let mut dropped = Vec::new();
        self.device_watchers.retain(|path, (watched, _)| {
            let keep = current
                .get(path)
                .is_some_and(|device| Arc::ptr_eq(device, watched));
            if !keep {
                dropped.push(path.clone());
            }
            keep
        });
        // Whether a dropped device has a row, not whether its last state lists
        // it: the change that unlisted it may not have reached the lists.
        let mut listed_changed = self.rows().any(|row| dropped.contains(row.device_path()));

        for device in &devices {
            if self.device_watchers.contains_key(&device.object_path) {
                continue;
            }

            let mut watcher = WatcherToken::new();
            watchers::spawn_device_watcher(sender, device, watcher.reset());
            // Checked only after the watcher has read its baselines: a change
            // making the device listed is then either seen here or reported by
            // the watcher, never neither.
            listed_changed |= is_listed(&device.info.get());
            self.device_watchers
                .insert(device.object_path.clone(), (Arc::clone(device), watcher));
        }

        listed_changed
    }

    pub(super) fn rebuild_device_lists(&mut self) {
        let Some(bluetooth) = &self.bluetooth else {
            return;
        };

        let devices = bluetooth.devices.get();
        let shown_available = (0..self.available_devices.len())
            .filter_map(|idx| self.available_devices.get(idx))
            .map(|item| (item.device_path().clone(), item.index.current_index()))
            .collect();
        let lists = build_split_device_lists(&devices, &shown_available);
        let previous_slots = self.visible.then(|| self.row_slots());
        let had_my_devices = !self.my_devices.is_empty();

        reconcile_list(&mut self.my_devices.guard(), &lists.my_devices);
        reconcile_list(
            &mut self.available_devices.guard(),
            &lists.available_devices,
        );

        let has_my_devices = !self.my_devices.is_empty();

        if let Some(previous_slots) = previous_slots {
            // The "My devices" section appearing or going away moves every
            // row below it, whatever its slot.
            if had_my_devices != has_my_devices {
                self.guard_all_rows();
            } else {
                self.guard_shifted_rows(&previous_slots);
            }
        }

        // The card's device may have changed its name or icon too.
        if bluetooth.pairing_request.get().is_some() {
            self.pairing_card.emit(PairingCardMsg::Refresh);
        }
    }

    pub(super) fn handle_visibility_changed(&mut self, visible: bool) {
        self.visible = visible;
        if !visible {
            for row in self.rows() {
                row.disarm_click_guard();
            }
        }
    }

    /// Every row, top to bottom: "my devices", then the available list below.
    fn rows(&self) -> impl Iterator<Item = &DeviceItem> {
        let mine = (0..self.my_devices.len()).filter_map(|idx| self.my_devices.get(idx));
        let available =
            (0..self.available_devices.len()).filter_map(|idx| self.available_devices.get(idx));
        mine.chain(available)
    }

    /// Each row's overall slot on screen, across both lists, and its buttons.
    fn row_slots(&self) -> HashMap<OwnedObjectPath, (usize, ActionsLayout)> {
        self.rows()
            .enumerate()
            .map(|(slot, row)| (row.device_path().clone(), (slot, row.actions_layout())))
            .collect()
    }

    /// Guards every row now in a different slot than before, or showing
    /// different buttons, so a click aimed at whatever used to be there doesn't
    /// land on it. That includes rows shifted by others moving, being inserted
    /// or being removed. A new row is guarded only if its slot was already
    /// occupied: a row appended below the old last row (or into an empty list)
    /// replaced nothing.
    fn guard_shifted_rows(
        &self,
        previous_slots: &HashMap<OwnedObjectPath, (usize, ActionsLayout)>,
    ) {
        for (slot, row) in self.rows().enumerate() {
            let layout = row.actions_layout();
            let shifted = match previous_slots.get(row.device_path()) {
                Some(&(previous_slot, previous_layout)) => {
                    if previous_slot == slot && layout.only_dismissed(previous_layout) {
                        // The buttons now sit where Dismiss was; the row
                        // itself stays clickable to retry the connection.
                        row.arm_actions_guard();
                        continue;
                    }
                    previous_slot != slot || previous_layout != layout
                }
                None => slot < previous_slots.len(),
            };
            if shifted {
                row.arm_click_guard();
            }
        }
    }

    /// Guards every row, for layout changes above the lists (the pairing card
    /// appearing, changing or going away).
    fn guard_all_rows(&self) {
        if self.visible {
            for row in self.rows() {
                row.arm_click_guard();
            }
        }
    }

    /// Shows the service's pairing request. A new prompt (or none) moves the
    /// rows below the card, and its buttons may appear under the pointer.
    pub(super) fn handle_pairing_request(&self, request: Option<PairingRequest>, new_prompt: bool) {
        if new_prompt {
            self.guard_all_rows();
        }
        self.pairing_card.emit(PairingCardMsg::SetRequest {
            request,
            new_prompt,
            guard: new_prompt && self.visible,
        });
    }
}

/// Reconciles the factory in place within a single guard, keyed by device path.
///
/// Rows whose device is no longer listed are dropped. Then, walking the new
/// order from the top, each device's row is moved to its place (or inserted,
/// if new); the rows above it are already in place. Rows keep their widget,
/// and only a row whose snapshot differs is marked changed, so an unchanged
/// list renders nothing. (relm4 works out the widget moves from the final
/// order itself, so the sequence of moves doesn't matter.)
fn reconcile_list(
    guard: &mut FactoryVecDequeGuard<'_, DeviceItem>,
    new_snapshots: &[DeviceSnapshot],
) {
    let listed: HashSet<&OwnedObjectPath> = new_snapshots
        .iter()
        .map(|snapshot| &snapshot.device.object_path)
        .collect();

    for idx in (0..guard.len()).rev() {
        let still_listed = guard
            .get(idx)
            .is_some_and(|item| listed.contains(item.device_path()));
        if !still_listed {
            guard.remove(idx);
        }
    }

    let rows: HashMap<OwnedObjectPath, DynamicIndex> = (0..guard.len())
        .filter_map(|idx| guard.get(idx))
        .map(|item| (item.device_path().clone(), item.index.clone()))
        .collect();

    for (position, snapshot) in new_snapshots.iter().enumerate() {
        let Some(row) = rows.get(&snapshot.device.object_path) else {
            guard.insert(
                position,
                DeviceItemInit {
                    snapshot: snapshot.clone(),
                },
            );
            continue;
        };

        let current = row.current_index();
        if current != position {
            guard.move_to(current, position);
        }

        let needs_update = guard
            .get(position)
            .is_some_and(|item| item.differs_from(snapshot));
        if needs_update && let Some(item) = guard.get_mut(position) {
            item.update_from_snapshot(snapshot.clone());
        }
    }
}
