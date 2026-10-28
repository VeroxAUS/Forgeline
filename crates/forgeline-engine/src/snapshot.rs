// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The `Snapshot`: everything the window draws, in one plain value. The window reads the latest
//! one and never touches a file. It holds derived counts only: no hunter name or ID.

use forgeline_core::discovery::Discovered;
use forgeline_save::model::SaveSummary;
use forgeline_update::manifest::Manifest;
use forgeline_update::status::LastOutcome;
use serde::{Deserialize, Serialize};

/// Size and modified time of the save file, used to tell whether it changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    pub size: u64,
    /// Nanoseconds since 1970; 0 if the system did not say.
    pub modified_ns: u64,
}

/// Where things were found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundView {
    pub steam_root: String,
    pub library: String,
    pub install: String,
    pub build_id: String,
    /// Steam account folders that have Wilds saves (normally one).
    pub accounts: usize,
    /// The account folder number used, when one was chosen.
    pub account_id: Option<u32>,
    pub mod_markers: Vec<String>,
}

impl FoundView {
    pub(crate) fn new(d: &Discovered, account_id: Option<u32>) -> FoundView {
        FoundView {
            steam_root: d.steam_root.display().to_string(),
            library: d.library.display().to_string(),
            install: d.install.display().to_string(),
            build_id: d.build_id.clone(),
            accounts: d.accounts.len(),
            account_id,
            mod_markers: d.mod_markers.clone(),
        }
    }
}

/// One save slot as counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotView {
    pub has_data: bool,
    /// The slot's own `Active` field.
    pub active_flag: bool,
    pub equipment: usize,
    pub weapons: usize,
    /// Armour entries in the box (each look of a piece counts).
    pub armor_entries: usize,
    /// Armour pieces counted once (the A and B looks are the same piece).
    pub armor_pieces: usize,
    pub box_item_types: usize,
    pub pouch_item_types: usize,
    pub bosses_with_kills: usize,
    pub found_flag_bits_set: u32,
}

/// The save, summarised.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveView {
    pub stamp: Stamp,
    /// Index of the active slot, when exactly one slot is flagged.
    pub active_slot: Option<usize>,
    pub slots: Vec<SlotView>,
}

impl SaveView {
    pub(crate) fn new(s: &SaveSummary, stamp: Stamp) -> SaveView {
        let slots = s
            .slots
            .iter()
            .map(|slot| SlotView {
                has_data: slot.has_data(),
                active_flag: slot.active_flag,
                equipment: slot.equipment.len(),
                weapons: slot.weapons_by_type().values().sum(),
                armor_entries: slot.armor_entries_by_piece().values().sum(),
                armor_pieces: slot.armor_distinct_series_pieces(),
                box_item_types: slot.box_items.len(),
                pouch_item_types: slot.pouch.len(),
                bosses_with_kills: slot.bosses_with_kills(),
                found_flag_bits_set: slot.found_flag_bits_set,
            })
            .collect();
        SaveView {
            stamp,
            active_slot: s.active_slot,
            slots,
        }
    }
}

/// The current extraction, in short.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestView {
    pub build_id: String,
    pub created: String,
    pub files: usize,
    pub missing: usize,
    pub classes: usize,
}

impl ManifestView {
    pub(crate) fn new(m: &Manifest) -> ManifestView {
        ManifestView {
            build_id: m.build_id.clone(),
            created: m.created.clone(),
            files: m.files.len(),
            missing: m.missing.len(),
            classes: m.classes.len(),
        }
    }
}

/// The game is on a different build than the data was extracted from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatchNotice {
    pub game_build: String,
    pub data_build: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataView {
    pub manifest: Option<ManifestView>,
    /// Why the current extraction cannot be used (present but unreadable); `None` if it is just
    /// missing or fine.
    pub manifest_error: Option<String>,
    pub patch_notice: Option<PatchNotice>,
    pub last_update: Option<LastOutcome>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// Counts up with every published snapshot. Not stored in the cache.
    #[serde(skip)]
    pub version: u64,
    /// True while this view is from the cache and not yet checked against the save as it is now.
    #[serde(skip)]
    pub refreshing: bool,
    /// Set when the save folder could not be watched, so changes are not picked up by themselves.
    #[serde(skip)]
    pub watch_error: Option<String>,
    pub found: Option<FoundView>,
    /// Why nothing was found (Steam or the game missing).
    pub found_error: Option<String>,
    /// The last good summary of the save. Kept when a later read fails.
    pub save: Option<SaveView>,
    /// Why the save could not be read just now. While it is set, `save` is the older good one.
    pub save_error: Option<String>,
    pub data: DataView,
}

impl Snapshot {
    /// What the window shows before anything is known.
    pub fn loading() -> Snapshot {
        Snapshot {
            version: 0,
            refreshing: true,
            watch_error: None,
            found: None,
            found_error: None,
            save: None,
            save_error: None,
            data: DataView::default(),
        }
    }

    /// True if both show the same content (the version and refresh flag do not count).
    pub fn same_view(&self, other: &Snapshot) -> bool {
        let (mut a, mut b) = (self.clone(), other.clone());
        a.version = 0;
        b.version = 0;
        a.refreshing = false;
        b.refreshing = false;
        a == b
    }
}
