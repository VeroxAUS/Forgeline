// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Runs the report over an invented Steam tree: it names what it found, masks the Steam ID, and
//! changes nothing in the tree.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn tree() -> PathBuf {
    let root = std::env::temp_dir().join(format!("forgeline-check-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let steam = root.join("steam");
    let app = steam.join("steamapps");
    fs::create_dir_all(app.join("common/MonsterHunterWilds")).unwrap();
    fs::write(
        app.join("appmanifest_2246340.acf"),
        "\"AppState\" { \"installdir\" \"MonsterHunterWilds\" \"buildid\" \"4242\" }",
    )
    .unwrap();
    fs::write(app.join("common/MonsterHunterWilds/dinput8.dll"), "x").unwrap();
    let saves = steam.join("userdata/123456789/2246340/remote/win64_save");
    fs::create_dir_all(&saves).unwrap();
    fs::write(saves.join("data001Slot.bin"), "12345").unwrap();
    root
}

#[test]
fn discover_reports_names_counts_and_a_masked_steam_id() {
    let root = tree();
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(["discover", "--steam-dir"])
        .arg(root.join("steam"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    assert!(text.contains("build id:     4242"), "{text}");
    assert!(
        text.contains("1 Steam account folder with Wilds saves"),
        "{text}"
    );
    assert!(text.contains("data001Slot.bin (5 bytes)"), "{text}");
    assert!(text.contains("dinput8.dll"), "{text}");
    assert!(text.contains("none of them was opened"), "{text}");
    // The account number and the full 64-bit Steam ID must not appear.
    assert!(!text.contains("123456789"), "{text}");
    assert!(!text.contains("76561198"), "{text}");
    assert!(text.contains("ending"), "{text}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn discover_fails_cleanly_when_the_game_is_missing() {
    let root = std::env::temp_dir().join(format!("forgeline-check-none-{}", std::process::id()));
    fs::create_dir_all(root.join("steam/steamapps")).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(["discover", "--steam-dir"])
        .arg(root.join("steam"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not installed"));
    let _ = fs::remove_dir_all(root);
}
