// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check pak`: indexes the original packs of the install, resolves the needed-files
//! list and, with `--names`, rebuilds the save's class and field names from the game exe. It
//! prints counts, pack names, the needed paths and times. It never prints file contents, and
//! the install is only read.

use crate::{copy_live_save, parse_steam_id, shown, USAGE};
use forgeline_core::datadir::is_protected;
use forgeline_core::discovery::{self, DiscoveryEnv};
use forgeline_core::memory;
use forgeline_pak::exescan;
use forgeline_pak::index::PackSet;
use forgeline_pak::paths::{resolve, NEEDED};
use forgeline_save::container;
use forgeline_save::hashes::collect;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

/// Name of the game exe in the install folder.
const EXE_NAME: &str = "MonsterHunterWilds.exe";
const NAMESPACES: &[&str] = &["app.savedata"];

pub fn run(args: Vec<String>) -> ExitCode {
    let (mut steam_dir, mut work_dir, mut file, mut steam_id, mut exe, mut names_out) =
        (None, None, None, None, None, None);
    let mut names = false;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        if a == "--names" {
            names = true;
            continue;
        }
        let slot = match a.as_str() {
            "--steam-dir" => &mut steam_dir,
            "--work-dir" => &mut work_dir,
            "--file" => &mut file,
            "--steam-id" => &mut steam_id,
            "--exe" => &mut exe,
            "--names-out" => &mut names_out,
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
    if (file.is_some() || names_out.is_some() || exe.is_some()) && !names {
        eprintln!("--file, --exe and --names-out only make sense with --names\n{USAGE}");
        return ExitCode::from(2);
    }

    let Some(steam_root) = steam_dir
        .clone()
        .map(PathBuf::from)
        .or_else(discovery::find_steam_root)
    else {
        eprintln!("Steam was not found; pass --steam-dir <folder>");
        return ExitCode::FAILURE;
    };
    let found = match discovery::discover(&DiscoveryEnv { steam_root }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("discovery failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("install:      {}", shown(&found.install));
    println!("build id:     {}", found.build_id);

    // Packs.
    let started = Instant::now();
    let set = match PackSet::open(&found.install) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("opening the packs failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let index_ms = started.elapsed().as_millis();
    println!(
        "packs:        {} opened (original re_*.pak files in the install folder only; pak_mods and mod files are never opened)",
        set.pack_count()
    );
    for (i, (path, entries)) in set.pack_files().enumerate() {
        println!(
            "  {:2}  {}  {} entries",
            i + 1,
            path.file_name().unwrap_or_default().to_string_lossy(),
            entries
        );
    }
    println!(
        "files:        {} distinct after later packs override earlier ones; index built in {} ms",
        set.len(),
        index_ms
    );

    // Needed files: resolve, then read each one to prove the reader on the real packs.
    let mut missing = Vec::new();
    let mut unreadable = Vec::new();
    let (mut read_ok, mut bytes) = (0usize, 0u64);
    let started = Instant::now();
    for n in NEEDED {
        match resolve(&set, n.stem) {
            None => missing.push(n.stem),
            Some((_, entry)) => match set.read(&entry) {
                Ok(data) => {
                    read_ok += 1;
                    bytes += data.len() as u64;
                }
                Err(e) => unreadable.push(format!("{}: {e}", n.stem)),
            },
        }
    }
    println!(
        "needed files: {} of {} resolved, {} read ({} bytes unpacked, {} ms)",
        NEEDED.len() - missing.len(),
        NEEDED.len(),
        read_ok,
        bytes,
        started.elapsed().as_millis()
    );
    for m in &missing {
        println!("  NOT FOUND: {m}");
    }
    for u in &unreadable {
        println!("  UNREADABLE: {u}");
    }
    let mut failed = !missing.is_empty() || !unreadable.is_empty();
    drop(set);

    if names {
        failed |= type_names(
            &found.install,
            &found.build_id,
            &found.protected_paths(),
            steam_dir,
            work_dir,
            file,
            steam_id,
            exe,
            names_out,
        );
    }

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
    if failed {
        println!("result:       FAILED (see the lines above)");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Rebuilds the names the save uses from the exe. Returns true on failure.
#[allow(clippy::too_many_arguments)]
fn type_names(
    install: &std::path::Path,
    build_id: &str,
    protected: &[PathBuf],
    steam_dir: Option<String>,
    work_dir: Option<String>,
    file: Option<String>,
    steam_id: Option<String>,
    exe: Option<String>,
    names_out: Option<String>,
) -> bool {
    let (copy, id): (PathBuf, u64) = if let Some(f) = file {
        let Some(id) = steam_id.as_deref().and_then(parse_steam_id) else {
            eprintln!("--file needs --steam-id <number>");
            return true;
        };
        let path = PathBuf::from(f);
        if is_protected(&path, &[]) {
            eprintln!(
                "refusing that file: it is inside a Steam, install or save folder. Pass a copy."
            );
            return true;
        }
        (path, id)
    } else {
        match copy_live_save(steam_dir.map(PathBuf::from), work_dir.map(PathBuf::from)) {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("{msg}");
                return true;
            }
        }
    };
    if let Some(out) = &names_out {
        if is_protected(&PathBuf::from(out), protected) {
            eprintln!("refusing --names-out: it is inside Steam, the install or a save folder");
            return true;
        }
    }

    let bytes = match std::fs::read(&copy) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read the save copy: {e}");
            return true;
        }
    };
    let stream = match container::open(&bytes, id) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("reading the save failed: {e}");
            return true;
        }
    };
    drop(bytes);
    let hashes = match collect(&stream) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("walking the save failed: {e}");
            return true;
        }
    };
    drop(stream);

    let exe = exe
        .map(PathBuf::from)
        .unwrap_or_else(|| install.join(EXE_NAME));
    let wanted: HashSet<u32> = hashes.all().into_iter().collect();
    let started = Instant::now();
    let scan = match exescan::scan(&exe, &wanted, NAMESPACES) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("scanning the exe failed: {e}");
            return true;
        }
    };
    let took = started.elapsed();
    let found_in = |set: &std::collections::BTreeSet<u32>| {
        set.iter().filter(|h| scan.names.contains_key(h)).count()
    };
    println!(
        "exe scan:     {} bytes, modified {} (seconds since 1970), build id {}, {} strings, {} class-like names, {} ms",
        scan.exe_size,
        scan.exe_modified.map_or("unknown".to_string(), |s| s.to_string()),
        build_id,
        scan.strings,
        scan.class_like,
        took.as_millis()
    );
    println!(
        "type names:   classes {} of {} named, fields {} of {} named (namespace tried: {})",
        found_in(&hashes.classes),
        hashes.classes.len(),
        found_in(&hashes.fields),
        hashes.fields.len(),
        NAMESPACES.join(", ")
    );
    if let Some(out) = names_out {
        let text: String = scan.names.values().map(|n| format!("{n}\n")).collect();
        match std::fs::write(&out, text) {
            Ok(()) => println!(
                "names file:   {} names written to {}",
                scan.names.len(),
                shown(&PathBuf::from(&out))
            ),
            Err(e) => {
                eprintln!("cannot write the names file: {e}");
                return true;
            }
        }
    }
    false
}
