// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Forgeline entry point. Modes so far: `--headless-check` (used by CI) and `--update`, the
//! short-lived child process the window starts to refresh the game data. The window itself arrives
//! in step 9.

use forgeline_core::datadir::DataDir;
use forgeline_core::discovery;
use forgeline_core::{memory, results::BenchResult};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: forgeline --headless-check [<fixture dir>] [--results-dir <dir>] [--limit-mb <n>] [--ci]\n       forgeline --update [--steam-dir <folder>] [--data-dir <folder>]";

/// Private bytes the data and engine side may use with fixture data (section 3.4 of the Phase 2 plan).
const DEFAULT_LIMIT_MB: u64 = 20;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("--headless-check") => headless_check(args.collect()),
        Some("--update") => update(args.collect()),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}\nThe window is not built yet (Phase 2, step 9).");
            ExitCode::from(2)
        }
    }
}

/// Loads the fixture data and builds a snapshot without any window, then checks the process's peak
/// private bytes against the limit and writes a results file. With a fixture folder that holds
/// `saves/one-slot.bin`, the real engine (worker thread, save watcher, cache) runs on it first, so
/// the number includes the engine; the check grows with the code.
fn headless_check(args: Vec<String>) -> ExitCode {
    let mut fixture: Option<PathBuf> = None;
    let mut results_dir = PathBuf::from("bench/results");
    let mut limit_mb = DEFAULT_LIMIT_MB;
    let mut ci = std::env::var("CI").is_ok_and(|v| v == "true");
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--results-dir" => match it.next() {
                Some(d) => results_dir = PathBuf::from(d),
                None => return usage_error("--results-dir needs a folder"),
            },
            "--limit-mb" => match it.next().and_then(|v| v.parse().ok()) {
                Some(n) => limit_mb = n,
                None => return usage_error("--limit-mb needs a number"),
            },
            "--ci" => ci = true,
            other if other.starts_with('-') => {
                return usage_error(&format!("unknown flag {other}"))
            }
            other => fixture = Some(PathBuf::from(other)),
        }
    }
    if let Some(dir) = &fixture {
        if !dir.is_dir() {
            return usage_error("the fixture folder does not exist");
        }
    }

    if let Some(dir) = &fixture {
        if let Err(why) = run_engine_on_fixture(dir) {
            eprintln!("headless check FAILED: {why}");
            return ExitCode::FAILURE;
        }
    }

    let mem = memory::read();
    let Some(peak) = mem.peak_private_bytes.or(mem.private_bytes) else {
        eprintln!("headless check: the system did not report memory use");
        return ExitCode::FAILURE;
    };

    let mut result = BenchResult::new("ci-headless", ci, "bytes", vec![peak as f64]);
    result
        .conditions
        .insert("mode".into(), "headless-check".into());
    result.conditions.insert("window".into(), "none".into());
    result
        .conditions
        .insert("wilds".into(), "not running".into());
    result
        .conditions
        .insert("limit_mb".into(), limit_mb.to_string());
    if let Some(name) = fixture.as_deref().and_then(|p| p.file_name()) {
        result
            .conditions
            .insert("fixture".into(), name.to_string_lossy().into_owned());
    }
    match result.write_into(&results_dir) {
        Ok(path) => println!("results file: {}", path.display()),
        Err(e) => {
            eprintln!("headless check: cannot write the results file: {e}");
            return ExitCode::FAILURE;
        }
    }

    let mb = peak as f64 / (1024.0 * 1024.0);
    println!("peak private bytes: {mb:.1} MB (limit {limit_mb} MB)");
    if mb > limit_mb as f64 {
        eprintln!("headless check FAILED: {mb:.1} MB is over the {limit_mb} MB limit");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// The invented Steam ID the fixture saves are keyed with (`fixtures/README.md`).
const FIXTURE_STEAM_ID: u64 = 76_561_197_960_269_970;

/// Runs the engine on `<dir>/saves/one-slot.bin` (if there) with a throwaway data folder and waits
/// for the first complete snapshot. Nothing to do for a folder without the fixture save.
fn run_engine_on_fixture(dir: &std::path::Path) -> Result<(), String> {
    use forgeline_engine::{Engine, EngineConfig, SaveOverride};
    let save = dir.join("saves").join("one-slot.bin");
    if !save.is_file() {
        return Ok(());
    }
    let data_root = std::env::temp_dir().join(format!("forgeline-headless-{}", std::process::id()));
    let data = DataDir::open(Some(data_root.clone())).map_err(|e| e.to_string())?;
    let (tx, rx) = std::sync::mpsc::channel();
    let engine = Engine::start(
        EngineConfig {
            data,
            steam_root: Some(data_root.clone()),
            account_id: None,
            save_override: Some(SaveOverride {
                path: save,
                steam_id64: FIXTURE_STEAM_ID,
            }),
        },
        move || {
            let _ = tx.send(());
        },
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let result = loop {
        let s = engine.snapshot();
        if !s.refreshing {
            break match (&s.save, &s.save_error) {
                (Some(v), None) => {
                    println!("engine: fixture save read, {} slots", v.slots.len());
                    Ok(())
                }
                (_, Some(e)) => Err(format!("the engine could not read the fixture save: {e}")),
                _ => Err("the engine produced no save summary".to_string()),
            };
        }
        let left = deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() || rx.recv_timeout(left).is_err() {
            break Err("the engine did not finish within 10 s".to_string());
        }
    };
    drop(engine);
    let _ = std::fs::remove_dir_all(data_root);
    result
}

/// The update child process: finds the game and the save, extracts the needed files into the data
/// folder's `staging/`, checks them and swaps them in. Runs at below-normal priority, writes only
/// under the data folder, prints what it does and exits when done, so none of its memory stays.
/// Exit code 0 for `Replaced` and `Unchanged`, 3 for `Rejected`, 1 for `Failed`.
fn update(args: Vec<String>) -> ExitCode {
    let (mut steam_dir, mut data_dir) = (None, None);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let slot = match a.as_str() {
            "--steam-dir" => &mut steam_dir,
            "--data-dir" => &mut data_dir,
            other => return usage_error(&format!("unknown flag {other}")),
        };
        match it.next() {
            Some(v) => *slot = Some(PathBuf::from(v)),
            None => return usage_error(&format!("{a} needs a folder")),
        }
    }
    let Some(steam_root) = steam_dir.or_else(discovery::find_steam_root) else {
        eprintln!("Steam was not found; pass --steam-dir <folder>");
        return ExitCode::FAILURE;
    };
    let data = match DataDir::open(data_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    forgeline_core::priority::set_below_normal();
    let mut last = String::new();
    let outcome = forgeline_update::update_from_steam(steam_root, &data, Vec::new(), &mut |p| {
        if p.phase != last {
            println!("{} ({}/{})", p.phase, p.done, p.total);
            last = p.phase.to_string();
        }
    });
    for line in outcome.describe() {
        println!("{line}");
    }
    ExitCode::from(outcome.exit_code())
}

fn usage_error(msg: &str) -> ExitCode {
    eprintln!("{msg}\n{USAGE}");
    ExitCode::from(2)
}
