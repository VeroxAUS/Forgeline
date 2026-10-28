// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check update`: runs the update flow once into a data folder you name and reports the
//! outcome, the time and the memory. Never touches the installed copy's data (the data folder is a
//! required argument), never writes to Steam, the install or the save.

use crate::{shown, USAGE};
use forgeline_core::datadir::DataDir;
use forgeline_core::discovery;
use forgeline_core::memory;
use forgeline_update::manifest::Manifest;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Instant;

pub fn run(args: Vec<String>) -> ExitCode {
    let (mut steam_dir, mut data_dir) = (None, None);
    let mut missing = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let Some(v) = it.next() else {
            eprintln!("{a} needs a value\n{USAGE}");
            return ExitCode::from(2);
        };
        match a.as_str() {
            "--steam-dir" => steam_dir = Some(PathBuf::from(v)),
            "--data-dir" => data_dir = Some(PathBuf::from(v)),
            "--simulate-missing" => missing.push(v),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(data_dir) = data_dir else {
        eprintln!(
            "update needs --data-dir <folder> (a dev folder, never the installed copy's)\n{USAGE}"
        );
        return ExitCode::from(2);
    };
    let Some(steam_root) = steam_dir.or_else(discovery::find_steam_root) else {
        eprintln!("Steam was not found; pass --steam-dir <folder>");
        return ExitCode::FAILURE;
    };
    let data = match DataDir::open(Some(data_dir)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    println!("data folder:  {}", shown(data.root()));
    if let Some(m) = missing.first() {
        println!(
            "simulating:   {} treated as not found ({} file(s))",
            m,
            missing.len()
        );
    }

    forgeline_core::priority::set_below_normal();
    let started = Instant::now();
    let mut last = String::new();
    let outcome = forgeline_update::update_from_steam(steam_root, &data, missing, &mut |p| {
        if p.phase != last {
            println!(
                "  {} ({}/{}), {} ms",
                p.phase,
                p.done,
                p.total,
                started.elapsed().as_millis()
            );
            last = p.phase.to_string();
        }
    });
    println!("update took:  {} ms", started.elapsed().as_millis());
    for line in outcome.describe() {
        println!("result:       {line}");
    }
    if let Some(m) = std::fs::read_to_string(data.current().join("manifest.json"))
        .ok()
        .and_then(|t| Manifest::from_json(&t))
    {
        let bytes: u64 = m.files.iter().map(|f| f.size).sum();
        println!(
            "current data: build {}, {} files ({} bytes), {} classes named, {} needed file(s) missing",
            m.build_id,
            m.files.len(),
            bytes,
            m.classes.len(),
            m.missing.len()
        );
    } else {
        println!("current data: none");
    }
    println!(
        "previous:     {}",
        if data.previous().exists() {
            "kept"
        } else {
            "none"
        }
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
    ExitCode::from(outcome.exit_code())
}
