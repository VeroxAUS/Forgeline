// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check engine`: starts the engine against a data folder you name and reports how
//! fast the view appears. Run it twice: the first run has no cache, the second shows the cache.
//! With `--watch <seconds>` it then waits for the game to save and reports how long each new
//! snapshot took after the save file's modified time. Counts only: no hunter name or ID.
//!
//! The engine opens the live save with sharing and reads it into memory; it never writes to it.
//! Use `--file <copy> --steam-id <id>` to read a copy instead.

use crate::{shown, USAGE};
use forgeline_core::datadir::DataDir;
use forgeline_core::memory;
use forgeline_engine::{Engine, EngineConfig, SaveOverride, Snapshot};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn run(args: Vec<String>) -> ExitCode {
    let (mut steam_dir, mut data_dir, mut file, mut steam_id) = (None, None, None, None);
    let mut watch_secs = 0u64;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let Some(v) = it.next() else {
            eprintln!("{a} needs a value\n{USAGE}");
            return ExitCode::from(2);
        };
        match a.as_str() {
            "--steam-dir" => steam_dir = Some(PathBuf::from(v)),
            "--data-dir" => data_dir = Some(PathBuf::from(v)),
            "--file" => file = Some(PathBuf::from(v)),
            "--steam-id" => steam_id = crate::parse_steam_id(&v),
            "--watch" => match v.parse() {
                Ok(n) => watch_secs = n,
                Err(_) => {
                    eprintln!("--watch needs a number of seconds\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(data_dir) = data_dir else {
        eprintln!(
            "engine needs --data-dir <folder> (a dev folder, never the installed copy's)\n{USAGE}"
        );
        return ExitCode::from(2);
    };
    let save_override = match (file, steam_id) {
        (Some(path), Some(steam_id64)) => Some(SaveOverride { path, steam_id64 }),
        (None, None) => None,
        _ => {
            eprintln!("--file and --steam-id go together\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let data = match DataDir::open(Some(data_dir)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    println!("data folder:  {}", shown(data.root()));

    let (tx, rx) = channel();
    let started = Instant::now();
    let engine = Engine::start(
        EngineConfig {
            data,
            steam_root: steam_dir,
            account_id: None,
            save_override,
        },
        move || {
            let _ = tx.send(Instant::now());
        },
    );
    let first = engine.snapshot();
    println!(
        "first view:   {} ms after start ({})",
        started.elapsed().as_millis(),
        if first.save.is_some() {
            "from the cache"
        } else {
            "nothing cached yet"
        }
    );

    // Wait for the background check to finish.
    let mut snap = first;
    let deadline = Instant::now() + Duration::from_secs(60);
    while snap.refreshing {
        let left = deadline.saturating_duration_since(Instant::now());
        if rx.recv_timeout(left).is_err() {
            eprintln!("the engine did not finish within 60 s");
            return ExitCode::FAILURE;
        }
        snap = engine.snapshot();
    }
    println!(
        "checked:      {} ms after start (view is current)",
        started.elapsed().as_millis()
    );
    print_snapshot(&snap);

    if watch_secs > 0 {
        println!("watching:     save in the game now, waiting up to {watch_secs} s");
        let end = Instant::now() + Duration::from_secs(watch_secs);
        let mut seen = snap.version;
        loop {
            let left = end.saturating_duration_since(Instant::now());
            match rx.recv_timeout(left) {
                Ok(_) => {
                    let s = engine.snapshot();
                    if s.version == seen {
                        continue;
                    }
                    seen = s.version;
                    let after_save = s.save.as_ref().and_then(|v| age_ms(v.stamp.modified_ns));
                    match after_save {
                        Some(ms) => println!(
                            "new snapshot: v{} appeared {ms} ms after the save's modified time",
                            s.version
                        ),
                        None => println!("new snapshot: v{}", s.version),
                    }
                    print_snapshot(&s);
                }
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        if seen == snap.version {
            println!("watching:     no change seen (the engine stayed asleep)");
        }
    }

    let mem = memory::read();
    let mb = |b: Option<u64>| {
        b.map_or("not reported".to_string(), |v| {
            format!("{:.1} MB", v as f64 / 1_048_576.0)
        })
    };
    println!(
        "memory:       peak private {}, now {}",
        mb(mem.peak_private_bytes),
        mb(mem.private_bytes)
    );
    ExitCode::SUCCESS
}

/// Milliseconds between a modified time (ns since 1970) and now, if the clock allows.
fn age_ms(modified_ns: u64) -> Option<u128> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    now.checked_sub(modified_ns as u128).map(|n| n / 1_000_000)
}

fn print_snapshot(s: &Snapshot) {
    match (&s.found, &s.found_error) {
        (Some(f), _) => println!(
            "game:         build {}, {} account folder(s), {} mod marker(s)",
            f.build_id,
            f.accounts,
            f.mod_markers.len()
        ),
        (None, Some(e)) => println!("game:         not found ({e})"),
        _ => println!("game:         unknown"),
    }
    match &s.save {
        Some(v) => {
            println!(
                "save:         {} slot(s), active {}",
                v.slots.len(),
                v.active_slot
                    .map_or("unknown".to_string(), |i| (i + 1).to_string())
            );
            for (i, slot) in v.slots.iter().enumerate() {
                println!(
                    "  slot {}: {} equipment ({} weapons, {} armour pieces), {} box item types, {} bosses with kills",
                    i + 1,
                    slot.equipment,
                    slot.weapons,
                    slot.armor_pieces,
                    slot.box_item_types,
                    slot.bosses_with_kills
                );
            }
        }
        None => println!("save:         none"),
    }
    if let Some(e) = &s.save_error {
        println!("save error:   {e}");
    }
    if let Some(e) = &s.watch_error {
        println!("watch error:  {e}");
    }
    match &s.data.manifest {
        Some(m) => println!(
            "game data:    build {}, {} files, {} classes, {} missing",
            m.build_id, m.files, m.classes, m.missing
        ),
        None => println!("game data:    none extracted"),
    }
    if let Some(p) = &s.data.patch_notice {
        println!(
            "patch notice: game is on build {}, data is from build {}",
            p.game_build, p.data_build
        );
    }
}
