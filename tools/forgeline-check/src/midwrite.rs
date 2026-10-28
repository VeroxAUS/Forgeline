// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check midwrite`: shows what the engine does when the save is read while it is being
//! written. It works on a copy of a save in a work folder you name, never the live save: the
//! engine watches the copy while a helper rewrites it the slow way (truncate, then small chunks
//! with pauses longer than the engine's quiet period, so the engine really reads the half file),
//! and, on Windows, holds the copy open with no sharing so reads are refused (a real NTFS lock).
//!
//! Pass if every snapshot that shows a summary shows the original one, a read of a half-written or
//! locked file produced a visible error, and the final snapshot is the original again with no
//! error. Counts only; no hunter name or ID is printed.

use crate::{parse_steam_id, shown, USAGE};
use forgeline_core::datadir::{is_protected, DataDir};
use forgeline_engine::{Engine, EngineConfig, SaveOverride, Snapshot};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc::{channel, Sender};
use std::time::{Duration, Instant};

/// Longer than the engine's 0.5 s quiet period, so each pause lets it read the file as it is.
const PAUSE: Duration = Duration::from_millis(900);
const CHUNKS: usize = 4;
#[cfg(windows)]
const LOCK_HOLD: Duration = Duration::from_millis(2500);

pub fn run(args: Vec<String>) -> ExitCode {
    let (mut file, mut steam_id, mut work) = (None, None, None);
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        let Some(v) = it.next() else {
            eprintln!("{a} needs a value\n{USAGE}");
            return ExitCode::from(2);
        };
        match a.as_str() {
            "--file" => file = Some(PathBuf::from(v)),
            "--steam-id" => steam_id = parse_steam_id(&v),
            "--work-dir" => work = Some(PathBuf::from(v)),
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(file), Some(steam_id), Some(work)) = (file, steam_id, work) else {
        eprintln!("midwrite needs --file <copy of the save>, --steam-id <id> and --work-dir <folder>\n{USAGE}");
        return ExitCode::from(2);
    };
    // The work folder must not be a game, Steam or save folder (the names are checked too).
    if is_protected(&work, &[]) {
        eprintln!("refusing that folder: it looks like a Steam, game or save folder");
        return ExitCode::FAILURE;
    }
    let data = match DataDir::open(Some(work.join("engine-data"))) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let original = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot read {}: {e}", shown(&file));
            return ExitCode::FAILURE;
        }
    };
    let target = work.join("save-under-test.bin");
    if let Err(e) = std::fs::create_dir_all(&work).and_then(|_| std::fs::write(&target, &original))
    {
        eprintln!("cannot prepare the work folder: {e}");
        return ExitCode::FAILURE;
    }
    println!("work folder:  {}", shown(&work));
    println!(
        "save copy:    {} bytes (a copy; the live save is never touched)",
        original.len()
    );

    let (tx, rx) = channel();
    let engine = Engine::start(
        EngineConfig {
            data,
            steam_root: Some(work.clone()),
            account_id: None,
            save_override: Some(SaveOverride {
                path: target.clone(),
                steam_id64: steam_id,
            }),
        },
        move || {
            let _ = tx.send(());
        },
    );
    // Wait for the first complete view; that is the "good" one everything is compared with.
    let deadline = Instant::now() + Duration::from_secs(30);
    let good = loop {
        let s = engine.snapshot();
        if !s.refreshing {
            break s;
        }
        if rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .is_err()
        {
            eprintln!("the engine did not finish its first view");
            return ExitCode::FAILURE;
        }
    };
    let Some(good_save) = good.save.clone() else {
        eprintln!(
            "the engine could not read the copy: {}",
            good.save_error.as_deref().unwrap_or("no reason given")
        );
        return ExitCode::FAILURE;
    };
    println!(
        "good view:    v{}, {} slot(s)",
        good.version,
        good_save.slots.len()
    );

    // The helper rewrites the copy slowly while this thread records every snapshot.
    let (done_tx, done_rx) = channel::<()>();
    let writer = {
        let (target, original) = (target.clone(), original.clone());
        std::thread::spawn(move || write_slowly(&target, &original, done_tx))
    };
    let mut seen: Vec<std::sync::Arc<Snapshot>> = Vec::new();
    let mut last_version = good.version;
    let mut finished_at: Option<Instant> = None;
    loop {
        let _ = rx.recv_timeout(Duration::from_millis(200));
        let s = engine.snapshot();
        if s.version != last_version {
            last_version = s.version;
            seen.push(s);
        }
        if finished_at.is_none() && done_rx.try_recv().is_ok() {
            finished_at = Some(Instant::now());
        }
        // After the writer is done, give the engine time to settle on the final file.
        if finished_at.is_some_and(|t| t.elapsed() > Duration::from_secs(3)) {
            break;
        }
    }
    let _ = writer.join();

    let mut wrong = 0;
    let mut errors = 0;
    for s in &seen {
        let matches = s.save.as_ref().is_some_and(|v| v.slots == good_save.slots);
        if !matches {
            wrong += 1;
        }
        if s.save_error.is_some() {
            errors += 1;
        }
        println!(
            "snapshot v{}: {}; {}",
            s.version,
            if matches {
                "original summary kept"
            } else {
                "WRONG SUMMARY"
            },
            s.save_error.as_deref().unwrap_or("no error")
        );
    }
    let last = engine.snapshot();
    let recovered = last.save_error.is_none()
        && last
            .save
            .as_ref()
            .is_some_and(|v| v.slots == good_save.slots);
    println!(
        "snapshots:    {} seen, {errors} with a visible error, {wrong} wrong",
        seen.len()
    );
    println!(
        "final view:   {}",
        if recovered {
            "original summary, no error"
        } else {
            "NOT RECOVERED"
        }
    );
    if !cfg!(windows) {
        println!("lock test:    skipped (the no-sharing lock is Windows only)");
    }
    if wrong == 0 && errors > 0 && recovered {
        println!("result:       PASS: a half-written or locked save never gave a wrong view");
        ExitCode::SUCCESS
    } else {
        println!("result:       FAIL");
        ExitCode::FAILURE
    }
}

/// Truncates the copy and writes it back in chunks with pauses (the engine reads the half file
/// during each pause); on Windows then holds it open with no sharing; finally writes it whole.
fn write_slowly(path: &Path, content: &[u8], done: Sender<()>) {
    let chunk = content.len().div_ceil(CHUNKS);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
    {
        for part in content.chunks(chunk).take(CHUNKS - 1) {
            let _ = f.write_all(part);
            let _ = f.flush();
            std::thread::sleep(PAUSE);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // share_mode(0): nobody else may open it, like a program that has it locked.
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .share_mode(0)
            .open(path)
        {
            let _ = f.write_all(&content[..content.len() / 2]);
            std::thread::sleep(LOCK_HOLD);
        }
    }
    let _ = std::fs::write(path, content);
    let _ = done.send(());
}
