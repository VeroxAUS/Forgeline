// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The update flow: read the files we need out of the game's own packs, rebuild the save's class
//! names from the game exe, check the result against the last good data, and swap it in only if
//! it passes. Anything that goes wrong leaves the last good data (`current/`) untouched.
//!
//! Everything is written under the app's data directory (`staging/` while it runs, then
//! `current/`); the game install, Steam and the save are only read. Nothing is fetched from the
//! network. The caller (the `--update` child process, or `forgeline-check update`) runs this once
//! and exits, so the pack index and file buffers never stay in the app's memory.

pub mod manifest;
pub mod status;
pub mod swap;
#[cfg(feature = "test-support")]
pub mod testtree;
pub mod validate;

use forgeline_core::datadir::{is_protected, DataDir};
use forgeline_core::discovery::{self, DiscoveryEnv, SaveAccount};
use forgeline_pak::exescan;
use forgeline_pak::index::PackSet;
use forgeline_pak::paths::{resolve, Category, NEEDED};
use forgeline_save::container;
use forgeline_save::hashes::collect;
use manifest::{fnv1a64, FileRecord, Manifest};
use status::{Kind, LastOutcome, UpdateStatus};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use validate::{blocks, validate, Finding};

pub use validate::Severity;

/// Name of the game exe in the install folder.
pub const EXE_NAME: &str = "MonsterHunterWilds.exe";
/// Name of the save file read for the class hashes (the active account's first slot file).
pub const SAVE_FILE: &str = "data001Slot.bin";
/// Namespaces tried when rebuilding class names.
const NAMESPACES: &[&str] = &["app.savedata"];
/// The extracted files are game data files that start like this container (`USR`, from the Phase
/// 1 notes in `docs/research.md`). If a real file starts differently the update is blocked and the
/// file is named, so a wrong assumption here is seen, never silent.
const CONTAINER_MAGIC: &[u8] = b"USR";

/// What the update needs to know. `wanted` is the set of class and field hashes the save uses
/// (see `forgeline_save::hashes::collect`), so only names that matter are rebuilt.
#[derive(Debug, Clone)]
pub struct UpdateEnv {
    pub install: PathBuf,
    pub exe: PathBuf,
    pub build_id: String,
    pub wanted: HashSet<u32>,
    /// Test hook: needed files to treat as not found (used to check that a broken staging is
    /// rejected). Empty in normal use.
    pub pretend_missing: Vec<String>,
}

/// One progress report: what is happening and how far along.
#[derive(Debug, Clone, Copy)]
pub struct Progress<'a> {
    pub phase: &'a str,
    pub done: u32,
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutcome {
    /// New data was swapped in. Warnings (not blocks) come with it.
    Replaced {
        build_id: String,
        findings: Vec<Finding>,
    },
    /// The new extraction holds the same data as the last good one; nothing was swapped.
    Unchanged { build_id: String },
    /// The new extraction failed the gate; the last good data stays. The findings say why.
    Rejected(Vec<Finding>),
    /// Something broke (a file could not be read or written, no save, and so on).
    Failed(String),
}

impl UpdateOutcome {
    pub fn kind(&self) -> Kind {
        match self {
            UpdateOutcome::Replaced { .. } => Kind::Replaced,
            UpdateOutcome::Unchanged { .. } => Kind::Unchanged,
            UpdateOutcome::Rejected(_) => Kind::Rejected,
            UpdateOutcome::Failed(_) => Kind::Failed,
        }
    }
}

impl UpdateOutcome {
    /// Process exit code for the command-line modes: 0 for `Replaced` and `Unchanged`, 3 when the
    /// gate rejected the new data, 1 when something broke.
    pub fn exit_code(&self) -> u8 {
        match self {
            UpdateOutcome::Replaced { .. } | UpdateOutcome::Unchanged { .. } => 0,
            UpdateOutcome::Rejected(_) => 3,
            UpdateOutcome::Failed(_) => 1,
        }
    }

    /// The outcome as lines of text: the word (`Replaced`, `Unchanged`, `Rejected`, `Failed`),
    /// then one line per finding or the error.
    pub fn describe(&self) -> Vec<String> {
        fn finding(f: &Finding) -> String {
            let tag = match f.severity {
                Severity::Block => "BLOCK",
                Severity::Warn => "warning",
            };
            format!("  {tag}: {}", f.message)
        }
        match self {
            UpdateOutcome::Replaced { build_id, findings } => {
                let mut v = vec![format!("Replaced (game build {build_id})")];
                v.extend(findings.iter().map(finding));
                v
            }
            UpdateOutcome::Unchanged { build_id } => {
                vec![format!("Unchanged (game build {build_id})")]
            }
            UpdateOutcome::Rejected(findings) => {
                let mut v = vec!["Rejected: the last good data was kept".to_string()];
                v.extend(findings.iter().map(finding));
                v
            }
            UpdateOutcome::Failed(why) => vec![format!("Failed: {why}")],
        }
    }
}

/// Fails with a message instead of an `io::Error` chain, naming what was being done.
fn io<T>(what: &str, r: std::io::Result<T>) -> Result<T, String> {
    r.map_err(|e| format!("{what}: {e}"))
}

fn category_name(c: Category) -> &'static str {
    match c {
        Category::Skills => "skills",
        Category::Weapons => "weapons",
        Category::Armor => "armor",
        Category::Decorations => "decorations",
        Category::Charms => "charms",
        Category::Artian => "artian",
        Category::Items => "items",
    }
}

/// Writes the progress file (best effort: a failure to write it never fails the update).
struct Reporter<'a> {
    data: &'a DataDir,
    last: Option<LastOutcome>,
    sink: &'a mut dyn FnMut(&Progress),
}

impl Reporter<'_> {
    fn report(&mut self, phase: &str, done: u32, total: u32) {
        (self.sink)(&Progress { phase, done, total });
        let s = UpdateStatus {
            running: true,
            phase: phase.to_string(),
            done,
            total,
            ..UpdateStatus::idle(self.last.clone())
        };
        let _ = s.write(&self.data.update_status());
    }

    fn finish(&self, outcome: &UpdateOutcome, build_id: Option<String>) {
        let (findings, error) = match outcome {
            UpdateOutcome::Replaced { findings, .. } => (findings.clone(), None),
            UpdateOutcome::Rejected(f) => (f.clone(), None),
            UpdateOutcome::Failed(e) => (vec![], Some(e.clone())),
            UpdateOutcome::Unchanged { .. } => (vec![], None),
        };
        let last = LastOutcome {
            kind: outcome.kind(),
            at: forgeline_core::results::utc_now(),
            build_id,
            findings,
            error,
        };
        let _ = UpdateStatus::idle(Some(last)).write(&self.data.update_status());
    }
}

/// Runs one update. Writes `staging/` and, only if the new data passes, swaps it into `current/`;
/// reports progress through `progress` and the status file.
pub fn run_update(
    env: &UpdateEnv,
    data: &DataDir,
    progress: &mut dyn FnMut(&Progress),
) -> UpdateOutcome {
    let last = UpdateStatus::read(&data.update_status()).and_then(|s| s.last);
    let mut rep = Reporter {
        data,
        last,
        sink: progress,
    };
    let outcome = match run(env, data, &mut rep) {
        Ok(o) => o,
        Err(msg) => {
            let _ = swap::clear_staging(data);
            UpdateOutcome::Failed(msg)
        }
    };
    rep.finish(&outcome, Some(env.build_id.clone()));
    outcome
}

fn run(env: &UpdateEnv, data: &DataDir, rep: &mut Reporter) -> Result<UpdateOutcome, String> {
    let total = NEEDED.len() as u32 + 4;
    rep.report("preparing", 0, total);
    io(
        "creating the data folder",
        std::fs::create_dir_all(data.root().join("data")),
    )?;
    io("recovering the last good data", swap::recover(data))?;
    io("clearing an old staging folder", swap::clear_staging(data))?;
    let staging = data.staging();
    io(
        "creating the staging folder",
        std::fs::create_dir_all(staging.join("raw")),
    )?;

    rep.report("reading the game's packs", 1, total);
    let set = PackSet::open(&env.install).map_err(|e| format!("reading the game's packs: {e}"))?;

    let mut files = Vec::new();
    let mut missing = Vec::new();
    for (i, n) in NEEDED.iter().enumerate() {
        rep.report("extracting needed files", 2 + i as u32, total);
        let found = if env.pretend_missing.iter().any(|m| m == n.stem) {
            None
        } else {
            resolve(&set, n.stem)
        };
        let Some((version, entry)) = found else {
            missing.push(n.stem.to_string());
            continue;
        };
        let bytes = set
            .read(&entry)
            .map_err(|e| format!("reading {}: {e}", n.stem))?;
        let out = staging.join("raw").join(format!("{}.{version}", n.stem));
        if let Some(parent) = out.parent() {
            io("creating a staging folder", std::fs::create_dir_all(parent))?;
        }
        io(&format!("writing {}", n.stem), std::fs::write(&out, &bytes))?;
        files.push(FileRecord {
            path: n.stem.to_string(),
            version,
            category: category_name(n.category).to_string(),
            size: bytes.len() as u64,
            hash: format!("{:016x}", fnv1a64(&bytes)),
            opens: bytes.starts_with(CONTAINER_MAGIC),
        });
    }
    drop(set);

    rep.report("rebuilding class names", total - 2, total);
    let scan = exescan::scan(&env.exe, &env.wanted, NAMESPACES)
        .map_err(|e| format!("scanning the game exe: {e}"))?;
    let mut classes: Vec<String> = scan
        .names
        .values()
        .filter(|n| n.contains('.'))
        .cloned()
        .collect();
    classes.sort();
    let names: Vec<&String> = scan.names.values().collect();
    let typenames = serde_json::json!({ "schema": 1, "names": names });
    io(
        "writing the type-name table",
        std::fs::write(staging.join("typenames.json"), typenames.to_string()),
    )?;

    let staged = Manifest {
        schema: manifest::SCHEMA,
        build_id: env.build_id.clone(),
        exe_size: scan.exe_size,
        exe_modified: scan.exe_modified,
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        created: forgeline_core::results::utc_now(),
        files,
        missing,
        classes,
    };
    io(
        "writing the manifest",
        std::fs::write(staging.join("manifest.json"), staged.to_json()),
    )?;

    rep.report("checking the new data", total - 1, total);
    let last_text = std::fs::read_to_string(data.current().join("manifest.json")).ok();
    let last_good = last_text.as_deref().and_then(Manifest::from_json);
    let mut findings = validate(&staged, last_good.as_ref());
    if data.current().exists() && last_good.is_none() {
        // A manifest from another app version cannot be compared but is not damaged: warn, so a
        // schema change never locks updates out. Anything else unreadable blocks.
        findings.push(if last_text.as_deref().is_some_and(Manifest::is_other_schema) {
            Finding {
                severity: Severity::Warn,
                message: "the last good data was made by another version of the app, so the new data was not compared with it"
                    .to_string(),
            }
        } else {
            Finding {
                severity: Severity::Block,
                message: "the last good data's manifest is damaged, so the new data cannot be checked against it; delete the data folder's current/ folder to start fresh"
                    .to_string(),
            }
        });
    }
    if blocks(&findings) {
        io("clearing the staging folder", swap::clear_staging(data))?;
        return Ok(UpdateOutcome::Rejected(findings));
    }
    if last_good
        .as_ref()
        .is_some_and(|old| old.same_data_as(&staged))
    {
        io("clearing the staging folder", swap::clear_staging(data))?;
        return Ok(UpdateOutcome::Unchanged {
            build_id: staged.build_id,
        });
    }

    rep.report("swapping in the new data", total, total);
    io("swapping in the new data", swap::commit(data))?;
    Ok(UpdateOutcome::Replaced {
        build_id: staged.build_id,
        findings,
    })
}

const PROTECTED_MSG: &str = "the data folder is inside Steam, the install or a save folder";

/// Finds the game and the save through Steam, then runs the update. This is what the `--update`
/// child process and `forgeline-check update` call. The save is only read (into memory) to learn
/// which names matter; nothing from it is kept.
pub fn update_from_steam(
    steam_root: PathBuf,
    data: &DataDir,
    pretend_missing: Vec<String>,
    progress: &mut dyn FnMut(&Progress),
) -> UpdateOutcome {
    let fail = |msg: String| {
        let last = UpdateStatus::read(&data.update_status()).and_then(|s| s.last);
        let rep = Reporter {
            data,
            last,
            sink: &mut |_| {},
        };
        let outcome = UpdateOutcome::Failed(msg);
        let _ = std::fs::create_dir_all(data.root().join("data"));
        rep.finish(&outcome, None);
        outcome
    };

    // Before discovery: if discovery fails, `fail` below writes into the data folder, so a data
    // folder inside the Steam root must be refused first.
    if is_protected(data.root(), std::slice::from_ref(&steam_root)) {
        return UpdateOutcome::Failed(PROTECTED_MSG.to_string());
    }
    let found = match discovery::discover(&DiscoveryEnv { steam_root }) {
        Ok(f) => f,
        Err(e) => return fail(format!("finding the game: {e}")),
    };
    if is_protected(data.root(), &found.protected_paths()) {
        // Not `fail`: that would create the data folder and write the status file inside it.
        return UpdateOutcome::Failed(PROTECTED_MSG.to_string());
    }
    let account = match found.save_account() {
        SaveAccount::One(a) => a,
        SaveAccount::None => {
            return fail(
                "no save was found, and the update needs it to know which names to rebuild"
                    .to_string(),
            )
        }
        SaveAccount::Several(_) => {
            return fail("several Steam accounts have Wilds saves; pick one first".to_string())
        }
    };
    let wanted = match wanted_from_save(&account.save_dir.join(SAVE_FILE), account.steam_id64) {
        Ok(w) => w,
        Err(e) => return fail(e),
    };
    let env = UpdateEnv {
        exe: found.install.join(EXE_NAME),
        install: found.install,
        build_id: found.build_id,
        wanted,
        pretend_missing,
    };
    run_update(&env, data, progress)
}

/// The class and field hashes the save uses, read from a copy of the save held in memory.
fn wanted_from_save(file: &Path, steam_id: u64) -> Result<HashSet<u32>, String> {
    let bytes = std::fs::read(file).map_err(|e| format!("reading the save: {e}"))?;
    let stream = container::open(&bytes, steam_id).map_err(|e| format!("opening the save: {e}"))?;
    drop(bytes);
    let hashes = collect(&stream).map_err(|e| format!("walking the save: {e}"))?;
    Ok(hashes.all().into_iter().collect())
}
