// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs save-diff over the committed fixtures: it lists differences by field name and number,
//! never shows hunter data, and refuses the live save folder.

use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/saves")
        .join(name)
}

fn run(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_save-diff"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn two_different_saves_list_named_differences_without_hunter_data() {
    let a = fixture("one-slot.bin");
    let b = fixture("two-slots.bin");
    let out = run(&[
        a.as_os_str(),
        b.as_os_str(),
        "--steam-id".as_ref(),
        "4242".as_ref(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("differences:"), "{text}");
    assert!(
        text.contains("app.savedata.cUserSaveData._Data[0].Active: 1 -> 0"),
        "{text}"
    );
    assert!(text.contains("3 identity field(s) left out"), "{text}");
    for never in ["Fixture", "FIXT-", "NOPE"] {
        assert!(!text.contains(never), "{never} leaked: {text}");
    }
}

#[test]
fn the_same_save_twice_has_no_differences() {
    let a = fixture("two-slots.bin");
    let out = run(&[
        a.as_os_str(),
        a.as_os_str(),
        "--steam-id".as_ref(),
        "4242".as_ref(),
    ]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("differences: 0"), "{text}");
}

#[test]
fn a_file_inside_a_save_folder_is_refused() {
    let dir = std::env::temp_dir().join(format!("save-diff-{}/win64_save", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let copy = dir.join("data001Slot.bin");
    std::fs::copy(fixture("one-slot.bin"), &copy).unwrap();
    let a = fixture("one-slot.bin");
    let out = run(&[
        copy.as_os_str(),
        a.as_os_str(),
        "--steam-id".as_ref(),
        "4242".as_ref(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing"));
    std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
}

#[test]
fn a_wrong_steam_id_is_reported() {
    let a = fixture("one-slot.bin");
    let out = run(&[
        a.as_os_str(),
        a.as_os_str(),
        "--steam-id".as_ref(),
        "1".as_ref(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Steam ID"));
}
