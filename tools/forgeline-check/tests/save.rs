// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs `forgeline-check save` over an invented Steam tree that holds a fixture save: it reports
//! counts, shows no hunter data, reads through a copy and leaves the "live" file untouched.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/saves")
        .join(name)
}

/// A Steam tree with the account the fixtures were built for (account 4242).
fn tree(tag: &str) -> (PathBuf, PathBuf) {
    let root =
        std::env::temp_dir().join(format!("forgeline-check-save-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let steam = root.join("steam");
    let app = steam.join("steamapps");
    fs::create_dir_all(app.join("common/MonsterHunterWilds")).unwrap();
    fs::write(
        app.join("appmanifest_2246340.acf"),
        "\"AppState\" { \"installdir\" \"MonsterHunterWilds\" \"buildid\" \"4242\" }",
    )
    .unwrap();
    let saves = steam.join("userdata/4242/2246340/remote/win64_save");
    fs::create_dir_all(&saves).unwrap();
    fs::copy(fixture("two-slots.bin"), saves.join("data001Slot.bin")).unwrap();
    (root, saves)
}

fn check(args: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn reads_a_copy_of_the_save_and_leaves_the_original_alone() {
    let (root, saves) = tree("auto");
    let live = saves.join("data001Slot.bin");
    let before = (
        fs::read(&live).unwrap(),
        fs::metadata(&live).unwrap().modified().unwrap(),
    );
    let work = root.join("work");
    let out = check(&[
        "save".as_ref(),
        "--steam-dir".as_ref(),
        root.join("steam").as_os_str(),
        "--work-dir".as_ref(),
        work.as_os_str(),
    ]);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("slots:        3"), "{text}");
    assert!(text.contains("active slot:  3"), "{text}");
    assert!(text.contains("slot 3: has data, ACTIVE"), "{text}");
    assert!(text.contains("slot 2: empty"), "{text}");
    assert!(text.contains("read time:"), "{text}");
    for never in ["Fixture", "FIXT-", "NOPE", "4242"] {
        assert!(!text.contains(never), "{never} leaked: {text}");
    }
    assert!(work.join("save-copy.bin").exists());
    assert_eq!(fs::read(&live).unwrap(), before.0);
    assert_eq!(fs::metadata(&live).unwrap().modified().unwrap(), before.1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_work_folder_inside_the_save_folder_is_refused() {
    let (root, saves) = tree("refuse");
    let out = check(&[
        "save".as_ref(),
        "--steam-dir".as_ref(),
        root.join("steam").as_os_str(),
        "--work-dir".as_ref(),
        saves.join("copies").as_os_str(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing"));
    assert!(!saves.join("copies").exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_given_copy_is_read_with_the_short_or_long_steam_id() {
    let copy = fixture("one-slot.bin");
    for id in ["4242", "76561197960269970"] {
        let out = check(&[
            "save".as_ref(),
            "--file".as_ref(),
            copy.as_os_str(),
            "--steam-id".as_ref(),
            id.as_ref(),
        ]);
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "{text}{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(text.contains("slot 1: has data, ACTIVE"), "{text}");
        assert!(
            text.contains("weapons: 4 (type 0 x1, type 3 x1, type 13 x2)"),
            "{text}"
        );
        assert!(text.contains("light bowguns: 2"), "{text}");
        assert!(text.contains("armour box entries: 4 (by piece number: 1 x1, 3 x3); A look 3, B look 1; counted once for stats: 3"), "{text}");
        assert!(
            text.contains("talismans: 1; mantles: 1; other equipment: 0"),
            "{text}"
        );
    }
}

#[test]
fn a_wrong_steam_id_gives_a_clear_message() {
    let copy = fixture("one-slot.bin");
    let out = check(&[
        "save".as_ref(),
        "--file".as_ref(),
        copy.as_os_str(),
        "--steam-id".as_ref(),
        "7".as_ref(),
    ]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Steam ID"));
}

#[test]
fn the_equipment_dump_lists_numbers_for_in_use_entries() {
    let copy = fixture("one-slot.bin");
    let out = check(&[
        "save".as_ref(),
        "--file".as_ref(),
        copy.as_os_str(),
        "--steam-id".as_ref(),
        "4242".as_ref(),
        "--equip-dump".as_ref(),
    ]);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(out.status.success(), "{text}");
    assert!(
        text.contains(
            "slot 1: first field over all 12 entries: 0 x3, 1 x1, 13 x4, 21 x1, 37 x1, 141 x2"
        ),
        "{text}"
    );
    assert!(text.contains("slot 1 entry 0: t4s1=13 t7s4=0"), "{text}");
    assert!(!text.contains("Fixture"), "{text}");
}

#[test]
fn the_equipment_layout_lists_the_fields_of_the_equipment_class() {
    let copy = fixture("one-slot.bin");
    let out = check(&[
        "save".as_ref(),
        "--file".as_ref(),
        copy.as_os_str(),
        "--steam-id".as_ref(),
        "4242".as_ref(),
        "--equip-layout".as_ref(),
    ]);
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(out.status.success(), "{text}");
    assert!(text.contains("_EquipBox: array of 12 x app.savedata.cEquipWork; first field: 0 x3, 1 x1, 13 x4, 21 x1, 37 x1, 141 x2"), "{text}");
}
