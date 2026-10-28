// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A typed summary of the save: the data the window needs and nothing else.
//!
//! Classes and fields are found by hashing the names in [`n`], so no type table is needed to read.
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
    pub const CATEGORY: &str = "Category_Gender";
    pub const EQUIP_ID: &str = "EquipIdFixed";
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
        CATEGORY,
        EQUIP_ID,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipEntry {
    pub category: u8,
    pub id: u32,
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
    let (cat, id) = (name_hash(n::CATEGORY), name_hash(n::EQUIP_ID));
    for entry in boxed.classes() {
        let (mut category, mut eid) = (None, None);
        for f in entry?.fields() {
            let f = f?;
            if f.name_hash == cat {
                category = int(&f.value);
            } else if f.name_hash == id {
                eid = int(&f.value);
            }
        }
        if let (Some(c), Some(i)) = (category, eid) {
            if c != i64::from(EMPTY_CATEGORY) {
                out.equipment.push(EquipEntry {
                    category: c as u8,
                    id: i as u32,
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
