// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Developer report for PC test rounds. Prints what the foundation sees as counts and names only:
//! never a hunter name or ID, never file contents.
//!
//! `pak` indexes the install's original packs, resolves and reads the needed game files and (with
//! `--names`) rebuilds the save's class names from the exe; it only reads the install and prints
//! counts, pack names and the needed paths.
//!
//! `engine` starts the engine against a data folder you name and reports how fast the view appears
//! (from the cache, then checked) and, with `--watch`, how fast a new save shows up.
//!
//! `midwrite` rewrites a copy of a save slowly (and, on Windows, locks it) while the engine watches
//! it, to show that a half-written or locked save never gives a wrong view.
//!
//! `update` runs the update flow into a separate data folder (never the installed copy's): it reads
//! the install and the save, writes only that folder, and reports Replaced, Unchanged, Rejected or
//! Failed. `--simulate-missing` pretends one needed file was not found, to see the gate reject it.
//!
//! `discover` reads nothing from the save or the install beyond folder listings and two small
//! Steam text files, and writes nothing at all. `save` copies the save file to its own work folder
//! (the only thing it writes) and reads the copy; it never reads or writes the live save in place
//! and refuses a work folder inside Steam, the install or a save folder.

mod engine;
mod midwrite;
mod pak;
mod update;

use forgeline_core::datadir::is_protected;
use forgeline_core::discovery::{self, DiscoveryEnv, SaveAccount};
use forgeline_core::memory;
use forgeline_save::model::{
    equipment_dump, equipment_layout, summarize, EquipKind, SaveSummary, LIGHT_BOWGUN_TYPE,
};
use forgeline_save::names::TypeNames;
use forgeline_save::{container, SaveError};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: forgeline-check discover [--steam-dir <folder>]\n       forgeline-check save [--steam-dir <folder>] [--work-dir <folder>]\n       forgeline-check save --file <copy of the save> --steam-id <id>\n       forgeline-check pak [--steam-dir <folder>] [--names [--file <copy of the save> --steam-id <id>] [--exe <file>] [--names-out <file>]]\n       forgeline-check update --data-dir <folder> [--steam-dir <folder>] [--simulate-missing <needed file path>]\n       forgeline-check engine --data-dir <folder> [--steam-dir <folder>] [--watch <seconds>] [--file <copy of the save> --steam-id <id>]\n       forgeline-check midwrite --file <copy of the save> --steam-id <id> --work-dir <folder>\n       add --equip-dump to `save` to list the equipment box entries, or --equip-layout for the fields of the equipment class (numbers only)";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("discover") => discover(args.collect()),
        Some("save") => save(args.collect()),
        Some("pak") => pak::run(args.collect()),
        Some("update") => update::run(args.collect()),
        Some("engine") => engine::run(args.collect()),
        Some("midwrite") => midwrite::run(args.collect()),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn discover(args: Vec<String>) -> ExitCode {
    let mut steam_dir: Option<PathBuf> = None;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--steam-dir" => match it.next() {
                Some(d) => steam_dir = Some(PathBuf::from(d)),
                None => {
                    eprintln!("--steam-dir needs a folder\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(steam_root) = steam_dir.or_else(discovery::find_steam_root) else {
        eprintln!("Steam was not found; pass --steam-dir <folder>");
        return ExitCode::FAILURE;
    };

    println!("steam folder: {}", shown(&steam_root));
    let found = match discovery::discover(&DiscoveryEnv { steam_root }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("discovery failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("library:      {}", shown(&found.library));
    println!("install:      {}", shown(&found.install));
    println!("build id:     {}", found.build_id);

    match found.save_account() {
        SaveAccount::None => {
            println!("saves:        no Steam account folder with Wilds saves found")
        }
        SaveAccount::One(_) => println!("saves:        1 Steam account folder with Wilds saves"),
        SaveAccount::Several(many) => println!(
            "saves:        {} Steam account folders with Wilds saves (the window will ask which)",
            many.len()
        ),
    }
    for (n, a) in found.accounts.iter().enumerate() {
        // Only the tail of the Steam ID is shown, so a pasted report does not identify the account.
        let id = a.steam_id64.to_string();
        println!(
            "  account {}: Steam ID ending {}, {} slot file(s): {}",
            n + 1,
            &id[id.len().saturating_sub(4)..],
            a.slot_files.len(),
            a.slot_files
                .iter()
                .map(|f| format!("{} ({} bytes)", f.name, f.size))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if found.mod_markers.is_empty() {
        println!("mod markers:  none found in the install (listed by name only, never opened)");
    } else {
        println!(
            "mod markers:  {} (found by name in the install; none of them was opened or read)",
            found.mod_markers.join(", ")
        );
    }
    ExitCode::SUCCESS
}

/// A path for display with the user's home folder replaced, so a pasted report holds no user name.
pub(crate) fn shown(p: &Path) -> String {
    let text = p.display().to_string();
    for var in ["USERPROFILE", "HOME"] {
        if let Some(home) = std::env::var_os(var).filter(|h| !h.is_empty()) {
            let home = home.to_string_lossy().into_owned();
            if let Some(rest) = text.strip_prefix(&home) {
                return format!("~{rest}");
            }
        }
    }
    text
}

/// Reads the save through a copy and reports counts. Never prints a hunter name or ID.
fn save(args: Vec<String>) -> ExitCode {
    let (mut steam_dir, mut work_dir, mut file, mut steam_id) = (None, None, None, None);
    let equip_dump = args.iter().any(|a| a == "--equip-dump");
    let equip_layout = args.iter().any(|a| a == "--equip-layout");
    let mut it = args
        .into_iter()
        .filter(|a| a != "--equip-dump" && a != "--equip-layout");
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--steam-dir" => &mut steam_dir,
            "--work-dir" => &mut work_dir,
            "--file" => &mut file,
            "--steam-id" => &mut steam_id,
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        };
        match it.next() {
            Some(v) => *slot = Some(v),
            None => {
                eprintln!("{a} needs a value\n{USAGE}");
                return ExitCode::from(2);
            }
        }
    }

    let (copy, id): (PathBuf, u64) = if let Some(f) = file {
        let Some(id) = steam_id.as_deref().and_then(parse_steam_id) else {
            eprintln!("--file needs --steam-id <number>\n{USAGE}");
            return ExitCode::from(2);
        };
        let path = PathBuf::from(f);
        if is_protected(&path, &[]) {
            eprintln!(
                "refusing that file: it is inside a Steam, install or save folder. Pass a copy."
            );
            return ExitCode::FAILURE;
        }
        (path, id)
    } else {
        match copy_live_save(steam_dir.map(PathBuf::from), work_dir.map(PathBuf::from)) {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("{msg}");
                return ExitCode::FAILURE;
            }
        }
    };

    let bytes = match std::fs::read(&copy) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read the copy: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("save copy:    {} bytes", bytes.len());
    let started = Instant::now();
    let stream = match container::open(&bytes, id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("reading failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    drop(bytes);
    let summary = match summarize(&stream) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("reading failed: {e}");
            if let SaveError::Stream { offset, .. } = e {
                eprintln!(
                    "the container opened; the class stream is {} bytes",
                    stream.len()
                );
                eprintln!("words around the failure (u32, hex; text-like words and long runs are masked):");
                eprintln!("{}", words_around(&stream, offset));
            }
            return ExitCode::FAILURE;
        }
    };
    let took = started.elapsed();
    if equip_layout {
        match equipment_layout(&stream, &TypeNames::builtin()) {
            Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
            Err(e) => eprintln!("equipment layout failed: {e}"),
        }
    }
    if equip_dump {
        match equipment_dump(&stream) {
            Ok(lines) => lines.iter().for_each(|l| println!("{l}")),
            Err(e) => eprintln!("equipment dump failed: {e}"),
        }
    }
    print_summary(&summary);
    println!(
        "read time:    {} ms (decrypt, inflate, full walk and summary)",
        took.as_millis()
    );
    let mem = memory::read();
    let mb = |b: Option<u64>| {
        b.map_or("not reported".to_string(), |v| {
            format!("{:.1} MB", v as f64 / 1_048_576.0)
        })
    };
    println!(
        "memory:       peak private {}, now {} (this report process, whole run)",
        mb(mem.peak_private_bytes),
        mb(mem.private_bytes)
    );
    ExitCode::SUCCESS
}

pub(crate) fn parse_steam_id(raw: &str) -> Option<u64> {
    const BASE: u64 = 76_561_197_960_265_728;
    let v: u64 = raw.parse().ok()?;
    Some(if v < (1 << 32) { BASE + v } else { v })
}

/// Finds the save, copies it into the work folder and returns the copy and the Steam ID.
pub(crate) fn copy_live_save(
    steam_dir: Option<PathBuf>,
    work_dir: Option<PathBuf>,
) -> Result<(PathBuf, u64), String> {
    let steam_root = steam_dir
        .or_else(discovery::find_steam_root)
        .ok_or("Steam was not found; pass --steam-dir <folder>")?;
    let found = discovery::discover(&DiscoveryEnv { steam_root })
        .map_err(|e| format!("discovery failed: {e}"))?;
    let account = match found.save_account() {
        SaveAccount::One(a) => a,
        SaveAccount::None => return Err("no Steam account folder with Wilds saves found".into()),
        SaveAccount::Several(_) => {
            return Err(
                "several Steam accounts have Wilds saves; use --file with a copy and --steam-id"
                    .into(),
            )
        }
    };
    let slot_file = account
        .slot_files
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case("data001Slot.bin"))
        .ok_or("data001Slot.bin was not found in the save folder")?;

    let work = work_dir.unwrap_or_else(|| std::env::temp_dir().join("forgeline-check-work"));
    if is_protected(&work, &found.protected_paths()) {
        return Err(
            "refusing that work folder: it is inside Steam, the install or a save folder".into(),
        );
    }
    std::fs::create_dir_all(&work).map_err(|e| format!("cannot create the work folder: {e}"))?;
    let copy = work.join("save-copy.bin");
    std::fs::copy(account.save_dir.join(&slot_file.name), &copy)
        .map_err(|e| format!("cannot copy the save: {e}"))?;
    let id = account.steam_id64.to_string();
    println!(
        "steam account: ID ending {}",
        &id[id.len().saturating_sub(4)..]
    );
    println!("work folder:  {}", shown(&work));
    Ok((copy, account.steam_id64))
}

fn print_summary(s: &SaveSummary) {
    println!("slots:        {}", s.slots.len());
    match s.active_slot {
        Some(i) => println!("active slot:  {} (from the slot's Active field)", i + 1),
        None => println!("active slot:  unknown (no single slot has its Active field set)"),
    }
    for (i, slot) in s.slots.iter().enumerate() {
        let mut by_cat: BTreeMap<u8, usize> = BTreeMap::new();
        for e in &slot.equipment {
            *by_cat.entry(e.category).or_default() += 1;
        }
        println!(
            "slot {}: {}{}",
            i + 1,
            if slot.has_data() { "has data" } else { "empty" },
            if s.active_slot == Some(i) {
                ", ACTIVE"
            } else {
                ""
            }
        );
        if !slot.has_data() {
            continue;
        }
        println!(
            "  equipment box: {} entries, {} in use",
            slot.equipment_box_len,
            slot.equipment.len()
        );
        println!(
            "  by category: {}",
            by_cat
                .iter()
                .map(|(c, n)| format!("{c} x{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let weapons = slot.weapons_by_type();
        println!(
            "  weapons: {} ({}); only type 13 (light bowgun) is named, the rest are by number",
            weapons.values().sum::<usize>(),
            weapons
                .iter()
                .map(|(t, n)| format!("type {t} x{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  light bowguns: {}",
            weapons.get(&LIGHT_BOWGUN_TYPE).copied().unwrap_or(0)
        );
        let by_piece = slot.armor_entries_by_piece();
        println!(
            "  armour box entries: {} (by piece number: {}); A look {}, B look {}; counted once for stats: {}",
            by_piece.values().sum::<usize>(),
            by_piece
                .iter()
                .map(|(p, n)| format!("{p} x{n}"))
                .collect::<Vec<_>>()
                .join(", "),
            slot.count_of(|k| matches!(k, EquipKind::Armor { look_b: false, .. })),
            slot.count_of(|k| matches!(k, EquipKind::Armor { look_b: true, .. })),
            slot.armor_distinct_series_pieces()
        );
        println!(
            "  talismans: {}; mantles: {}; other equipment: {}",
            slot.count_of(|k| matches!(k, EquipKind::Talisman { .. })),
            slot.count_of(|k| matches!(k, EquipKind::Mantle { .. })),
            slot.count_of(|k| matches!(k, EquipKind::Unknown { .. }))
        );
        println!(
            "  item box types with a count: {}; pouch entries: {}",
            slot.box_items.len(),
            slot.pouch.len()
        );
        println!("  item found flags set: {}", slot.found_flag_bits_set);
        println!(
            "  monster entries: {}, with kills: {}",
            slot.bosses.len(),
            slot.bosses_with_kills()
        );
    }
}

/// Structure words near a failure, for pinning down a layout difference without printing text:
/// a word whose four bytes are all printable ASCII (or half of a UTF-16 pair) shows as `text`.
fn words_around(stream: &[u8], offset: usize) -> String {
    let start = offset.saturating_sub(64) & !3;
    let end = (offset + 64).min(stream.len()) & !3;
    let mut out = Vec::new();
    for at in (start..end).step_by(4) {
        let w = &stream[at..at + 4];
        let printable = |b: &u8| (0x20..0x7f).contains(b);
        let text = w.iter().all(printable)
            || (w[1] == 0 && w[3] == 0 && printable(&w[0]) && printable(&w[2]));
        let v = u32::from_le_bytes([w[0], w[1], w[2], w[3]]);
        out.push(if text {
            format!("{at}:text")
        } else {
            format!("{at}:{v:x}")
        });
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_around_masks_text_and_shows_structure() {
        let mut s = Vec::new();
        s.extend_from_slice(&0x1234_u32.to_le_bytes());
        s.extend_from_slice(b"ABCD");
        s.extend_from_slice(&[b'h', 0, b'i', 0]);
        s.extend_from_slice(&0xFFFF_FFFF_u32.to_le_bytes());
        let out = words_around(&s, 8);
        assert_eq!(out, "0:1234 4:text 8:text 12:ffffffff");
    }
}
