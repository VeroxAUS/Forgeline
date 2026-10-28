// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Path hashing and the list of game files we need.
//!
//! Packs have no names: each entry is keyed by two murmur3 hashes (seed `0xFFFFFFFF`) of the
//! UTF-16 path, one over the lower-cased and one over the upper-cased path. Game paths start with
//! `natives/stm/` and end with a file-version number that changes between game versions, so a
//! path is looked up with every suffix from 0 to [`MAX_VERSION`] until both hashes match.

use crate::index::{EntryRef, PackSet};

pub const PREFIX: &str = "natives/stm/";
pub const MAX_VERSION: u32 = 70;

/// Plain murmur3 (x86, 32 bit) over bytes.
pub fn murmur3(data: &[u8], seed: u32) -> u32 {
    let (c1, c2) = (0xcc9e_2d51u32, 0x1b87_3593u32);
    let mut h = seed;
    let mut chunks = data.chunks_exact(4);
    for c in &mut chunks {
        let k = u32::from_le_bytes([c[0], c[1], c[2], c[3]])
            .wrapping_mul(c1)
            .rotate_left(15)
            .wrapping_mul(c2);
        h ^= k;
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    let tail = chunks.remainder();
    if !tail.is_empty() {
        let mut k = 0u32;
        for (i, b) in tail.iter().enumerate() {
            k |= u32::from(*b) << (8 * i);
        }
        h ^= k.wrapping_mul(c1).rotate_left(15).wrapping_mul(c2);
    }
    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

fn utf16(path: &str, upper: bool) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(path.len() * 2);
    for c in path.chars() {
        let c = if upper {
            c.to_ascii_uppercase()
        } else {
            c.to_ascii_lowercase()
        };
        let mut units = [0u16; 2];
        for u in c.encode_utf16(&mut units) {
            bytes.extend_from_slice(&u.to_le_bytes());
        }
    }
    bytes
}

/// The two hashes of a full path (for example `natives/stm/Boot/Master.scn.21`): lower-cased
/// first, upper-cased second.
pub fn hash_pair(path: &str) -> (u32, u32) {
    (
        murmur3(&utf16(path, false), 0xFFFF_FFFF),
        murmur3(&utf16(path, true), 0xFFFF_FFFF),
    )
}

/// Finds a game file by its path without prefix and version, for example
/// `GameDesign/Common/Equip/SkillData.user`. Returns the version suffix that matched and the
/// entry that wins across the packs.
pub fn resolve(set: &PackSet, stem: &str) -> Option<(u32, EntryRef)> {
    (0..=MAX_VERSION).find_map(|v| {
        let (lower, upper) = hash_pair(&format!("{PREFIX}{stem}.{v}"));
        set.entry(lower, upper).map(|e| (v, e))
    })
}

/// What a needed file is for. The tags follow the categories in `docs/research.md` (Phase 1,
/// step 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Skills,
    Weapons,
    Armor,
    Decorations,
    Charms,
    Artian,
    Items,
}

#[derive(Debug, Clone, Copy)]
pub struct NeededFile {
    /// Path below `natives/stm/`, without the version suffix.
    pub stem: &'static str,
    pub category: Category,
}

macro_rules! nf {
    ($cat:ident, $stem:expr) => {
        NeededFile {
            stem: $stem,
            category: Category::$cat,
        }
    };
}

/// Game files the planner needs, taken from the files the Phase 1 extraction test located and
/// named in `docs/research.md`. Paths only, no game data. Only names the notes give in full are
/// listed; the Artian and charm files whose exact location the notes leave open are added when a
/// phase needs them. A stem that does not resolve on a real install is reported by name.
pub const NEEDED: &[NeededFile] = &[
    nf!(Skills, "GameDesign/Common/Equip/SkillData.user"),
    nf!(Skills, "GameDesign/Common/Equip/SkillCommonData.user"),
    nf!(Skills, "GameDesign/Common/Equip/HunterActiveSkillData.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Bow.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/ChargeAxe.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/GunLance.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Hammer.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/HeavyBowgun.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Lance.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/LightBowgun.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/LongSword.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Tachi.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Rod.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/ShortSword.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/SlashAxe.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/TwinSword.user"),
    nf!(Weapons, "GameDesign/Common/Weapon/Whistle.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorData.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorSeriesData.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorRecipeData.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorUpgradeData.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorUpgradeRecipeData.user"),
    nf!(Armor, "GameDesign/Common/Equip/ArmorSpUpgradeCostData.user"),
    nf!(Armor, "GameDesign/Common/Equip/OuterArmorData.user"),
    nf!(Decorations, "GameDesign/Common/Equip/AccessoryData.user"),
    nf!(
        Decorations,
        "GameDesign/Common/Equip/AccessoryJudgeData.user"
    ),
    nf!(
        Decorations,
        "GameDesign/Common/Equip/AccessoryRankJudgeData.user"
    ),
    nf!(Charms, "GameDesign/Common/Equip/AmuletData.user"),
    nf!(Charms, "GameDesign/Common/Equip/AmuletRecipeData.user"),
    nf!(Artian, "GameDesign/Common/Equip/ArtianGrindingData.user"),
    nf!(Artian, "GameDesign/Common/Equip/ArtianSkillGroupData.user"),
    nf!(Items, "GameDesign/Common/Item/itemData.user"),
    nf!(Items, "GameDesign/Common/Item/ItemRecipe.user"),
    nf!(Items, "GameDesign/Common/Item/FixItems.user"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn murmur3_matches_published_vectors() {
        // Standard murmur3 x86 32-bit test vectors.
        assert_eq!(murmur3(b"", 0), 0);
        assert_eq!(murmur3(b"", 1), 0x514e_28b7);
        assert_eq!(murmur3(b"", 0xFFFF_FFFF), 0x81f1_6f39);
        assert_eq!(murmur3(b"test", 0), 0xba6b_d213);
        assert_eq!(murmur3(b"Hello, world!", 0), 0xc036_3e43);
        assert_eq!(
            murmur3(b"The quick brown fox jumps over the lazy dog", 0),
            0x2e4f_f723
        );
    }

    #[test]
    fn murmur3_matches_a_class_hash_confirmed_on_the_real_save() {
        // docs/research.md: the root class of the real save hashes to 827fccdc.
        assert_eq!(
            murmur3(b"app.savedata.cUserSaveData", 0xFFFF_FFFF),
            0x827f_ccdc
        );
    }

    #[test]
    fn path_hashes_differ_by_case_only_through_the_two_hashes() {
        let a = hash_pair("natives/stm/Boot/Master.scn.21");
        let b = hash_pair("NATIVES/STM/BOOT/MASTER.SCN.21");
        assert_eq!(a, b);
        assert_ne!(a.0, a.1);
        // Non-ASCII letters are left alone, and a character outside the BMP is two units.
        assert_ne!(hash_pair("a\u{1F600}"), hash_pair("a"));
    }

    #[test]
    fn needed_list_has_no_duplicates_and_no_prefix_or_version() {
        let mut seen = std::collections::HashSet::new();
        for n in NEEDED {
            assert!(seen.insert(n.stem), "duplicate {}", n.stem);
            assert!(!n.stem.starts_with(PREFIX), "{}", n.stem);
            assert!(n.stem.ends_with(".user"), "{}", n.stem);
        }
        assert!(NEEDED.len() >= 30);
    }
}
