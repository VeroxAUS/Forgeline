// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builds an invented Steam tree for tests: an app manifest, an install with a tiny synthetic
//! pack that holds every needed file and a fake exe, and the account folder with a fixture save.
//! Behind the `test-support` feature; never part of the app. Every byte is made up here.

use forgeline_pak::paths::NEEDED;
use forgeline_pak::testpack::{build, plain, Data, Item};
use std::path::{Path, PathBuf};

/// The Steam ID the fixture saves were built for (account 4242).
pub const FIXTURE_STEAM_ID: u64 = 76_561_197_960_269_970;

#[derive(Debug, Clone)]
pub struct TreeOptions {
    pub build_id: String,
    /// Needed files left out of the pack.
    pub omit: Vec<String>,
    /// Mixed into every file's bytes, so a different tag means different data.
    pub tag: String,
    /// Short class names the fake exe holds (without the namespace).
    pub exe_classes: Vec<String>,
    /// Whether the files start like the real container (`USR`).
    pub valid_containers: bool,
}

impl Default for TreeOptions {
    fn default() -> TreeOptions {
        TreeOptions {
            build_id: "100".into(),
            omit: vec![],
            tag: "a".into(),
            exe_classes: [
                "cUserSaveData",
                "cUserSaveParam",
                "cEquipParam",
                "cEquipWork",
            ]
            .map(String::from)
            .to_vec(),
            valid_containers: true,
        }
    }
}

fn file_bytes(stem: &str, o: &TreeOptions) -> Vec<u8> {
    let magic: &[u8] = if o.valid_containers {
        b"USR\0"
    } else {
        b"BAD\0"
    };
    let mut b = magic.to_vec();
    b.extend_from_slice(format!("{} {stem} invented data", o.tag).as_bytes());
    b
}

/// (Re)writes the tree under `root` and returns the Steam folder. Safe to call again with other
/// options to simulate a game update.
pub fn make(root: &Path, o: &TreeOptions) -> PathBuf {
    let steam = root.join("steam");
    let app = steam.join("steamapps");
    let install = app.join("common/MonsterHunterWilds");
    std::fs::create_dir_all(install.join("pak_mods")).unwrap();
    std::fs::write(
        app.join("appmanifest_2246340.acf"),
        format!(
            "\"AppState\" {{ \"installdir\" \"MonsterHunterWilds\" \"buildid\" \"{}\" }}",
            o.build_id
        ),
    )
    .unwrap();
    let items = NEEDED
        .iter()
        .filter(|n| !o.omit.iter().any(|m| m == n.stem))
        .map(|n| Item {
            path: format!("natives/stm/{}.7", n.stem),
            data: Data::Stored(file_bytes(n.stem, o)),
        })
        .collect();
    std::fs::write(install.join("re_chunk_000.pak"), build(&plain(items))).unwrap();
    // Opening this would fail: mods are never read.
    std::fs::write(install.join("pak_mods").join("re_chunk_999.pak"), b"junk").unwrap();

    let mut exe = Vec::new();
    for s in ["app.savedata", "_Data", "Active", "_EquipBox"]
        .iter()
        .map(|s| s.to_string())
        .chain(o.exe_classes.iter().cloned())
    {
        exe.extend_from_slice(s.as_bytes());
        exe.extend_from_slice(&[0, 0]);
    }
    std::fs::write(install.join("MonsterHunterWilds.exe"), exe).unwrap();

    let saves = steam.join("userdata/4242/2246340/remote/win64_save");
    std::fs::create_dir_all(&saves).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves/two-slots.bin");
    std::fs::copy(fixture, saves.join("data001Slot.bin")).unwrap();
    steam
}

/// A fresh temporary folder for one test.
pub fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("forgeline-update-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}
