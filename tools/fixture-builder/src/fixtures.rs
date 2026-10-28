// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The invented content of the fixture saves and the expected values the reader must find.
//!
//! Everything here is made up: a fake Steam account, round numbers, no hunter ID that belongs to
//! anyone. Class names that come from the real save are marked in `fixtures/README.md`; field
//! names and layouts follow `docs/PLAN.md` section 3 and the Phase 1 notes. Fields marked `*` in
//! the README are invented and may not exist in the real save.

use crate::crypto::{build_file, deflate_wrapper, STEAM_ID_BASE};
use crate::stream::{write_stream, Class, Value};
use serde_json::{json, Value as Json};
use std::collections::BTreeMap;

/// The fake Steam account number every fixture uses.
pub const ACCOUNT_ID: u64 = 4242;

pub struct EquipEntry {
    pub category: u8,
    pub id: u32,
}

#[derive(Default)]
pub struct Slot {
    pub active: bool,
    pub used: bool,
    pub name: &'static str,
    pub hunter_id: &'static str,
    pub equip: Vec<EquipEntry>,
    /// (item id, count); a count of 0 means the box entry is empty.
    pub box_items: Vec<(i32, i32)>,
    pub pouch: Vec<(i32, i32)>,
    pub found_flags: Vec<u32>,
    /// (fixed id, kills, captures)
    pub bosses: Vec<(i32, i32, i32)>,
}

pub struct Fixture {
    pub name: &'static str,
    pub description: &'static str,
    pub slots: [Slot; 3],
}

/// Equipment category 141 marks an empty box entry.
const EMPTY_CATEGORY: u8 = 141;

fn work(e: &EquipEntry) -> Class {
    Class::new("app.savedata.cEquipWork")
        .with("Category_Gender", Value::U8(e.category))
        .with("EquipIdFixed", Value::U32(e.id))
        .with("BonusByCreating", Value::U32(0))
        .with("BonusByGrinding", Value::U32(0))
        .with("GrindingNum", Value::U8(0))
        .with("BowgunCustomizeId", Value::ArrayS16(vec![-1; 4]))
        .with("EquipmentAccessoryIdArray", Value::ArrayS16(vec![-1; 3]))
}

fn item_work(i: &(i32, i32)) -> Class {
    Class::new("app.savedata.cItemWork")
        .with("ItemIdFixed", Value::S32(i.0))
        .with("Num", Value::S32(i.1))
}

fn slot_class(s: &Slot) -> Class {
    Class::new("app.savedata.cUserSaveParam")
        .with("Active", Value::U8(u8::from(s.active)))
        .with(
            "_LastSaveStamp",
            Value::F32(if s.used { 1.0 } else { -1.0 }),
        )
        .with(
            "_BasicData",
            Value::Class(
                Class::new("app.savedata.cBasicParam")
                    .with("_CharName", Value::Str(s.name.to_string()))
                    .with("_HunterId", Value::Str(s.hunter_id.to_string()))
                    .with("_PlayTime", Value::U32(if s.used { 3600 } else { 0 }))
                    .with("_Created", Value::Bool(s.used))
                    .with("_Anchor", Value::Struct16([0x11; 16])),
            ),
        )
        .with(
            "_Equip",
            Value::Class(Class::new("app.savedata.cEquipParam").with(
                "_EquipBox",
                Value::ArrayClass(s.equip.iter().map(work).collect()),
            )),
        )
        .with(
            "_Item",
            Value::Class(
                Class::new("app.savedata.cItemParam")
                    .with(
                        "_BoxItem",
                        Value::ArrayClass(s.box_items.iter().map(item_work).collect()),
                    )
                    .with(
                        "_PouchItem",
                        Value::ArrayClass(s.pouch.iter().map(item_work).collect()),
                    )
                    .with(
                        "_ItemFoundFlag",
                        Value::Class(
                            Class::new("app.savedata.cFlagParam")
                                .with("_Value", Value::ArrayU32(s.found_flags.clone())),
                        ),
                    ),
            ),
        )
        .with(
            "_EnemyReport",
            Value::Class(
                Class::new("app.savedata.cEnemyReportParam").with(
                    "_Boss",
                    Value::ArrayClass(
                        s.bosses
                            .iter()
                            .map(|b| {
                                Class::new("app.savedata.cBossReportParam")
                                    .with("FixedId", Value::S32(b.0))
                                    .with("SlayingNum", Value::S32(b.1))
                                    .with("CaptureNum", Value::S32(b.2))
                            })
                            .collect(),
                    ),
                ),
            ),
        )
}

pub fn stream_for(f: &Fixture) -> Vec<u8> {
    let root = Class::new("app.savedata.cUserSaveData").with(
        "_Data",
        Value::ArrayClass(f.slots.iter().map(slot_class).collect()),
    );
    write_stream(&[root])
}

pub fn save_bytes(f: &Fixture) -> Vec<u8> {
    build_file(&deflate_wrapper(&stream_for(f)), STEAM_ID_BASE + ACCOUNT_ID)
}

/// Light bowgun entries (category 13) with the given ids.
fn weapons(ids: &[u32]) -> std::vec::IntoIter<EquipEntry> {
    ids.iter()
        .map(|&id| EquipEntry { category: 13, id })
        .collect::<Vec<_>>()
        .into_iter()
}

fn empty_slot() -> Slot {
    Slot::default()
}

pub fn all() -> Vec<Fixture> {
    let hunter = Slot {
        active: true,
        used: true,
        name: "Fixture Hunter",
        hunter_id: "FIXT-0001-NOPE",
        equip: weapons(&[0, 1, 2])
            .chain([
                EquipEntry { category: 1, id: 4 },
                EquipEntry {
                    category: 1,
                    id: 25,
                },
                EquipEntry {
                    category: 21,
                    id: 57,
                },
                EquipEntry {
                    category: EMPTY_CATEGORY,
                    id: 0,
                },
                EquipEntry {
                    category: EMPTY_CATEGORY,
                    id: 0,
                },
            ])
            .collect(),
        box_items: vec![(1, 10), (2, 5), (5, 99), (805, 1), (7, 0)],
        pouch: vec![(1, 3), (2, 2)],
        found_flags: vec![0b1011, 0, 0x8000_0000],
        bosses: vec![(1, 3, 0), (2, 0, 0), (3, 5, 1)],
    };
    let second = Slot {
        active: false,
        used: true,
        name: "Fixture Alt",
        hunter_id: "FIXT-0002-NOPE",
        equip: weapons(&[7])
            .chain([EquipEntry { category: 1, id: 9 }])
            .collect(),
        box_items: vec![(3, 1)],
        pouch: vec![],
        found_flags: vec![1],
        bosses: vec![(1, 1, 0)],
    };
    let third = Slot {
        active: true,
        used: true,
        name: "Fixture Third",
        hunter_id: "FIXT-0003-NOPE",
        equip: weapons(&[3, 4])
            .chain([
                EquipEntry {
                    category: 21,
                    id: 12,
                },
                EquipEntry {
                    category: EMPTY_CATEGORY,
                    id: 0,
                },
            ])
            .collect(),
        box_items: vec![(10, 2), (11, 4)],
        pouch: vec![(10, 1)],
        found_flags: vec![0xF, 0xF0],
        bosses: vec![(4, 2, 1), (5, 0, 0)],
    };

    vec![
        Fixture {
            name: "one-slot",
            description: "Slot 1 holds data and is the active one; slots 2 and 3 are empty.",
            slots: [hunter, empty_slot(), empty_slot()],
        },
        Fixture {
            name: "two-slots",
            description: "Slots 1 and 3 hold data; slot 3 is the active one, so picking the first slot with data is wrong.",
            slots: [second, empty_slot(), third],
        },
    ]
}

fn slot_expected(index: usize, s: &Slot) -> Json {
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for e in s.equip.iter().filter(|e| e.category != EMPTY_CATEGORY) {
        *by_category.entry(e.category.to_string()).or_default() += 1;
    }
    json!({
        "slot": index + 1,
        "has_data": s.used,
        "active": s.active,
        "equipment_in_use": s.equip.iter().filter(|e| e.category != EMPTY_CATEGORY).count(),
        "equipment_by_category": by_category,
        "box_item_types_in_use": s.box_items.iter().filter(|i| i.1 > 0).count(),
        "pouch_entries": s.pouch.len(),
        "found_flag_bits_set": s.found_flags.iter().map(|w| w.count_ones()).sum::<u32>(),
        "boss_entries": s.bosses.len(),
        "bosses_with_kills": s.bosses.iter().filter(|b| b.1 > 0).count(),
    })
}

/// The values the reader must report, plus the strings that must never appear in its summary.
pub fn expected_json(f: &Fixture) -> String {
    let never: Vec<&str> = f
        .slots
        .iter()
        .filter(|s| s.used)
        .flat_map(|s| [s.name, s.hunter_id])
        .collect();
    let value = json!({
        "fixture": f.name,
        "description": f.description,
        "synthetic": true,
        "steam_account_id": ACCOUNT_ID,
        "steam_id64": STEAM_ID_BASE + ACCOUNT_ID,
        "slot_count": 3,
        "slots_with_data": f.slots.iter().filter(|s| s.used).count(),
        "active_slot": f.slots.iter().position(|s| s.active).map(|i| i + 1),
        "slots": f.slots.iter().enumerate().map(|(i, s)| slot_expected(i, s)).collect::<Vec<_>>(),
        "never_in_summary": never,
    });
    let mut s = serde_json::to_string_pretty(&value).expect("expected values serialise");
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::tests::{decrypt, inflate};
    use crate::stream::tests::walk;

    #[test]
    fn every_fixture_decrypts_inflates_and_walks_to_the_end() {
        for f in all() {
            let file = save_bytes(&f);
            let stream = inflate(&decrypt(&file, STEAM_ID_BASE + ACCOUNT_ID));
            assert_eq!(stream, stream_for(&f));
            // root + 3 slots + per slot: basic, equip, item, flag, enemy report, plus entries.
            let want: usize = 1 + f
                .slots
                .iter()
                .map(|s| 1 + 5 + s.equip.len() + s.box_items.len() + s.pouch.len() + s.bosses.len())
                .sum::<usize>();
            assert_eq!(walk(&stream), want, "{}", f.name);
        }
    }

    #[test]
    fn building_twice_gives_the_same_bytes() {
        for f in all() {
            assert_eq!(save_bytes(&f), save_bytes(&f));
        }
    }

    #[test]
    fn only_one_slot_is_active_and_expected_values_agree() {
        for f in all() {
            assert_eq!(f.slots.iter().filter(|s| s.active).count(), 1);
            let v: Json = serde_json::from_str(&expected_json(&f)).unwrap();
            assert_eq!(v["slot_count"], 3);
            assert!(v["slots_with_data"].as_u64().unwrap() >= 1);
        }
        let one = all().remove(0);
        let v: Json = serde_json::from_str(&expected_json(&one)).unwrap();
        assert_eq!(v["slots"][0]["equipment_in_use"], 6);
        assert_eq!(v["slots"][0]["equipment_by_category"]["13"], 3);
        assert_eq!(v["slots"][0]["box_item_types_in_use"], 4);
        assert_eq!(v["slots"][0]["found_flag_bits_set"], 4);
        assert_eq!(v["slots"][0]["bosses_with_kills"], 2);
    }

    #[test]
    fn hunter_name_and_id_never_appear_in_the_plain_expected_values_except_the_never_list() {
        let one = all().remove(0);
        let json = expected_json(&one);
        let hits = json.matches("FIXT-0001-NOPE").count();
        assert_eq!(hits, 1, "only in never_in_summary");
    }
}
