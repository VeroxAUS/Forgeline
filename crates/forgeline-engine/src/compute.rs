// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builds the next `Snapshot` from the previous one. Pure of threads and timers, so tests and
//! tools can call it directly.

use crate::snapshot::{DataView, FoundView, ManifestView, PatchNotice, SaveView, Snapshot, Stamp};
use forgeline_core::datadir::DataDir;
use forgeline_core::discovery::{self, Discovered, DiscoveryEnv, SaveAccount};
use forgeline_save::model::read_save;
use forgeline_update::manifest::Manifest;
use forgeline_update::status::UpdateStatus;
use forgeline_update::SAVE_FILE;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

/// A save copy to read instead of the live one.
#[derive(Debug, Clone)]
pub struct SaveOverride {
    pub path: PathBuf,
    pub steam_id64: u64,
}

pub struct ComputeInput<'a> {
    pub data: &'a DataDir,
    pub steam_root: Option<PathBuf>,
    pub account_id: Option<u32>,
    pub save_override: Option<&'a SaveOverride>,
    /// Read the save even if its size and modified time match the previous summary.
    pub force_read: bool,
}

pub struct ComputeOutput {
    pub snapshot: Snapshot,
    /// The save file that was (or would be) read; the engine watches its folder.
    pub save_path: Option<PathBuf>,
}

/// Tries to read the save this many times before giving up.
const READ_TRIES: u32 = 4;
const READ_RETRY_WAIT: Duration = Duration::from_millis(150);

pub fn compute(input: &ComputeInput, prev: &Snapshot) -> ComputeOutput {
    let mut next = prev.clone();
    next.watch_error = None;

    // Where is everything?
    let steam_root = input.steam_root.clone().or_else(discovery::find_steam_root);
    let found = match &steam_root {
        Some(root) => discovery::discover(&DiscoveryEnv {
            steam_root: root.clone(),
        })
        .map_err(|e| e.to_string()),
        None => Err("Steam was not found".to_string()),
    };

    // Which save file, and whose key?
    let mut account_used = None;
    let target: Result<(PathBuf, u64), String> = match (input.save_override, &found) {
        (Some(o), _) => Ok((o.path.clone(), o.steam_id64)),
        (None, Err(e)) => Err(e.clone()),
        (None, Ok(d)) => pick_save(d, input.account_id).map(|(path, id, acct)| {
            account_used = Some(acct);
            (path, id)
        }),
    };

    match &found {
        Ok(d) => {
            next.found = Some(FoundView::new(d, account_used));
            next.found_error = None;
        }
        Err(e) => {
            next.found = None;
            next.found_error = Some(e.clone());
        }
    }

    let save_path = match target {
        Ok((path, id)) => {
            read_into(&mut next, &path, id, input.force_read);
            Some(path)
        }
        Err(why) => {
            // The old summary stays; the reason is visible.
            next.save_error = Some(why);
            None
        }
    };

    next.data = data_view(input.data, found.as_ref().ok());
    ComputeOutput {
        snapshot: next,
        save_path,
    }
}

fn pick_save(d: &Discovered, wanted: Option<u32>) -> Result<(PathBuf, u64, u32), String> {
    let account = match (d.save_account(), wanted) {
        (SaveAccount::None, _) => {
            return Err("no Steam account folder with Wilds saves was found".into())
        }
        (SaveAccount::One(a), _) => a,
        (SaveAccount::Several(list), Some(id)) => list
            .iter()
            .find(|a| a.account_id == id)
            .ok_or("the chosen Steam account has no Wilds saves")?,
        (SaveAccount::Several(_), None) => {
            return Err("several Steam accounts have Wilds saves; choose one".into())
        }
    };
    let name = account
        .slot_files
        .iter()
        .find(|f| f.name.eq_ignore_ascii_case(SAVE_FILE))
        .map_or(SAVE_FILE, |f| f.name.as_str());
    Ok((
        account.save_dir.join(name),
        account.steam_id64,
        account.account_id,
    ))
}

/// Reads and summarises the save into `next`. On any failure the old summary stays and the error
/// is set; on success the error is cleared.
fn read_into(next: &mut Snapshot, path: &Path, steam_id64: u64, force: bool) {
    if !force {
        if let (Some(old), Ok(meta)) = (&next.save, std::fs::metadata(path)) {
            if old.stamp == stamp_of(&meta) {
                next.save_error = None;
                return;
            }
        }
    }
    match read_stable(path) {
        Ok((bytes, stamp)) => match read_save(&bytes, steam_id64) {
            Ok(summary) => {
                next.save = Some(SaveView::new(&summary, stamp));
                next.save_error = None;
            }
            Err(e) => next.save_error = Some(e.to_string()),
        },
        Err(why) => next.save_error = Some(why),
    }
}

pub(crate) fn stamp_of(meta: &std::fs::Metadata) -> Stamp {
    let modified_ns = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_nanos() as u64);
    Stamp {
        size: meta.len(),
        modified_ns,
    }
}

/// Reads the whole file into memory (sharing with the game) and checks that its size and
/// modified time did not change across the read. Retries a few times if they did or the file is
/// locked. All parsing is on the returned copy, never on a handle to the live file.
pub fn read_stable(path: &Path) -> Result<(Vec<u8>, Stamp), String> {
    let mut last = "the save could not be read".to_string();
    for attempt in 0..READ_TRIES {
        if attempt > 0 {
            std::thread::sleep(READ_RETRY_WAIT);
        }
        let before = match std::fs::metadata(path) {
            Ok(m) => stamp_of(&m),
            Err(e) => {
                last = format!("cannot read the save ({})", e.kind());
                continue;
            }
        };
        let mut bytes = Vec::with_capacity(before.size as usize);
        match std::fs::File::open(path).and_then(|mut f| f.read_to_end(&mut bytes)) {
            Ok(_) => {}
            Err(e) => {
                last = format!("cannot read the save ({})", e.kind());
                continue;
            }
        }
        match std::fs::metadata(path) {
            Ok(m) if stamp_of(&m) == before && bytes.len() as u64 == before.size => {
                return Ok((bytes, before));
            }
            _ => last = "the save changed while it was being read; trying again".into(),
        }
    }
    Err(format!("{last} (the game may be writing it)"))
}

fn data_view(data: &DataDir, found: Option<&Discovered>) -> DataView {
    let mut view = DataView::default();
    let path = data.current().join("manifest.json");
    match std::fs::read_to_string(&path) {
        Ok(text) => match Manifest::from_json(&text) {
            Some(m) => {
                if let Some(f) = found {
                    if !f.build_id.is_empty() && f.build_id != m.build_id {
                        view.patch_notice = Some(PatchNotice {
                            game_build: f.build_id.clone(),
                            data_build: m.build_id.clone(),
                        });
                    }
                }
                view.manifest = Some(ManifestView::new(&m));
            }
            None => {
                view.manifest_error =
                    Some("the extracted data's manifest is damaged or from another version".into())
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => view.manifest_error = Some(format!("cannot read the manifest ({})", e.kind())),
    }
    view.last_update = UpdateStatus::read(&data.update_status()).and_then(|s| s.last);
    view
}
