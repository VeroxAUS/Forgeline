// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reads the committed fixture saves back and compares with the expected values the builder wrote
//! next to them. Also checks that bad input fails cleanly.

use forgeline_save::model::{read_save, EquipKind, SaveSummary};
use forgeline_save::{container, SaveError};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves")
}

fn load(name: &str) -> (Vec<u8>, Value) {
    let bin = std::fs::read(fixtures().join(format!("{name}.bin"))).unwrap();
    let expected: Value = serde_json::from_slice(
        &std::fs::read(fixtures().join(format!("{name}.expected.json"))).unwrap(),
    )
    .unwrap();
    (bin, expected)
}

fn steam_id(expected: &Value) -> u64 {
    expected["steam_id64"].as_u64().unwrap()
}

/// The summary in the same shape as the expected-values file.
fn as_expected(s: &SaveSummary) -> Value {
    let slots: Vec<Value> = s
        .slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
            for e in &slot.equipment {
                *by_category.entry(e.category.to_string()).or_default() += 1;
            }
            let strmap = |m: BTreeMap<i32, usize>| -> BTreeMap<String, usize> {
                m.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
            };
            json!({
                "slot": i + 1,
                "has_data": slot.has_data(),
                "active": slot.active_flag,
                "equipment_in_use": slot.equipment.len(),
                "equipment_by_category": by_category,
                "weapons_by_type": strmap(slot.weapons_by_type()),
                "armor_entries": slot.count_of(|k| matches!(k, EquipKind::Armor { .. })),
                "armor_entries_by_piece": strmap(slot.armor_entries_by_piece()),
                "armor_distinct_series_pieces": slot.armor_distinct_series_pieces(),
                "talismans": slot.count_of(|k| matches!(k, EquipKind::Talisman { .. })),
                "mantles": slot.count_of(|k| matches!(k, EquipKind::Mantle { .. })),
                "box_item_types_in_use": slot.box_items.len(),
                "pouch_entries": slot.pouch.len(),
                "found_flag_bits_set": slot.found_flag_bits_set,
                "boss_entries": slot.bosses.len(),
                "bosses_with_kills": slot.bosses_with_kills(),
            })
        })
        .collect();
    json!({ "slots": slots })
}

#[test]
fn every_fixture_reads_back_with_the_expected_values() {
    for name in ["one-slot", "two-slots"] {
        let (bin, expected) = load(name);
        let summary = read_save(&bin, steam_id(&expected)).unwrap();
        assert_eq!(
            as_expected(&summary)["slots"],
            expected["slots"],
            "{name}: slots"
        );
        assert_eq!(summary.slots.len() as u64, expected["slot_count"]);
        assert_eq!(
            summary.slots.iter().filter(|s| s.has_data()).count() as u64,
            expected["slots_with_data"]
        );
        assert_eq!(
            summary.active_slot.map(|i| i as u64 + 1),
            expected["active_slot"].as_u64(),
            "{name}: active slot"
        );
    }
}

#[test]
fn the_active_slot_is_the_flagged_one_not_the_first_with_data() {
    let (bin, expected) = load("two-slots");
    let summary = read_save(&bin, steam_id(&expected)).unwrap();
    assert_eq!(summary.active_slot, Some(2));
    assert!(summary.slots[0].has_data());
}

#[test]
fn the_summary_holds_no_hunter_name_or_id() {
    for name in ["one-slot", "two-slots"] {
        let (bin, expected) = load(name);
        let summary = read_save(&bin, steam_id(&expected)).unwrap();
        let text = format!("{summary:?}");
        for never in expected["never_in_summary"].as_array().unwrap() {
            assert!(!text.contains(never.as_str().unwrap()), "{name}: {never}");
        }
    }
}

#[test]
fn a_wrong_steam_id_is_reported_as_such() {
    let (bin, expected) = load("one-slot");
    let err = read_save(&bin, steam_id(&expected) + 1).unwrap_err();
    assert_eq!(err, SaveError::WrongSteamId);
}

#[test]
fn damaged_files_fail_cleanly_and_never_panic() {
    let (bin, expected) = load("one-slot");
    let id = steam_id(&expected);

    // Cut short, at several places.
    for cut in [0, 3, 15, 100, bin.len() / 2, bin.len() - 1] {
        assert!(read_save(&bin[..cut], id).is_err(), "cut at {cut}");
    }

    // One flipped bit in every 4 KiB of the file: it may decrypt to garbage, but must not panic.
    for at in (0..bin.len()).step_by(4096) {
        let mut copy = bin.clone();
        copy[at] ^= 0x10;
        let _ = read_save(&copy, id);
    }
}

#[test]
fn garbage_streams_fail_cleanly() {
    use forgeline_save::stream::Stream;
    // A deterministic spread of nonsense, plus truncations of a real stream.
    let mut x = 0x1234_5678_u32;
    for len in [0usize, 1, 7, 8, 9, 63, 64, 1000] {
        let junk: Vec<u8> = (0..len)
            .map(|_| {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (x >> 24) as u8
            })
            .collect();
        let _ = Stream::new(&junk).check();
        let _ = forgeline_save::model::summarize(&junk);
    }
    let (bin, expected) = load("two-slots");
    let stream = container::open(&bin, steam_id(&expected)).unwrap();
    assert!(Stream::new(&stream).check().is_ok());
    for cut in (0..stream.len()).step_by(7) {
        let _ = forgeline_save::model::summarize(&stream[..cut]);
    }
}

#[test]
fn collected_hashes_include_the_classes_and_fields_the_reader_looks_up() {
    use forgeline_save::hash::name_hash;
    use forgeline_save::hashes::collect;
    let (bin, expected) = load("one-slot");
    let stream = container::open(&bin, steam_id(&expected)).unwrap();
    let h = collect(&stream).unwrap();
    assert!(h.classes.contains(&name_hash("app.savedata.cUserSaveData")));
    assert!(h.classes.contains(&name_hash("app.savedata.cEquipWork")));
    assert!(h.fields.contains(&name_hash("_Data")));
    assert!(h.fields.contains(&name_hash("Active")));
    // Field and class sets are separate, and `all` is their union.
    assert!(!h.classes.contains(&name_hash("_Data")));
    assert_eq!(h.all().len(), h.classes.len() + h.fields.len());
    // Garbage fails cleanly.
    assert!(collect(&[1, 2, 3, 4, 5, 6, 7, 8, 9]).is_err());
}
