// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs `forgeline-check pak` over an invented Steam tree whose install holds a tiny synthetic
//! pack and a fake exe: the needed files resolve and read, a missing one is named and fails the
//! run, `pak_mods` is never opened, and the save's class names are rebuilt from the exe strings.

use forgeline_pak::paths::NEEDED;
use forgeline_pak::testpack::{build, plain, Data, Item};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/saves")
        .join(name)
}

/// A pack with one stored entry per stem, under version 7, with an encrypted table.
fn pack(stems: &[&str]) -> Vec<u8> {
    build(&plain(
        stems
            .iter()
            .map(|s| Item {
                path: format!("natives/stm/{s}.7"),
                data: Data::Stored(s.as_bytes().to_vec()),
            })
            .collect(),
    ))
}

fn tree(tag: &str, stems: &[&str]) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("forgeline-check-pak-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let steam = root.join("steam");
    let app = steam.join("steamapps");
    let install = app.join("common/MonsterHunterWilds");
    fs::create_dir_all(install.join("pak_mods")).unwrap();
    fs::write(
        app.join("appmanifest_2246340.acf"),
        "\"AppState\" { \"installdir\" \"MonsterHunterWilds\" \"buildid\" \"4242\" }",
    )
    .unwrap();
    fs::write(install.join("re_chunk_000.pak"), pack(stems)).unwrap();
    // Opening this would fail the run: it is junk, and mods are never read.
    fs::write(install.join("pak_mods").join("re_chunk_999.pak"), b"junk").unwrap();
    // The names the real save is known to use (class names are the short form here).
    let exe: Vec<u8> = [
        "cUserSaveData",
        "cUserSaveParam",
        "cEquipParam",
        "cEquipWork",
        "_Data",
        "Active",
        "_EquipBox",
    ]
    .iter()
    .flat_map(|s| s.bytes().chain([0, 0]))
    .collect();
    fs::write(install.join("MonsterHunterWilds.exe"), exe).unwrap();
    let saves = steam.join("userdata/4242/2246340/remote/win64_save");
    fs::create_dir_all(&saves).unwrap();
    fs::copy(fixture("two-slots.bin"), saves.join("data001Slot.bin")).unwrap();
    root
}

fn run(root: &Path, extra: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .arg("pak")
        .arg("--steam-dir")
        .arg(root.join("steam"))
        .args(extra)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

#[test]
fn resolves_and_reads_the_needed_files_and_never_opens_pak_mods() {
    let all: Vec<&str> = NEEDED.iter().map(|n| n.stem).collect();
    let root = tree("ok", &all);
    let (ok, text) = run(&root, &[]);
    assert!(ok, "{text}");
    let n = NEEDED.len();
    assert!(text.contains("packs:        1 opened"), "{text}");
    assert!(
        text.contains(&format!("files:        {n} distinct")),
        "{text}"
    );
    assert!(
        text.contains(&format!("needed files: {n} of {n} resolved, {n} read")),
        "{text}"
    );
    assert!(text.contains("memory:"), "{text}");
    assert!(!text.contains("re_chunk_999"), "{text}");
    assert!(!text.contains("FAILED"), "{text}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_missing_needed_file_is_named_and_fails_the_run() {
    let all: Vec<&str> = NEEDED.iter().map(|n| n.stem).collect();
    let root = tree("missing", &all[1..]);
    let (ok, text) = run(&root, &[]);
    assert!(!ok, "{text}");
    assert!(text.contains(&format!("NOT FOUND: {}", all[0])), "{text}");
    assert!(text.contains("result:       FAILED"), "{text}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rebuilds_the_save_names_from_the_exe_and_writes_the_names_file() {
    let all: Vec<&str> = NEEDED.iter().map(|n| n.stem).collect();
    let root = tree("names", &all);
    let names = root.join("names.txt");
    let work = root.join("work");
    let (ok, text) = run(
        &root,
        &[
            "--names",
            "--work-dir",
            work.to_str().unwrap(),
            "--names-out",
            names.to_str().unwrap(),
        ],
    );
    assert!(ok, "{text}");
    assert!(text.contains("exe scan:"), "{text}");
    assert!(text.contains("type names:   classes 4 of"), "{text}");
    let written = fs::read_to_string(&names).unwrap();
    for want in [
        "app.savedata.cUserSaveData",
        "app.savedata.cEquipWork",
        "_Data",
        "Active",
        "_EquipBox",
    ] {
        assert!(
            written.lines().any(|l| l == want),
            "{want} missing: {written}"
        );
    }
    for never in ["Fixture", "FIXT-", "NOPE"] {
        assert!(
            !text.contains(never) && !written.contains(never),
            "{never} leaked"
        );
    }
    let _ = fs::remove_dir_all(root);
}
