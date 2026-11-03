// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The engine against fixture saves (invented files under `fixtures/`; no real save, no game).

use forgeline_core::datadir::DataDir;
use forgeline_engine::{Engine, EngineConfig, SaveOverride, Snapshot};
use forgeline_update::manifest::{Manifest, SCHEMA};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves")
}

fn steam_id() -> u64 {
    let text = std::fs::read_to_string(fixtures().join("one-slot.expected.json")).unwrap();
    serde_json::from_str::<serde_json::Value>(&text).unwrap()["steam_id64"]
        .as_u64()
        .unwrap()
}

struct Rig {
    root: PathBuf,
    save: PathBuf,
}

impl Rig {
    fn new(name: &str) -> Rig {
        let root =
            std::env::temp_dir().join(format!("forgeline-engine-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("save")).unwrap();
        let save = root.join("save").join("data001Slot.bin");
        let rig = Rig { root, save };
        rig.put("one-slot.bin");
        rig
    }
    fn put(&self, fixture: &str) {
        // A write, not a copy: on Windows a copy keeps the source's modified time, and the fixtures
        // all share one checkout time, so the engine could not tell them apart.
        std::fs::write(&self.save, std::fs::read(fixtures().join(fixture)).unwrap()).unwrap();
    }
    fn start(&self) -> (Engine, Receiver<()>) {
        let (tx, rx) = channel();
        let engine = Engine::start(
            EngineConfig {
                data: DataDir::open(Some(self.root.join("data-dir"))).unwrap(),
                // An empty folder: discovery reports "game not installed", the save override is used.
                steam_root: Some(self.root.join("save")),
                account_id: None,
                save_override: Some(SaveOverride {
                    path: self.save.clone(),
                    steam_id64: steam_id(),
                }),
            },
            move || {
                let _ = tx.send(());
            },
        );
        (engine, rx)
    }
}

impl Drop for Rig {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn wait_for(
    engine: &Engine,
    rx: &Receiver<()>,
    what: &str,
    pred: impl Fn(&Snapshot) -> bool,
) -> Arc<Snapshot> {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let s = engine.snapshot();
        if pred(&s) {
            return s;
        }
        let left = end.saturating_duration_since(Instant::now());
        assert!(!left.is_zero(), "timed out waiting for: {what}");
        let _ = rx.recv_timeout(left);
    }
}

fn settled(s: &Snapshot) -> bool {
    !s.refreshing && s.save.is_some()
}

#[test]
fn a_changed_save_produces_a_new_snapshot() {
    let rig = Rig::new("changed");
    let (engine, rx) = rig.start();
    let first = wait_for(&engine, &rx, "first view", settled);
    assert_eq!(first.save.as_ref().unwrap().active_slot, Some(0));
    assert!(
        first.found_error.is_some(),
        "no game in the empty Steam folder"
    );

    rig.put("two-slots.bin");
    let second = wait_for(&engine, &rx, "changed view", |s| {
        s.save.as_ref().is_some_and(|v| v.active_slot == Some(2))
    });
    assert!(second.version > first.version);
    assert!(second.save_error.is_none());
}

#[test]
fn a_corrupt_save_keeps_the_old_summary_and_shows_an_error() {
    let rig = Rig::new("corrupt");
    let (engine, rx) = rig.start();
    let good = wait_for(&engine, &rx, "first view", settled);

    std::fs::write(&rig.save, b"half a file").unwrap();
    let bad = wait_for(&engine, &rx, "error shown", |s| s.save_error.is_some());
    assert_eq!(bad.save, good.save, "the old summary stays");

    rig.put("one-slot.bin");
    let fixed = wait_for(&engine, &rx, "error cleared", |s| {
        s.save_error.is_none() && s.save.is_some()
    });
    assert_eq!(fixed.save.as_ref().unwrap().active_slot, Some(0));
}

#[test]
fn many_quick_changes_end_on_the_last_one() {
    let rig = Rig::new("burst");
    let (engine, rx) = rig.start();
    wait_for(&engine, &rx, "first view", settled);
    for i in 0..6 {
        rig.put(if i % 2 == 0 {
            "two-slots.bin"
        } else {
            "one-slot.bin"
        });
        std::thread::sleep(Duration::from_millis(30));
    }
    rig.put("two-slots.bin");
    wait_for(&engine, &rx, "last write seen", |s| {
        s.save.as_ref().is_some_and(|v| v.active_slot == Some(2)) && s.save_error.is_none()
    });
}

#[test]
fn a_second_start_has_the_cached_view_and_the_cache_holds_no_hunter_data() {
    let rig = Rig::new("cache");
    {
        let (engine, rx) = rig.start();
        wait_for(&engine, &rx, "first view", settled);
    }
    let cache = std::fs::read_to_string(rig.root.join("data-dir").join("view-cache.json")).unwrap();
    for never in ["Fixture Hunter", "FIXT-0001-NOPE"] {
        assert!(!cache.contains(never));
    }

    let (engine, rx) = rig.start();
    // Available the moment `start` returns, before the background check has run.
    assert!(
        engine.snapshot().save.is_some(),
        "the cached view is there at once"
    );
    let settled_view = wait_for(&engine, &rx, "cache confirmed", |s| !s.refreshing);
    assert_eq!(settled_view.save.as_ref().unwrap().active_slot, Some(0));
}

#[test]
fn a_cache_for_an_older_save_is_replaced_by_the_new_one() {
    let rig = Rig::new("stale");
    {
        let (engine, rx) = rig.start();
        wait_for(&engine, &rx, "first view", settled);
    }
    rig.put("two-slots.bin");
    let (engine, rx) = rig.start();
    wait_for(&engine, &rx, "recomputed", |s| {
        !s.refreshing && s.save.as_ref().is_some_and(|v| v.active_slot == Some(2))
    });
}

#[test]
fn refresh_picks_up_the_extracted_data_and_flags_nothing_without_a_game() {
    let rig = Rig::new("data");
    let (engine, rx) = rig.start();
    let first = wait_for(&engine, &rx, "first view", settled);
    assert!(first.data.manifest.is_none());

    let manifest = Manifest {
        schema: SCHEMA,
        build_id: "100".into(),
        exe_size: 1,
        exe_modified: None,
        tool_version: "test".into(),
        created: "2026-01-01T00:00:00Z".into(),
        files: vec![],
        missing: vec![],
        classes: vec!["a".into(), "b".into()],
    };
    let current = rig.root.join("data-dir").join("data").join("current");
    std::fs::create_dir_all(&current).unwrap();
    std::fs::write(current.join("manifest.json"), manifest.to_json()).unwrap();
    engine.refresh();
    let s = wait_for(&engine, &rx, "manifest seen", |s| s.data.manifest.is_some());
    assert_eq!(s.data.manifest.as_ref().unwrap().classes, 2);
    assert!(
        s.data.patch_notice.is_none(),
        "no game build to compare with"
    );
}

#[test]
fn a_swap_cut_short_is_recovered_when_the_engine_starts() {
    let rig = Rig::new("recover");
    let manifest = Manifest {
        schema: SCHEMA,
        build_id: "100".into(),
        exe_size: 1,
        exe_modified: None,
        tool_version: "test".into(),
        created: "2026-01-01T00:00:00Z".into(),
        files: vec![],
        missing: vec![],
        classes: vec!["a".into()],
    };
    // The state after a crash between the two renames: `previous` holds the data, no `current`.
    let previous = rig.root.join("data-dir").join("data").join("previous");
    std::fs::create_dir_all(&previous).unwrap();
    std::fs::write(previous.join("manifest.json"), manifest.to_json()).unwrap();

    let (engine, rx) = rig.start();
    wait_for(&engine, &rx, "recovered data seen", |s| {
        s.data.manifest.is_some()
    });
    assert!(rig
        .root
        .join("data-dir")
        .join("data")
        .join("current")
        .join("manifest.json")
        .is_file());
}
