// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A typed summary of the save: the data the window needs and nothing else.
//!
//! Classes and fields are found by hashing the names in [`n`], so no type table is needed to read.
//! The one exception is an equipment box entry, whose field names are unknown: its first two
//! fields are read by position.
//! The hunter name, hunter IDs and everything else in `_BasicData` are never read, so they cannot
//! reach the summary. Numbers stay numbers: enum fields are read as plain integers.
//!
//! Names and layout come from `docs/research.md` (Phase 1 steps 3 and 4). The ones marked invented
//! in `fixtures/README.md` have not been seen in a real save yet; step 11 checks them on the
//! maintainer's save, and a name that is missing there shows up as a slot with no data, never as a
//! wrong number.

use crate::container;
use crate::hash::name_hash;
use crate::stream::{ClassView, Stream, Value};
use crate::SaveError;

/// The class and field names the reader looks for. Change a name here and nowhere else.
pub mod n {
    pub const ROOT: &str = "app.savedata.cUserSaveData";
    pub const SLOTS: &str = "_Data";
    pub const ACTIVE: &str = "Active";
    pub const EQUIP: &str = "_Equip";
    pub const EQUIP_BOX: &str = "_EquipBox";
    pub const ITEM: &str = "_Item";
    pub const BOX_ITEM: &str = "_BoxItem";
    pub const POUCH_ITEM: &str = "_PouchItem";
    pub const FOUND_FLAG: &str = "_ItemFoundFlag";
    pub const FLAG_VALUE: &str = "_Value";
    pub const ITEM_ID: &str = "ItemIdFixed";
    pub const ITEM_NUM: &str = "Num";
    pub const ENEMY_REPORT: &str = "_EnemyReport";
    pub const BOSS: &str = "_Boss";
    pub const BOSS_ID: &str = "FixedId";
    pub const BOSS_KILLS: &str = "SlayingNum";
    pub const BOSS_CAPTURES: &str = "CaptureNum";
    /// Where the hunter's name and IDs live. Never read by the model; reports skip it.
    pub const BASIC_DATA: &str = "_BasicData";

    /// Every name above that is a class or field, for building a [`crate::names::TypeNames`].
    pub const ALL: &[&str] = &[
        ROOT,
        SLOTS,
        ACTIVE,
        EQUIP,
        EQUIP_BOX,
        ITEM,
        BOX_ITEM,
        POUCH_ITEM,
        FOUND_FLAG,
        FLAG_VALUE,
        ITEM_ID,
        ITEM_NUM,
        ENEMY_REPORT,
        BOSS,
        BOSS_ID,
        BOSS_KILLS,
        BOSS_CAPTURES,
        BASIC_DATA,
    ];
}

/// Equipment category value that marks an empty box entry.
pub const EMPTY_CATEGORY: u8 = 141;

/// Equipment categories seen in the maintainer's save, checked against their equipment box
/// screenshots on 2026-10-28 (`docs/research.md`, Phase 2 step 4 notes).
pub mod category {
    /// Armour, look "A" (the green dot in the game's box).
    pub const ARMOR_A: u8 = 0;
    /// Armour, look "B" (the orange diamond): same stats, other appearance.
    pub const ARMOR_B: u8 = 1;
    /// Any weapon; the first number is the weapon type.
    pub const WEAPON: u8 = 13;
    /// Talisman.
    pub const TALISMAN: u8 = 21;
    /// Mantle.
    pub const MANTLE: u8 = 37;
}

/// The weapon type number of the light bowgun (the last of the 14 types in the save's order).
pub const LIGHT_BOWGUN_TYPE: i32 = 13;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipEntry {
    /// First field, see [`category`].
    pub category: u8,
    /// Second field. Weapon: the weapon type. Armour: the series. Talisman and mantle: the id.
    pub first: i32,
    /// Third field. Weapon: the weapon within its type (0 for the base weapon; the maintainer's
    /// light bowguns are 2, 16 and 17). Armour: the piece number 0 to 4. Otherwise 0.
    pub second: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipKind {
    Weapon {
        weapon_type: i32,
        id: i32,
    },
    /// `look_b` is false for the "A" look and true for "B".
    Armor {
        look_b: bool,
        series: i32,
        piece: i32,
    },
    Talisman {
        id: i32,
    },
    Mantle {
        id: i32,
    },
    Unknown {
        category: u8,
    },
}

impl EquipEntry {
    pub fn kind(&self) -> EquipKind {
        match self.category {
            category::WEAPON => EquipKind::Weapon {
                weapon_type: self.first,
                id: self.second,
            },
            category::ARMOR_A | category::ARMOR_B => EquipKind::Armor {
                look_b: self.category == category::ARMOR_B,
                series: self.first,
                piece: self.second,
            },
            category::TALISMAN => EquipKind::Talisman { id: self.first },
            category::MANTLE => EquipKind::Mantle { id: self.first },
            other => EquipKind::Unknown { category: other },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEntry {
    pub id: i32,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BossEntry {
    pub id: i32,
    pub kills: i32,
    pub captures: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlotSummary {
    /// The slot's own `Active` field.
    pub active_flag: bool,
    /// Entries in the equipment box, empty ones included (a diagnostic: 0 means the box was not
    /// found at all).
    pub equipment_box_len: usize,
    /// Equipment box entries in use (the empty category is dropped).
    pub equipment: Vec<EquipEntry>,
    /// Item box entries with a count above zero.
    pub box_items: Vec<ItemEntry>,
    /// Pouch entries with a count above zero.
    pub pouch: Vec<ItemEntry>,
    pub found_flag_bits_set: u32,
    pub bosses: Vec<BossEntry>,
}

impl SlotSummary {
    /// A slot has data when anything was found in it. An unused slot has an empty box.
    pub fn has_data(&self) -> bool {
        !self.equipment.is_empty()
            || !self.box_items.is_empty()
            || !self.pouch.is_empty()
            || self.found_flag_bits_set > 0
            || self.bosses.iter().any(|b| b.kills > 0 || b.captures > 0)
    }

    /// Weapons owned, by weapon type number.
    pub fn weapons_by_type(&self) -> std::collections::BTreeMap<i32, usize> {
        let mut m = std::collections::BTreeMap::new();
        for e in &self.equipment {
            if let EquipKind::Weapon { weapon_type, .. } = e.kind() {
                *m.entry(weapon_type).or_default() += 1;
            }
        }
        m
    }

    /// Armour entries in the box by piece number, the way the game's box counts them (each look
    /// of a piece is its own entry).
    pub fn armor_entries_by_piece(&self) -> std::collections::BTreeMap<i32, usize> {
        let mut m = std::collections::BTreeMap::new();
        for e in &self.equipment {
            if let EquipKind::Armor { piece, .. } = e.kind() {
                *m.entry(piece).or_default() += 1;
            }
        }
        m
    }

    /// Armour pieces counted once for stats: the A and B looks of a piece are the same piece.
    pub fn armor_distinct_series_pieces(&self) -> usize {
        self.equipment
            .iter()
            .filter_map(|e| match e.kind() {
                EquipKind::Armor { series, piece, .. } => Some((series, piece)),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    pub fn count_of(&self, f: impl Fn(&EquipKind) -> bool) -> usize {
        self.equipment.iter().filter(|e| f(&e.kind())).count()
    }

    pub fn bosses_with_kills(&self) -> usize {
        self.bosses.iter().filter(|b| b.kills > 0).count()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SaveSummary {
    pub slots: Vec<SlotSummary>,
    /// Index of the active slot, when exactly one slot has its `Active` field set. Otherwise
    /// `None`, and the window shows every slot and says the active one is unknown.
    pub active_slot: Option<usize>,
}

/// Opens, checks and summarises a save file.
pub fn read_save(file: &[u8], steam_id64: u64) -> Result<SaveSummary, SaveError> {
    let stream = container::open(file, steam_id64)?;
    summarize(&stream)
}

/// Checks a class stream end to end, then summarises it.
pub fn summarize(stream: &[u8]) -> Result<SaveSummary, SaveError> {
    let stream = Stream::new(stream);
    stream.check()?;
    let root_hash = name_hash(n::ROOT);
    let mut root = None;
    for entry in stream.top_level() {
        let (_, class) = entry?;
        if class.class_hash()? == root_hash {
            root = Some(class);
            break;
        }
    }
    let root = root.ok_or(SaveError::Missing("save root class"))?;
    let Some(Value::Array(slots)) = root.field(name_hash(n::SLOTS))? else {
        return Err(SaveError::Missing("slot list"));
    };

    let mut out = SaveSummary::default();
    for slot in slots.classes() {
        out.slots.push(read_slot(&slot?)?);
    }
    let flagged: Vec<usize> = out
        .slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.active_flag)
        .map(|(i, _)| i)
        .collect();
    if let [only] = flagged[..] {
        out.active_slot = Some(only);
    }
    Ok(out)
}

fn int(v: &Value) -> Option<i64> {
    v.scalar().map(|s| s.as_i64())
}

fn class_of<'a>(v: Value<'a>) -> Option<ClassView<'a>> {
    match v {
        Value::Class(c) => Some(c),
        _ => None,
    }
}

fn read_slot(slot: &ClassView) -> Result<SlotSummary, SaveError> {
    let h = |s| name_hash(s);
    let (active, equip, item, enemy) = (h(n::ACTIVE), h(n::EQUIP), h(n::ITEM), h(n::ENEMY_REPORT));
    let mut out = SlotSummary::default();
    for f in slot.fields() {
        let f = f?;
        if f.name_hash == active {
            out.active_flag = int(&f.value).is_some_and(|v| v != 0);
        } else if f.name_hash == equip {
            if let Some(c) = class_of(f.value) {
                read_equipment(&c, &mut out)?;
            }
        } else if f.name_hash == item {
            if let Some(c) = class_of(f.value) {
                read_items(&c, &mut out)?;
            }
        } else if f.name_hash == enemy {
            if let Some(c) = class_of(f.value) {
                read_enemies(&c, &mut out)?;
            }
        }
    }
    Ok(out)
}

fn read_equipment(equip: &ClassView, out: &mut SlotSummary) -> Result<(), SaveError> {
    let Some(Value::Array(boxed)) = equip.field(name_hash(n::EQUIP_BOX))? else {
        return Ok(());
    };
    // The field names of an equipment entry are not known yet (all 12 are unresolved hashes in
    // `docs/research.md`), so the first three fields are read by position: a one-byte category, then
    // two 32-bit numbers. This is the layout seen in the real save.
    out.equipment_box_len = boxed.count as usize;
    for entry in boxed.classes() {
        let (mut category, mut eid, mut sub) = (None, None, 0);
        for (i, f) in entry?.fields().take(3).enumerate() {
            let f = f?;
            match (i, f.value.scalar()) {
                (0, Some(sc)) if sc.size == 1 => category = Some(sc.as_i64()),
                (1, Some(sc)) if sc.size == 4 => eid = Some(sc.as_i64()),
                (2, Some(sc)) if sc.size == 4 => sub = sc.as_i64() as i32,
                _ => {}
            }
        }
        if let (Some(c), Some(i)) = (category, eid) {
            if c != i64::from(EMPTY_CATEGORY) {
                out.equipment.push(EquipEntry {
                    category: c as u8,
                    first: i as i32,
                    second: sub,
                });
            }
        }
    }
    Ok(())
}

fn read_item_array(v: Value, into: &mut Vec<ItemEntry>) -> Result<(), SaveError> {
    let Value::Array(arr) = v else { return Ok(()) };
    let (idh, numh) = (name_hash(n::ITEM_ID), name_hash(n::ITEM_NUM));
    for entry in arr.classes() {
        let (mut id, mut num) = (None, None);
        for f in entry?.fields() {
            let f = f?;
            if f.name_hash == idh {
                id = int(&f.value);
            } else if f.name_hash == numh {
                num = int(&f.value);
            }
        }
        if let (Some(id), Some(num)) = (id, num) {
            if num > 0 {
                into.push(ItemEntry {
                    id: id as i32,
                    count: num as i32,
                });
            }
        }
    }
    Ok(())
}

fn read_items(item: &ClassView, out: &mut SlotSummary) -> Result<(), SaveError> {
    let (boxh, pouch, found) = (
        name_hash(n::BOX_ITEM),
        name_hash(n::POUCH_ITEM),
        name_hash(n::FOUND_FLAG),
    );
    for f in item.fields() {
        let f = f?;
        if f.name_hash == boxh {
            read_item_array(f.value, &mut out.box_items)?;
        } else if f.name_hash == pouch {
            read_item_array(f.value, &mut out.pouch)?;
        } else if f.name_hash == found {
            if let Some(c) = class_of(f.value) {
                if let Some(Value::Array(words)) = c.field(name_hash(n::FLAG_VALUE))? {
                    if words.member_size == 4 {
                        out.found_flag_bits_set += words
                            .scalars()?
                            .map(|w| (w as u32).count_ones())
                            .sum::<u32>();
                    }
                }
            }
        }
    }
    Ok(())
}

fn read_enemies(enemy: &ClassView, out: &mut SlotSummary) -> Result<(), SaveError> {
    let Some(Value::Array(bosses)) = enemy.field(name_hash(n::BOSS))? else {
        return Ok(());
    };
    let (idh, kh, ch) = (
        name_hash(n::BOSS_ID),
        name_hash(n::BOSS_KILLS),
        name_hash(n::BOSS_CAPTURES),
    );
    for entry in bosses.classes() {
        let mut b = BossEntry {
            id: 0,
            kills: 0,
            captures: 0,
        };
        for f in entry?.fields() {
            let f = f?;
            let v = int(&f.value).unwrap_or(0) as i32;
            if f.name_hash == idh {
                b.id = v;
            } else if f.name_hash == kh {
                b.kills = v;
            } else if f.name_hash == ch {
                b.captures = v;
            }
        }
        out.bosses.push(b);
    }
    Ok(())
}

/// A structural dump of every in-use equipment box entry, for working out what the fields mean.
/// Numbers only: scalars up to 4 bytes with their value, 8-byte values as `wide`, strings as
/// `text`, small arrays with their values. No hunter data lives in an equipment entry.
pub fn equipment_dump(stream: &[u8]) -> Result<Vec<String>, SaveError> {
    let stream = Stream::new(stream);
    let root_hash = name_hash(n::ROOT);
    let mut lines = Vec::new();
    for entry in stream.top_level() {
        let (_, class) = entry?;
        if class.class_hash()? != root_hash {
            continue;
        }
        let Some(Value::Array(slots)) = class.field(name_hash(n::SLOTS))? else {
            continue;
        };
        for (si, slot) in slots.classes().enumerate() {
            let Some(Value::Class(equip)) = slot?.field(name_hash(n::EQUIP))? else {
                continue;
            };
            let Some(Value::Array(boxed)) = equip.field(name_hash(n::EQUIP_BOX))? else {
                continue;
            };
            let mut histogram = std::collections::BTreeMap::<i64, usize>::new();
            let mut shown = Vec::new();
            for (ei, e) in boxed.classes().enumerate() {
                let e = e?;
                let mut cols = Vec::new();
                let mut first = None;
                for f in e.fields() {
                    let f = f?;
                    cols.push(match f.value {
                        Value::Scalar(sc) if sc.size <= 4 => {
                            if first.is_none() {
                                first = Some(sc.as_i64());
                            }
                            match sc.as_f32() {
                                Some(x) => format!("t{}={x}", sc.code),
                                None => format!("t{}s{}={}", sc.code, sc.size, sc.as_i64()),
                            }
                        }
                        // Wide values (such as 64-bit ids) are not printed, only whether they are zero.
                        Value::Scalar(sc) => format!(
                            "t{}s{}={}",
                            sc.code,
                            sc.size,
                            if sc.as_u64() == 0 { "wide0" } else { "wide!=0" }
                        ),
                        Value::Str(_) => "text".to_string(),
                        Value::Struct(b) => format!("struct{}", b.len()),
                        Value::Class(_) => "class".to_string(),
                        Value::Array(a) => match a.scalars() {
                            Ok(v) if a.count <= 8 && a.member_size <= 4 => {
                                let size = a.member_size;
                                let signed = matches!(a.member_type, 3 | 5 | 7 | 9);
                                let vals: Vec<String> = v
                                    .map(|x| match (signed, size) {
                                        (true, 1) => (x as u8 as i8).to_string(),
                                        (true, 2) => (x as u16 as i16).to_string(),
                                        (true, 4) => (x as u32 as i32).to_string(),
                                        _ => x.to_string(),
                                    })
                                    .collect();
                                format!("arr[{}]", vals.join(","))
                            }
                            _ => format!("arr<{}x{}>", a.count, a.member_size),
                        },
                    });
                }
                let key = first.unwrap_or(-1);
                *histogram.entry(key).or_default() += 1;
                if key != i64::from(EMPTY_CATEGORY) {
                    shown.push(format!("slot {} entry {ei}: {}", si + 1, cols.join(" ")));
                }
            }
            lines.push(format!(
                "slot {}: first field over all {} entries: {}",
                si + 1,
                boxed.count,
                histogram
                    .iter()
                    .map(|(k, n)| format!("{k} x{n}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            lines.extend(shown);
        }
    }
    Ok(lines)
}

/// A structural listing of the fields of each slot's equipment class (`_Equip`): for every field
/// its name (or hash), kind and size, and for class values and class arrays a compact numeric
/// summary. Used to find where equipped gear, armour and talismans live. Numbers only.
pub fn equipment_layout(
    stream: &[u8],
    names: &crate::names::TypeNames,
) -> Result<Vec<String>, SaveError> {
    /// Numbers only, nested classes shown to `depth` levels.
    fn compact(
        c: &ClassView,
        names: &crate::names::TypeNames,
        depth: u32,
    ) -> Result<String, SaveError> {
        let mut parts = Vec::new();
        for f in c.fields() {
            let f = f?;
            let v = match f.value {
                Value::Scalar(sc) if sc.size <= 4 => format!("{}", sc.as_i64()),
                Value::Scalar(sc) => {
                    if sc.as_u64() == 0 {
                        "wide0".to_string()
                    } else {
                        "wide!=0".to_string()
                    }
                }
                Value::Str(_) => "text".to_string(),
                Value::Struct(b) => format!("struct{}", b.len()),
                Value::Class(inner) if depth > 0 => {
                    format!("{{{}}}", compact(&inner, names, depth - 1)?)
                }
                Value::Class(_) => "class".to_string(),
                Value::Array(a) => match a.scalars() {
                    Ok(v) if a.count <= 8 && a.member_size <= 4 => {
                        let vals: Vec<String> = v.map(|x| (x as u32 as i32).to_string()).collect();
                        format!("[{}]", vals.join(","))
                    }
                    _ => format!("arr{}", a.count),
                },
            };
            parts.push(format!("{}={v}", names.show(f.name_hash)));
        }
        Ok(parts.join(" "))
    }

    let stream = Stream::new(stream);
    let root_hash = name_hash(n::ROOT);
    let mut lines = Vec::new();
    for entry in stream.top_level() {
        let (_, class) = entry?;
        if class.class_hash()? != root_hash {
            continue;
        }
        let Some(Value::Array(slots)) = class.field(name_hash(n::SLOTS))? else {
            continue;
        };
        for (si, slot) in slots.classes().enumerate() {
            let Some(Value::Class(equip)) = slot?.field(name_hash(n::EQUIP))? else {
                continue;
            };
            if si > 0 {
                // Slots without data repeat the same shape; the first slot is enough.
                continue;
            }
            for f in equip.fields() {
                let f = f?;
                let name = names.show(f.name_hash);
                match f.value {
                    Value::Scalar(sc) if sc.size <= 4 => {
                        lines.push(format!("{name}: scalar t{} = {}", sc.code, sc.as_i64()))
                    }
                    Value::Scalar(sc) => {
                        lines.push(format!("{name}: scalar t{} s{}", sc.code, sc.size))
                    }
                    Value::Str(_) => lines.push(format!("{name}: text")),
                    Value::Struct(b) => lines.push(format!("{name}: struct of {} bytes", b.len())),
                    Value::Class(c) => {
                        lines.push(format!("{name}: class {}", names.show(c.class_hash()?)));
                        lines.push(format!("    {}", compact(&c, names, 3)?));
                    }
                    Value::Array(a) if a.kind == 1 => {
                        let mut first = std::collections::BTreeMap::<i64, usize>::new();
                        let mut sample = String::new();
                        let mut class_name = String::new();
                        for (i, c) in a.classes().enumerate() {
                            let c = c?;
                            if i == 0 {
                                class_name = names.show(c.class_hash()?);
                            }
                            let key = match c.fields().next() {
                                Some(Ok(f)) => f.value.scalar().map_or(-2, |s| s.as_i64()),
                                _ => -3,
                            };
                            *first.entry(key).or_default() += 1;
                            if sample.is_empty() && key != i64::from(EMPTY_CATEGORY) && key != 0 {
                                sample = compact(&c, names, 3)?;
                            }
                        }
                        lines.push(format!(
                            "{name}: array of {} x {class_name}; first field: {}",
                            a.count,
                            first
                                .iter()
                                .map(|(k, n)| format!("{k} x{n}"))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                        if !sample.is_empty() {
                            lines.push(format!("    sample: {sample}"));
                        }
                    }
                    Value::Array(a) => lines.push(format!(
                        "{name}: array of {} x t{} size {}",
                        a.count, a.member_type, a.member_size
                    )),
                }
            }
        }
    }
    Ok(lines)
}
