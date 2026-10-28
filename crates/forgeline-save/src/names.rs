// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Turns class and field hashes back into names, for reports. The reader itself never needs this:
//! it looks fields up by hashing the names it wants. A hash with no name prints as `#xxxxxxxx`.

use crate::hash::name_hash;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct TypeNames {
    map: HashMap<u32, String>,
}

impl TypeNames {
    pub fn new() -> TypeNames {
        TypeNames::default()
    }

    /// Adds a name (a class such as `app.savedata.cEquipWork`, or a field such as `_EquipBox`).
    pub fn add(&mut self, name: &str) {
        self.map.insert(name_hash(name), name.to_string());
    }

    /// One name per line; blank lines and lines starting with `#` are skipped.
    pub fn add_lines(&mut self, text: &str) {
        for line in text.lines().map(str::trim) {
            if !line.is_empty() && !line.starts_with('#') {
                self.add(line);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&self, hash: u32) -> Option<&str> {
        self.map.get(&hash).map(String::as_str)
    }

    /// The name, or `#` and the hash in hex.
    pub fn show(&self, hash: u32) -> String {
        match self.get(hash) {
            Some(n) => n.to_string(),
            None => format!("#{hash:08x}"),
        }
    }
}

/// Class names found in the real save in Phase 1 (`docs/research.md`, step 3 table and the list
/// after it).
const KNOWN_CLASSES: &[&str] = &[
    "via.storage.saveService.SaveFileDetail",
    "app.savedata.cUserSaveData",
    "app.savedata.cUserSaveParam",
    "app.savedata.cEquipParam",
    "app.savedata.cItemParam",
    "app.savedata.cBossReportParam",
    "app.savedata.cStoryParam",
    "app.savedata.cMissionParam",
    "app.savedata.cLargeWorkshopParam",
    "app.savedata.cEquipWork",
    "app.savedata.cEquipMyset",
    "app.savedata.cItemWork",
    "app.savedata.cCharmEquipped",
    "app.savedata.cArtianPartsWork",
    "app.savedata.cArtianCreateCountParam",
    "app.savedata.cWeaponFlagParam",
    "app.savedata.cArmorFlagParam",
    "app.savedata.cOtEquipWork",
];

impl TypeNames {
    /// The names the reader uses plus the class names already known from Phase 1. Step 5 replaces
    /// this with the table rebuilt from the game's `.exe`.
    pub fn builtin() -> TypeNames {
        let mut t = TypeNames::new();
        for n in crate::model::n::ALL.iter().chain(KNOWN_CLASSES) {
            t.add(n);
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_names_resolve_and_unknown_ones_show_the_hash() {
        let mut t = TypeNames::new();
        t.add_lines("# comment\n\n_EquipBox\n  app.savedata.cEquipWork \n");
        assert_eq!(t.len(), 2);
        assert_eq!(t.show(name_hash("_EquipBox")), "_EquipBox");
        assert_eq!(t.show(0x1234_abcd), "#1234abcd");
    }
}
