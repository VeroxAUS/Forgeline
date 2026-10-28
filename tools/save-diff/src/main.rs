// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Compares two save copies and lists which fields changed, by class and field name with the old
//! and new numbers. Meant for questions like "does the save record the accepted quest?": take a
//! copy before and a copy after, and read the list.
//!
//! Reads copies only: any path inside a Steam, `userdata`, `steamapps` or `win64_save` folder is
//! refused, and nothing is written. Text and raw bytes are reported as lengths only, and the
//! `_BasicData` field (hunter name and IDs) is left out, so a report holds no hunter data.

use forgeline_core::datadir::is_protected;
use forgeline_save::container;
use forgeline_save::flatten::{flatten, Leaf};
use forgeline_save::hash::name_hash;
use forgeline_save::model::n;
use forgeline_save::names::TypeNames;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str =
    "usage: save-diff <before.bin> <after.bin> --steam-id <id> [--names <file>] [--max <lines>]\n\
  <id> is the 64-bit Steam ID, or the short account number from the userdata folder name.\n\
  Both files must be copies, not files inside win64_save.";

const STEAM_ID_BASE: u64 = 0x0110_0001_0000_0000;

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}

fn steam_id64(raw: &str) -> Result<u64, String> {
    let v: u64 = raw
        .parse()
        .map_err(|_| format!("--steam-id must be a number\n{USAGE}"))?;
    Ok(if v < (1 << 32) { STEAM_ID_BASE + v } else { v })
}

type Leaves = BTreeMap<String, Leaf>;

fn leaves(path: &Path, id: u64, names: &TypeNames) -> Result<(Leaves, usize), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let stream = container::open(&bytes, id).map_err(|e| {
        format!(
            "{}: {e}",
            path.file_name().unwrap_or_default().to_string_lossy()
        )
    })?;
    drop(bytes);
    let mut out = Leaves::new();
    let skipped = flatten(
        &stream,
        names,
        &[name_hash(n::BASIC_DATA)],
        &mut |p, leaf| {
            out.insert(p.to_string(), leaf);
        },
    )
    .map_err(|e| e.to_string())?;
    Ok((out, skipped))
}

/// What differs between two leaf maps, as report lines.
pub fn diff(before: &Leaves, after: &Leaves) -> Vec<String> {
    let mut lines = Vec::new();
    for (path, old) in before {
        match after.get(path) {
            None => lines.push(format!("removed  {path} (was {old})")),
            Some(new) if new != old => {
                lines.push(format!("changed  {path}: {old} -> {new}"));
            }
            _ => {}
        }
    }
    for (path, new) in after {
        if !before.contains_key(path) {
            lines.push(format!("added    {path} = {new}"));
        }
    }
    lines
}

fn run(args: Vec<String>) -> Result<(), String> {
    let mut files = Vec::new();
    let (mut id, mut names_file, mut max) = (None, None, 200usize);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--steam-id" => id = it.next(),
            "--names" => names_file = it.next(),
            "--max" => {
                max = it
                    .next()
                    .and_then(|m| m.parse().ok())
                    .ok_or_else(|| format!("--max needs a number\n{USAGE}"))?
            }
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            _ if a.starts_with("--") => return Err(format!("unknown option {a}\n{USAGE}")),
            _ => files.push(a),
        }
    }
    let [before, after] = &files[..] else {
        return Err(USAGE.to_string());
    };
    let id = steam_id64(&id.ok_or_else(|| USAGE.to_string())?)?;
    for f in [before, after] {
        if is_protected(Path::new(f), &[]) {
            return Err(format!(
                "refusing {f}: it is inside a Steam save folder. Copy the file somewhere else first."
            ));
        }
    }

    let mut names = TypeNames::builtin();
    if let Some(f) = names_file {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("cannot read {f}: {e}"))?;
        names.add_lines(&text);
    }
    let (a, skipped_a) = leaves(Path::new(before), id, &names)?;
    let (b, skipped_b) = leaves(Path::new(after), id, &names)?;

    let lines = diff(&a, &b);
    println!(
        "values compared: {} before, {} after ({} identity field(s) left out)",
        a.len(),
        b.len(),
        skipped_a.max(skipped_b)
    );
    println!("differences: {}", lines.len());
    for l in lines.iter().take(max) {
        println!("{l}");
    }
    if lines.len() > max {
        println!("... {} more (raise --max to see them)", lines.len() - max);
    }
    if lines.is_empty() {
        println!("The two saves hold the same values.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_ids_accept_the_short_and_long_form() {
        assert_eq!(steam_id64("4242").unwrap(), STEAM_ID_BASE + 4242);
        assert_eq!(
            steam_id64("76561197960269970").unwrap(),
            STEAM_ID_BASE + 4242
        );
        assert!(steam_id64("abc").is_err());
    }

    #[test]
    fn live_save_folders_are_refused_wherever_they_sit() {
        for p in [
            "C:/Steam/userdata/1/2246340/remote/win64_save/data001Slot.bin",
            "D:/Games/steamapps/x.bin",
            "win64_save/a.bin",
            "WIN64_SAVE/a.bin",
        ] {
            assert!(is_protected(Path::new(p), &[]), "{p}");
        }
        assert!(!is_protected(Path::new("E:/mhbis/copies/before.bin"), &[]));
    }

    #[test]
    fn diff_lists_changed_added_and_removed() {
        let mut a = Leaves::new();
        let mut b = Leaves::new();
        a.insert("x.A".into(), Leaf::Int(1));
        a.insert("x.B".into(), Leaf::Int(2));
        a.insert("x.C".into(), Leaf::Int(3));
        b.insert("x.A".into(), Leaf::Int(1));
        b.insert("x.B".into(), Leaf::Int(5));
        b.insert("x.D".into(), Leaf::Int(9));
        assert_eq!(
            diff(&a, &b),
            [
                "changed  x.B: 2 -> 5",
                "removed  x.C (was 3)",
                "added    x.D = 9"
            ]
        );
    }
}
