// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs `forgeline-check update` over an invented Steam tree: Replaced, Unchanged, and the
//! deliberately broken staging (`--simulate-missing`) rejected with the file named while the last
//! good data stays.

use forgeline_pak::paths::NEEDED;
use forgeline_update::testtree::{make, temp_root, TreeOptions};
use std::path::Path;
use std::process::Command;

fn run(steam: &Path, data: &Path, extra: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .arg("update")
        .arg("--steam-dir")
        .arg(steam)
        .arg("--data-dir")
        .arg(data)
        .args(extra)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.code().unwrap_or(-1), text)
}

#[test]
fn replaced_then_unchanged_then_a_simulated_missing_file_is_rejected() {
    let root = temp_root("check-update");
    let data = root.join("appdata");
    let steam = make(&root, &TreeOptions::default());

    let (code, text) = run(&steam, &data, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(
        text.contains("result:       Replaced (game build 100)"),
        "{text}"
    );
    assert!(text.contains(&format!("{} files", NEEDED.len())), "{text}");
    assert!(text.contains("previous:     none"), "{text}");
    assert!(text.contains("update took:"), "{text}");

    let (code, text) = run(&steam, &data, &[]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("result:       Unchanged"), "{text}");

    let gone = NEEDED[5].stem;
    let (code, text) = run(&steam, &data, &["--simulate-missing", gone]);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("Rejected"), "{text}");
    assert!(text.contains(gone), "{text}");
    assert!(
        text.contains(&format!("{} files", NEEDED.len())),
        "current data changed: {text}"
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn update_needs_a_data_folder_and_refuses_one_inside_the_install() {
    let root = temp_root("check-update-bad");
    let steam = make(&root, &TreeOptions::default());
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(["update", "--steam-dir"])
        .arg(&steam)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let inside = steam.join("steamapps/common/MonsterHunterWilds/appdata");
    let (code, text) = run(&steam, &inside, &[]);
    assert_ne!(code, 0, "{text}");
    assert!(!inside.exists());
    std::fs::remove_dir_all(root).ok();
}
