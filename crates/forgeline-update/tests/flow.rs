// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The update flow end to end over an invented Steam tree: the first update replaces, the second
//! says unchanged, a broken extraction is rejected and leaves the last good data alone, a game
//! update swaps in and keeps the old data as previous, and nothing outside the data folder is
//! written.

use forgeline_core::datadir::DataDir;
use forgeline_pak::paths::NEEDED;
use forgeline_update::manifest::Manifest;
use forgeline_update::status::{Kind, UpdateStatus};
use forgeline_update::testtree::{make, temp_root, TreeOptions};
use forgeline_update::validate::Severity;
use forgeline_update::{update_from_steam, Progress, UpdateOutcome};
use std::path::{Path, PathBuf};

fn data_dir(root: &Path) -> DataDir {
    DataDir::open(Some(root.join("appdata"))).unwrap()
}

fn update(steam: &Path, data: &DataDir, missing: &[&str]) -> (UpdateOutcome, Vec<String>) {
    let mut phases = Vec::new();
    let outcome = update_from_steam(
        steam.to_path_buf(),
        data,
        missing.iter().map(|s| s.to_string()).collect(),
        &mut |p: &Progress| {
            if phases.last().map(String::as_str) != Some(p.phase) {
                phases.push(p.phase.to_string());
            }
        },
    );
    (outcome, phases)
}

fn manifest(data: &DataDir) -> Manifest {
    Manifest::from_json(&std::fs::read_to_string(data.current().join("manifest.json")).unwrap())
        .unwrap()
}

/// Every file under `dir` with its size and modified time, to prove nothing changed.
fn snapshot(dir: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            let m = e.metadata().unwrap();
            if m.is_dir() {
                stack.push(p);
            } else {
                out.push((p, m.len(), m.modified().unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn first_update_replaces_and_the_second_says_unchanged() {
    let root = temp_root("twice");
    let steam = make(&root, &TreeOptions::default());
    let data = data_dir(&root);

    let (first, phases) = update(&steam, &data, &[]);
    match &first {
        UpdateOutcome::Replaced { build_id, findings } => {
            assert_eq!(build_id, "100");
            assert!(findings.is_empty(), "{findings:?}");
        }
        other => panic!("{other:?}"),
    }
    assert!(
        phases.contains(&"extracting needed files".to_string()),
        "{phases:?}"
    );
    let m = manifest(&data);
    assert_eq!(m.files.len(), NEEDED.len());
    assert!(m.missing.is_empty());
    assert!(
        m.classes.contains(&"app.savedata.cEquipWork".to_string()),
        "{:?}",
        m.classes
    );
    assert!(data.current().join("typenames.json").is_file());
    assert!(!data.staging().exists());
    assert!(!data.previous().exists());
    // Extracted files are on disk under raw/, named with their version.
    assert!(data
        .current()
        .join("raw/GameDesign/Common/Equip/SkillData.user.7")
        .is_file());

    let before = snapshot(&data.current());
    let (second, _) = update(&steam, &data, &[]);
    assert_eq!(
        second,
        UpdateOutcome::Unchanged {
            build_id: "100".into()
        }
    );
    assert_eq!(
        snapshot(&data.current()),
        before,
        "an unchanged update rewrote current/"
    );
    assert!(!data.staging().exists());

    let status = UpdateStatus::read(&data.update_status()).unwrap();
    assert!(!status.running);
    assert_eq!(status.last.unwrap().kind, Kind::Unchanged);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_broken_staging_is_rejected_with_the_missing_file_named_and_current_is_untouched() {
    let root = temp_root("broken");
    let steam = make(&root, &TreeOptions::default());
    let data = data_dir(&root);
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    let before = snapshot(&data.current());

    let gone = NEEDED[3].stem;
    let (outcome, _) = update(&steam, &data, &[gone]);
    let UpdateOutcome::Rejected(findings) = outcome else {
        panic!("{outcome:?}")
    };
    assert!(
        findings
            .iter()
            .any(|f| f.severity == Severity::Block && f.message.contains(gone)),
        "{findings:?}"
    );
    assert_eq!(snapshot(&data.current()), before);
    assert!(
        !data.staging().exists(),
        "a rejected staging folder was left behind"
    );
    assert!(!data.previous().exists());
    let last = UpdateStatus::read(&data.update_status())
        .unwrap()
        .last
        .unwrap();
    assert_eq!(last.kind, Kind::Rejected);
    assert!(last.findings.iter().any(|f| f.message.contains(gone)));
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn the_first_update_is_rejected_too_when_a_file_is_missing_or_does_not_open() {
    let root = temp_root("first-bad");
    let data = data_dir(&root);
    let omit = NEEDED[0].stem.to_string();
    let steam = make(
        &root,
        &TreeOptions {
            omit: vec![omit.clone()],
            ..TreeOptions::default()
        },
    );
    let (o, _) = update(&steam, &data, &[]);
    assert!(
        matches!(&o, UpdateOutcome::Rejected(f) if f.iter().any(|x| x.message.contains(&omit))),
        "{o:?}"
    );
    assert!(!data.current().exists());

    let steam = make(
        &root,
        &TreeOptions {
            valid_containers: false,
            ..TreeOptions::default()
        },
    );
    let (o, _) = update(&steam, &data, &[]);
    assert!(
        matches!(&o, UpdateOutcome::Rejected(f) if f.iter().any(|x| x.message.contains("does not open"))),
        "{o:?}"
    );
    assert!(!data.current().exists());
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_game_update_swaps_in_and_keeps_the_old_data_as_previous() {
    let root = temp_root("patch");
    let data = data_dir(&root);
    let steam = make(&root, &TreeOptions::default());
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));

    // The game is patched: a new build and changed files.
    make(
        &root,
        &TreeOptions {
            build_id: "101".into(),
            tag: "b".into(),
            ..TreeOptions::default()
        },
    );
    let (o, _) = update(&steam, &data, &[]);
    let UpdateOutcome::Replaced { build_id, findings } = o else {
        panic!("{o:?}")
    };
    assert_eq!(build_id, "101");
    assert!(findings
        .iter()
        .any(|f| f.severity == Severity::Warn && f.message.contains("new game build 101")));
    assert_eq!(manifest(&data).build_id, "101");
    let old: Manifest = Manifest::from_json(
        &std::fs::read_to_string(data.previous().join("manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(old.build_id, "100");

    // An older build than the current data is refused.
    make(
        &root,
        &TreeOptions {
            build_id: "99".into(),
            tag: "c".into(),
            ..TreeOptions::default()
        },
    );
    let (o, _) = update(&steam, &data, &[]);
    assert!(
        matches!(&o, UpdateOutcome::Rejected(f) if f.iter().any(|x| x.message.contains("older"))),
        "{o:?}"
    );
    assert_eq!(manifest(&data).build_id, "101");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn losing_a_class_blocks_the_update() {
    let root = temp_root("class");
    let data = data_dir(&root);
    let steam = make(&root, &TreeOptions::default());
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    make(
        &root,
        &TreeOptions {
            exe_classes: vec!["cUserSaveData".into()],
            tag: "b".into(),
            ..TreeOptions::default()
        },
    );
    let (o, _) = update(&steam, &data, &[]);
    assert!(
        matches!(&o, UpdateOutcome::Rejected(f) if f.iter().any(|x| x.message.contains("app.savedata.cEquipWork"))),
        "{o:?}"
    );
    assert_eq!(manifest(&data).build_id, "100");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn an_interrupted_swap_is_recovered_on_the_next_run() {
    let root = temp_root("recover");
    let data = data_dir(&root);
    let steam = make(&root, &TreeOptions::default());
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    // Simulate a crash between the two renames: current moved to previous, nothing swapped in.
    std::fs::rename(data.current(), data.previous()).unwrap();
    std::fs::create_dir_all(data.staging().join("raw")).unwrap();
    let (o, _) = update(&steam, &data, &[]);
    assert_eq!(
        o,
        UpdateOutcome::Unchanged {
            build_id: "100".into()
        }
    );
    assert!(data.current().join("manifest.json").is_file());
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn only_the_data_folder_is_written_and_the_install_and_save_are_left_alone() {
    let root = temp_root("readonly");
    let steam = make(&root, &TreeOptions::default());
    let data = data_dir(&root);
    let before = snapshot(&steam);
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Unchanged { .. }
    ));
    assert_eq!(snapshot(&steam), before);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn failures_before_the_extraction_are_reported_and_leave_data_alone() {
    let root = temp_root("fail");
    let data = data_dir(&root);
    // No Steam folder at all.
    let (o, _) = update(&root.join("nowhere"), &data, &[]);
    assert!(matches!(o, UpdateOutcome::Failed(_)), "{o:?}");
    assert!(!data.current().exists());
    let last = UpdateStatus::read(&data.update_status())
        .unwrap()
        .last
        .unwrap();
    assert_eq!(last.kind, Kind::Failed);
    assert!(last.error.is_some());

    // A data folder inside the install is refused.
    let steam = make(&root, &TreeOptions::default());
    let inside = DataDir::open(Some(
        steam.join("steamapps/common/MonsterHunterWilds/appdata"),
    ));
    assert!(
        inside.is_err(),
        "the data directory rules should refuse the install folder"
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_damaged_last_good_manifest_blocks_the_update_and_leaves_current_alone() {
    let root = temp_root("damaged");
    let steam = make(&root, &TreeOptions::default());
    let data = data_dir(&root);
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    std::fs::write(data.current().join("manifest.json"), "{ not json").unwrap();
    let before = snapshot(&data.current());

    let (outcome, _) = update(&steam, &data, &[]);
    let UpdateOutcome::Rejected(findings) = outcome else {
        panic!("{outcome:?}")
    };
    assert!(
        findings
            .iter()
            .any(|f| f.severity == Severity::Block && f.message.contains("damaged")),
        "{findings:?}"
    );
    assert_eq!(snapshot(&data.current()), before);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_manifest_from_another_schema_warns_but_does_not_lock_updates_out() {
    let root = temp_root("otherschema");
    let steam = make(&root, &TreeOptions::default());
    let data = data_dir(&root);
    assert!(matches!(
        update(&steam, &data, &[]).0,
        UpdateOutcome::Replaced { .. }
    ));
    std::fs::write(data.current().join("manifest.json"), "{\"schema\": 999}").unwrap();

    let (outcome, _) = update(&steam, &data, &[]);
    let UpdateOutcome::Replaced { findings, .. } = outcome else {
        panic!("{outcome:?}")
    };
    assert!(
        findings
            .iter()
            .any(|f| f.severity == Severity::Warn && f.message.contains("another version")),
        "{findings:?}"
    );
    assert_eq!(manifest(&data).schema, forgeline_update::manifest::SCHEMA);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_data_folder_inside_steam_that_the_name_check_misses_is_refused_without_writing_there() {
    let root = temp_root("protected");
    let steam = make(&root, &TreeOptions::default());
    // A Steam folder with an ordinary name: only discovery knows it is Steam.
    let valve = root.join("valve");
    std::fs::rename(&steam, &valve).unwrap();
    let data = DataDir::open(Some(valve.join("forgeline-data"))).unwrap();

    let (outcome, _) = update(&valve, &data, &[]);
    assert!(matches!(outcome, UpdateOutcome::Failed(_)), "{outcome:?}");
    assert!(
        !data.root().exists(),
        "the refusal created or wrote the data folder"
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn a_data_folder_inside_the_steam_folder_is_refused_even_when_discovery_fails() {
    let root = temp_root("protected-nogame");
    // An ordinary-named folder that is the Steam root but holds no game.
    let valve = root.join("valve");
    std::fs::create_dir_all(&valve).unwrap();
    let data = DataDir::open(Some(valve.join("forgeline-data"))).unwrap();

    let (outcome, _) = update(&valve, &data, &[]);
    assert!(matches!(outcome, UpdateOutcome::Failed(_)), "{outcome:?}");
    assert!(
        !data.root().exists(),
        "the failure wrote into the Steam folder"
    );
    std::fs::remove_dir_all(root).ok();
}
