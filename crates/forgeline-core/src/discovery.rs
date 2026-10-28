// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Finds the Steam folder, the library holding Wilds, the install, the build ID and the save
//! folder, with no configuration. The install is found through `libraryfolders.vdf`, never hardcoded.
//!
//! Read-only: this module lists folders and reads two small text files (the library list and the
//! app manifest). It never opens a save, a pack or any file inside the install, and never writes.

use crate::vdf;
use std::fmt;
use std::path::{Path, PathBuf};

/// Steam's app ID for Monster Hunter Wilds.
pub const APP_ID: &str = "2246340";

/// Added to a Steam account ID (the number in the `userdata` folder name) to give the 64-bit Steam ID.
const STEAM_ID64_BASE: u64 = 76_561_197_960_265_728;

/// Files and folders in the install that show the game has been modded. They are only reported as
/// present; none is ever opened.
const MOD_MARKERS: [&str; 3] = ["dinput8.dll", "reframework", "pak_mods"];

/// Environment variable that points discovery at a Steam folder (dev and test builds).
pub const STEAM_DIR_ENV: &str = "FORGELINE_STEAM_DIR";

#[derive(Debug, Clone)]
pub struct DiscoveryEnv {
    pub steam_root: PathBuf,
}

/// One save file in a `win64_save` folder. Only the name and size are recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFile {
    pub name: String,
    pub size: u64,
}

/// A Steam account folder that has Wilds saves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub account_id: u32,
    pub steam_id64: u64,
    pub save_dir: PathBuf,
    pub slot_files: Vec<SaveFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovered {
    pub steam_root: PathBuf,
    pub library: PathBuf,
    pub install: PathBuf,
    pub build_id: String,
    /// Every account folder with `2246340\remote\win64_save`. Normally one.
    pub accounts: Vec<Account>,
    /// Which of [`MOD_MARKERS`] exist in the install.
    pub mod_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryError {
    SteamNotFound,
    /// No library has the Wilds app manifest.
    GameNotInstalled,
    /// The manifest is there but unreadable or lacks an install folder or build ID.
    ManifestInvalid(String),
    /// The install folder named by the manifest does not exist.
    InstallMissing,
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::SteamNotFound => write!(f, "Steam was not found"),
            DiscoveryError::GameNotInstalled => {
                write!(
                    f,
                    "Monster Hunter Wilds is not installed in any Steam library"
                )
            }
            DiscoveryError::ManifestInvalid(why) => {
                write!(f, "the Wilds app manifest is unusable: {why}")
            }
            DiscoveryError::InstallMissing => write!(f, "the Wilds install folder is missing"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Which account the save belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveAccount<'a> {
    None,
    One(&'a Account),
    /// More than one account has Wilds saves; the window asks which.
    Several(&'a [Account]),
}

impl Discovered {
    pub fn save_account(&self) -> SaveAccount<'_> {
        match self.accounts.as_slice() {
            [] => SaveAccount::None,
            [one] => SaveAccount::One(one),
            many => SaveAccount::Several(many),
        }
    }

    /// Everything the app must never write to: the Steam folder, the library's `steamapps`, the
    /// install and every save folder. Pass to [`crate::datadir::DataDir::open_with_protected`].
    pub fn protected_paths(&self) -> Vec<PathBuf> {
        let mut v = vec![
            self.steam_root.clone(),
            self.library.join("steamapps"),
            self.install.clone(),
        ];
        v.extend(self.accounts.iter().map(|a| a.save_dir.clone()));
        v
    }
}

/// Finds the Steam folder: `FORGELINE_STEAM_DIR`, then the Windows registry, then the default location.
pub fn find_steam_root() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(STEAM_DIR_ENV) {
        return Some(PathBuf::from(dir));
    }
    system_steam_root()
}

#[cfg(windows)]
fn system_steam_root() -> Option<PathBuf> {
    registry_steam_path().or_else(|| {
        let default = PathBuf::from(r"C:\Program Files (x86)\Steam");
        default.is_dir().then_some(default)
    })
}

#[cfg(not(windows))]
fn system_steam_root() -> Option<PathBuf> {
    None
}

/// Reads `HKEY_CURRENT_USER\Software\Valve\Steam\SteamPath`.
#[cfg(windows)]
fn registry_steam_path() -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ};

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }
    let subkey = wide(r"Software\Valve\Steam");
    let value = wide("SteamPath");
    let mut bytes: u32 = 0;
    // SAFETY: first call only asks for the size; the buffers outlive both calls and `bytes`
    // tells the API how large the second buffer is.
    unsafe {
        let rc = RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut bytes,
        );
        if rc != 0 || bytes < 2 {
            return None;
        }
        let mut buf = vec![0u16; (bytes as usize).div_ceil(2)];
        let rc = RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buf.as_mut_ptr().cast(),
            &mut bytes,
        );
        if rc != 0 {
            return None;
        }
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let path = String::from_utf16_lossy(&buf[..len]);
        (!path.is_empty()).then(|| PathBuf::from(path.replace('/', "\\")))
    }
}

pub fn discover(env: &DiscoveryEnv) -> Result<Discovered, DiscoveryError> {
    let root = &env.steam_root;
    if !root.is_dir() {
        return Err(DiscoveryError::SteamNotFound);
    }

    // The library holding Wilds is the one with its app manifest, wherever libraryfolders.vdf puts it.
    let manifest_name = format!("appmanifest_{APP_ID}.acf");
    let (library, manifest_path) = library_paths(root)
        .into_iter()
        .map(|lib| {
            let manifest = lib.join("steamapps").join(&manifest_name);
            (lib, manifest)
        })
        .find(|(_, manifest)| manifest.is_file())
        .ok_or(DiscoveryError::GameNotInstalled)?;

    let text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| DiscoveryError::ManifestInvalid(format!("cannot read it ({})", e.kind())))?;
    let manifest = vdf::parse(&text).map_err(|e| DiscoveryError::ManifestInvalid(e.to_string()))?;
    let field = |key: &str| {
        manifest
            .path(&["AppState", key])
            .and_then(vdf::Value::as_str)
            .map(str::to_string)
    };
    let install_dir = field("installdir")
        .ok_or_else(|| DiscoveryError::ManifestInvalid("no installdir".into()))?;
    if install_dir.is_empty()
        || install_dir == ".."
        || install_dir == "."
        || install_dir.contains(['/', '\\'])
    {
        return Err(DiscoveryError::ManifestInvalid(
            "installdir is not a plain folder name".into(),
        ));
    }
    let build_id = field("buildid")
        .filter(|b| !b.is_empty() && b.bytes().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| DiscoveryError::ManifestInvalid("no numeric buildid".into()))?;

    let install = library.join("steamapps").join("common").join(install_dir);
    if !install.is_dir() {
        return Err(DiscoveryError::InstallMissing);
    }

    Ok(Discovered {
        steam_root: root.clone(),
        mod_markers: mod_markers(&install),
        accounts: accounts(root),
        library,
        install,
        build_id,
    })
}

/// The Steam folder itself plus every `path` in `libraryfolders.vdf`, without repeats.
fn library_paths(root: &Path) -> Vec<PathBuf> {
    let mut libs = vec![root.to_path_buf()];
    let file = root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(text) = std::fs::read_to_string(file) {
        if let Ok(v) = vdf::parse(&text) {
            let listed = v
                .get("libraryfolders")
                .map(vdf::Value::entries)
                .unwrap_or(&[]);
            for (_, entry) in listed {
                if let Some(p) = entry.get("path").and_then(vdf::Value::as_str) {
                    let p = PathBuf::from(p);
                    if !libs.contains(&p) {
                        libs.push(p);
                    }
                }
            }
        }
    }
    libs
}

fn mod_markers(install: &Path) -> Vec<String> {
    let Ok(rd) = std::fs::read_dir(install) else {
        return Vec::new();
    };
    let mut found: Vec<String> = rd
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(str::to_lowercase))
        .filter(|name| MOD_MARKERS.contains(&name.as_str()))
        .collect();
    found.sort();
    found
}

/// Account folders under `userdata` that have `2246340\remote\win64_save`.
fn accounts(root: &Path) -> Vec<Account> {
    let Ok(rd) = std::fs::read_dir(root.join("userdata")) else {
        return Vec::new();
    };
    let mut out: Vec<Account> = rd
        .flatten()
        .filter_map(|e| {
            let account_id: u32 = e.file_name().to_str()?.parse().ok()?;
            let save_dir = e.path().join(APP_ID).join("remote").join("win64_save");
            if !save_dir.is_dir() {
                return None;
            }
            Some(Account {
                account_id,
                steam_id64: STEAM_ID64_BASE + u64::from(account_id),
                slot_files: slot_files(&save_dir),
                save_dir,
            })
        })
        .collect();
    out.sort_by_key(|a| a.account_id);
    out
}

/// `data*.bin` files in the save folder, by name and size only.
fn slot_files(dir: &Path) -> Vec<SaveFile> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<SaveFile> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            let lower = name.to_lowercase();
            if !(lower.starts_with("data") && lower.ends_with(".bin")) {
                return None;
            }
            Some(SaveFile {
                name,
                size: e.metadata().ok()?.len(),
            })
        })
        .collect();
    files.sort_by(|a, b| a.name.cmp(&b.name));
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Tree(PathBuf);

    impl Tree {
        fn new(name: &str) -> Tree {
            let root =
                std::env::temp_dir().join(format!("forgeline-disc-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            Tree(root)
        }
        fn p(&self, rel: &str) -> PathBuf {
            self.0.join(rel)
        }
        fn write(&self, rel: &str, text: &str) {
            let p = self.p(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        fn dir(&self, rel: &str) {
            fs::create_dir_all(self.p(rel)).unwrap();
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn manifest(install: &str, build: &str) -> String {
        format!("\"AppState\"\n{{\n\t\"appid\"\t\"{APP_ID}\"\n\t\"installdir\"\t\"{install}\"\n\t\"buildid\"\t\"{build}\"\n}}\n")
    }

    /// Steam root `steam/` with a second library `lib2/` that holds the game, and two accounts of
    /// which only one has Wilds saves.
    fn two_library_tree(name: &str) -> Tree {
        let t = Tree::new(name);
        let lib2 = t.p("lib2");
        let vdf = format!(
            "\"libraryfolders\"\n{{\n\"0\" {{ \"path\" \"{}\" }}\n\"1\" {{ \"path\" \"{}\" \"apps\" {{ \"{APP_ID}\" \"1\" }} }}\n}}\n",
            t.p("steam").to_string_lossy().replace('\\', "\\\\"),
            lib2.to_string_lossy().replace('\\', "\\\\"),
        );
        t.write("steam/steamapps/libraryfolders.vdf", &vdf);
        t.write(
            &format!("lib2/steamapps/appmanifest_{APP_ID}.acf"),
            &manifest("MonsterHunterWilds", "25921695"),
        );
        t.dir("lib2/steamapps/common/MonsterHunterWilds");
        t.write(
            "steam/userdata/1000/2246340/remote/win64_save/data001Slot.bin",
            "abcd",
        );
        t.write(
            "steam/userdata/1000/2246340/remote/win64_save/notes.txt",
            "x",
        );
        t.dir("steam/userdata/2000/730/remote");
        t
    }

    fn env(t: &Tree) -> DiscoveryEnv {
        DiscoveryEnv {
            steam_root: t.p("steam"),
        }
    }

    #[test]
    fn finds_the_install_build_id_and_the_one_account_with_saves() {
        let t = two_library_tree("full");
        let d = discover(&env(&t)).unwrap();
        assert_eq!(d.library, t.p("lib2"));
        assert_eq!(d.install, t.p("lib2/steamapps/common/MonsterHunterWilds"));
        assert_eq!(d.build_id, "25921695");
        assert_eq!(d.accounts.len(), 1);
        let a = &d.accounts[0];
        assert_eq!(a.account_id, 1000);
        assert_eq!(a.steam_id64, 76_561_197_960_266_728);
        assert_eq!(
            a.slot_files,
            vec![SaveFile {
                name: "data001Slot.bin".into(),
                size: 4
            }]
        );
        assert!(matches!(d.save_account(), SaveAccount::One(_)));
    }

    #[test]
    fn mod_markers_are_listed_and_not_read() {
        let t = two_library_tree("mods");
        t.write(
            "lib2/steamapps/common/MonsterHunterWilds/dinput8.dll",
            "not a real dll",
        );
        t.dir("lib2/steamapps/common/MonsterHunterWilds/reframework");
        t.write("lib2/steamapps/common/MonsterHunterWilds/readme.txt", "x");
        let d = discover(&env(&t)).unwrap();
        assert_eq!(
            d.mod_markers,
            vec!["dinput8.dll".to_string(), "reframework".to_string()]
        );
    }

    #[test]
    fn several_accounts_with_saves_are_all_listed() {
        let t = two_library_tree("several");
        t.write(
            "steam/userdata/3000/2246340/remote/win64_save/data001Slot.bin",
            "zz",
        );
        let d = discover(&env(&t)).unwrap();
        assert_eq!(d.accounts.len(), 2);
        assert!(matches!(d.save_account(), SaveAccount::Several(a) if a.len() == 2));
    }

    #[test]
    fn game_in_the_steam_folder_itself_is_found_without_a_library_file() {
        let t = Tree::new("single");
        t.write(
            &format!("steam/steamapps/appmanifest_{APP_ID}.acf"),
            &manifest("MonsterHunterWilds", "1"),
        );
        t.dir("steam/steamapps/common/MonsterHunterWilds");
        let d = discover(&env(&t)).unwrap();
        assert_eq!(d.library, t.p("steam"));
        assert!(matches!(d.save_account(), SaveAccount::None));
    }

    #[test]
    fn missing_pieces_give_clear_errors() {
        let t = Tree::new("missing");
        assert_eq!(discover(&env(&t)), Err(DiscoveryError::SteamNotFound));

        t.dir("steam/steamapps");
        assert_eq!(discover(&env(&t)), Err(DiscoveryError::GameNotInstalled));

        t.write(
            &format!("steam/steamapps/appmanifest_{APP_ID}.acf"),
            &manifest("MonsterHunterWilds", "1"),
        );
        assert_eq!(discover(&env(&t)), Err(DiscoveryError::InstallMissing));

        t.write(
            &format!("steam/steamapps/appmanifest_{APP_ID}.acf"),
            "\"AppState\" { \"installdir\" \"x\" }",
        );
        assert!(matches!(
            discover(&env(&t)),
            Err(DiscoveryError::ManifestInvalid(_))
        ));

        t.write(
            &format!("steam/steamapps/appmanifest_{APP_ID}.acf"),
            "\"AppState\" {",
        );
        assert!(matches!(
            discover(&env(&t)),
            Err(DiscoveryError::ManifestInvalid(_))
        ));
    }

    #[test]
    fn an_install_dir_that_climbs_out_of_the_library_is_refused() {
        let t = Tree::new("climb");
        t.write(
            &format!("steam/steamapps/appmanifest_{APP_ID}.acf"),
            &manifest("..\\\\..", "1"),
        );
        assert!(matches!(
            discover(&env(&t)),
            Err(DiscoveryError::ManifestInvalid(_))
        ));
    }

    #[test]
    fn discovery_writes_nothing() {
        let t = two_library_tree("readonly");
        let snapshot = |root: &Path| {
            let mut v = Vec::new();
            let mut stack = vec![root.to_path_buf()];
            while let Some(d) = stack.pop() {
                for e in fs::read_dir(d).unwrap().flatten() {
                    let m = e.metadata().unwrap();
                    if m.is_dir() {
                        stack.push(e.path());
                    }
                    v.push((e.path(), m.len(), m.modified().unwrap()));
                }
            }
            v.sort();
            v
        };
        let before = snapshot(&t.0);
        discover(&env(&t)).unwrap();
        assert_eq!(snapshot(&t.0), before);
    }

    #[test]
    fn protected_paths_cover_steam_install_and_saves() {
        let t = two_library_tree("protected");
        let d = discover(&env(&t)).unwrap();
        let p = d.protected_paths();
        assert!(p.contains(&t.p("steam")));
        assert!(p.contains(&t.p("lib2/steamapps")));
        assert!(p.contains(&d.install));
        assert!(p.contains(&d.accounts[0].save_dir));
        let inside_install = d.install.join("forgeline-data");
        let r = crate::datadir::DataDir::open_with_protected(Some(inside_install), &p);
        assert!(r.is_err());
    }
}
