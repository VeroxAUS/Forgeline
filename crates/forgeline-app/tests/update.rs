// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs `forgeline --update` (the child process the window starts) over an invented Steam tree:
//! Replaced, then Unchanged, then Rejected for a broken game folder, with the exit codes the window
//! will read.

use forgeline_pak::paths::NEEDED;
use forgeline_update::testtree::{make, temp_root, TreeOptions};
use std::path::Path;
use std::process::Command;

fn run(steam: &Path, data: &Path) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline"))
        .arg("--update")
        .arg("--steam-dir")
        .arg(steam)
        .arg("--data-dir")
        .arg(data)
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
fn update_mode_replaces_then_says_unchanged_then_rejects_a_broken_game_folder() {
    let root = temp_root("app-update");
    let data = root.join("appdata");
    let steam = make(&root, &TreeOptions::default());

    let (code, text) = run(&steam, &data);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("Replaced (game build 100)"), "{text}");
    assert!(text.contains("extracting needed files"), "{text}");
    assert!(data.join("data/current/manifest.json").is_file());
    assert!(data.join("data/update-status.json").is_file());

    let (code, text) = run(&steam, &data);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("Unchanged (game build 100)"), "{text}");

    let omit = NEEDED[0].stem.to_string();
    make(
        &root,
        &TreeOptions {
            omit: vec![omit.clone()],
            tag: "b".into(),
            ..TreeOptions::default()
        },
    );
    let (code, text) = run(&steam, &data);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("Rejected"), "{text}");
    assert!(text.contains(&omit), "{text}");
    // The last good data is still there and still build 100.
    let manifest = std::fs::read_to_string(data.join("data/current/manifest.json")).unwrap();
    assert!(manifest.contains("\"build_id\": \"100\""), "{manifest}");
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn update_mode_refuses_a_data_folder_inside_the_install_and_bad_flags() {
    let root = temp_root("app-update-bad");
    let steam = make(&root, &TreeOptions::default());
    let inside = steam.join("steamapps/common/MonsterHunterWilds/appdata");
    let (code, text) = run(&steam, &inside);
    assert_eq!(code, 1, "{text}");
    assert!(!inside.exists(), "something was written inside the install");

    let out = Command::new(env!("CARGO_BIN_EXE_forgeline"))
        .args(["--update", "--bogus"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    std::fs::remove_dir_all(root).ok();
}
