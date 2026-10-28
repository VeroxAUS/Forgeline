// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The app's own data directory (`docs/plans/phase-2-foundation.md`, section 3.3).
//!
//! The directory is chosen once. A dev or test build always passes an override, so it never
//! touches the maintainer's installed copy or its cache. Nothing here creates a directory or
//! reads a file: step 1 only decides where things go and refuses places that must never be written.

use std::fmt;
use std::path::{Component, Path, PathBuf};

/// Environment variable that overrides the default directory (the `--data-dir` flag wins over it).
pub const ENV_VAR: &str = "FORGELINE_DATA_DIR";

/// Folder names that mark Steam, the game install or the game's save: the app never writes there.
const PROTECTED_NAMES: [&str; 6] = [
    "steam",
    "steamapps",
    "userdata",
    "win64_save",
    "monsterhunterwilds",
    "2246340",
];

#[derive(Debug, PartialEq, Eq)]
pub enum DataDirError {
    /// The path is inside Steam, the game install or the game's save folder.
    Protected(PathBuf),
    /// No override was given and no default location can be found.
    NoDefault,
    /// The path is not absolute.
    Relative(PathBuf),
}

impl fmt::Display for DataDirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataDirError::Protected(p) => write!(
                f,
                "{} is inside Steam, the game install or the save folder; Forgeline never writes there",
                p.display()
            ),
            DataDirError::NoDefault => write!(f, "no default data directory; pass --data-dir"),
            DataDirError::Relative(p) => write!(f, "{} is not an absolute path", p.display()),
        }
    }
}

impl std::error::Error for DataDirError {}

/// The resolved data directory and the fixed names inside it.
#[derive(Debug, PartialEq, Eq)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// Resolves the directory: `override_dir`, else `FORGELINE_DATA_DIR`, else the per-user default.
    pub fn open(override_dir: Option<PathBuf>) -> Result<DataDir, DataDirError> {
        Self::open_with_protected(override_dir, &[])
    }

    /// Like [`DataDir::open`], also refusing anything inside `protected` (the Steam library, the
    /// game install and the save folder as found by discovery in step 2).
    pub fn open_with_protected(
        override_dir: Option<PathBuf>,
        protected: &[PathBuf],
    ) -> Result<DataDir, DataDirError> {
        let env = std::env::var_os(ENV_VAR).map(PathBuf::from);
        Self::resolve(override_dir, env, default_dir(), protected)
    }

    fn resolve(
        override_dir: Option<PathBuf>,
        env_dir: Option<PathBuf>,
        default: Option<PathBuf>,
        protected: &[PathBuf],
    ) -> Result<DataDir, DataDirError> {
        let root = override_dir
            .or(env_dir)
            .or(default)
            .ok_or(DataDirError::NoDefault)?;
        if !root.is_absolute() {
            return Err(DataDirError::Relative(root));
        }
        if is_protected(&root, protected) {
            return Err(DataDirError::Protected(root));
        }
        Ok(DataDir { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
    /// Last good extraction.
    pub fn current(&self) -> PathBuf {
        self.root.join("data").join("current")
    }
    /// A new extraction while it runs.
    pub fn staging(&self) -> PathBuf {
        self.root.join("data").join("staging")
    }
    /// The previous good extraction, kept until the next successful update.
    pub fn previous(&self) -> PathBuf {
        self.root.join("data").join("previous")
    }
    pub fn update_status(&self) -> PathBuf {
        self.root.join("data").join("update-status.json")
    }
    pub fn view_cache(&self) -> PathBuf {
        self.root.join("view-cache.json")
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }
}

fn default_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Forgeline"))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .map(|p| p.join("forgeline"))
    }
}

/// True if `path` is inside one of `protected` or has a Steam, install or save folder name in it.
pub fn is_protected(path: &Path, protected: &[PathBuf]) -> bool {
    if protected.iter().any(|p| path.starts_with(p)) {
        return true;
    }
    path.components().any(|c| match c {
        Component::Normal(name) => {
            let lower = name.to_string_lossy().to_lowercase();
            PROTECTED_NAMES.contains(&lower.as_str())
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn abs(p: &str) -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(format!("C:\\{p}"))
        } else {
            PathBuf::from(format!("/{p}"))
        }
    }

    #[test]
    fn override_wins_over_env_and_default() {
        let d = DataDir::resolve(
            Some(abs("dev")),
            Some(abs("env")),
            Some(abs("default")),
            &[],
        )
        .unwrap();
        assert_eq!(d.root(), abs("dev"));
    }

    #[test]
    fn env_wins_over_default() {
        let d = DataDir::resolve(None, Some(abs("env")), Some(abs("default")), &[]).unwrap();
        assert_eq!(d.root(), abs("env"));
    }

    #[test]
    fn default_is_used_last_and_missing_default_is_an_error() {
        let d = DataDir::resolve(None, None, Some(abs("default")), &[]).unwrap();
        assert_eq!(d.root(), abs("default"));
        assert_eq!(
            DataDir::resolve(None, None, None, &[]),
            Err(DataDirError::NoDefault)
        );
    }

    #[test]
    fn relative_paths_are_refused() {
        let r = DataDir::resolve(Some(PathBuf::from("data")), None, None, &[]);
        assert!(matches!(r, Err(DataDirError::Relative(_))));
    }

    #[test]
    fn paths_inside_a_fake_steam_folder_are_refused() {
        for bad in [
            "Program Files/Steam/forgeline",
            "games/SteamLibrary/steamapps/common/MonsterHunterWilds/data",
            "Steam/userdata/123/2246340/remote/win64_save/x",
        ] {
            let r = DataDir::resolve(Some(abs(bad)), None, None, &[]);
            assert!(matches!(r, Err(DataDirError::Protected(_))), "{bad}");
        }
    }

    #[test]
    fn paths_inside_a_discovered_install_are_refused() {
        let install = abs("somewhere/game");
        let r = DataDir::resolve(
            Some(install.join("forgeline-data")),
            None,
            None,
            std::slice::from_ref(&install),
        );
        assert!(matches!(r, Err(DataDirError::Protected(_))));
    }

    #[test]
    fn layout_names_match_the_plan() {
        let d = DataDir::resolve(Some(abs("fl")), None, None, &[]).unwrap();
        assert!(d.current().ends_with("data/current") || d.current().ends_with("data\\current"));
        assert!(d.view_cache().ends_with("view-cache.json"));
        assert!(d.settings().ends_with("settings.json"));
        assert!(d.update_status().ends_with("update-status.json"));
    }
}
