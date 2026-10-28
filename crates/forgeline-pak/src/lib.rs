// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reads the game's own pack files, read-only. [`index`] lists every entry of the original
//! `re_*.pak` files (never `pak_mods`, never anything else), [`read`] pulls one entry out of
//! a pack, [`paths`] turns a path into the two hashes packs are keyed by and holds the list of
//! game files we need, and [`exescan`] rebuilds class and field names from the game's `.exe`.
//!
//! Nothing here writes to disk, and a damaged or unexpected file gives a [`PakError`], never a
//! panic. Format facts and the table key come from eigeen/ree-pak-rs (MIT); see
//! `THIRD-PARTY-NOTICES.md`.

pub mod exescan;
pub mod index;
pub mod paths;
pub mod read;
#[cfg(feature = "test-support")]
pub mod testpack;

use std::fmt;

#[derive(Debug)]
pub enum PakError {
    /// A file could not be opened or read.
    Io(String, std::io::Error),
    /// The file is not a pack this reader understands (named with the reason).
    Unsupported(String),
    /// A table, chunk list or entry broke a rule.
    Corrupt(String),
}

impl fmt::Display for PakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PakError::Io(what, e) => write!(f, "{what}: {e}"),
            PakError::Unsupported(w) => write!(f, "unsupported pack: {w}"),
            PakError::Corrupt(w) => write!(f, "damaged pack: {w}"),
        }
    }
}

impl std::error::Error for PakError {}
