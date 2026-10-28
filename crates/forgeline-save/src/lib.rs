// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reads a Monster Hunter Wilds save, read-only: open the container with the Steam ID
//! ([`container`]), walk the class stream ([`stream`]) and pull out a summary ([`model`]).
//!
//! Everything works on bytes handed in by the caller. Nothing here touches the disk, and a
//! corrupt or wrong-key input gives a [`SaveError`], never a panic.

pub mod container;
pub mod flatten;
pub mod hash;
pub mod hashes;
pub mod model;
pub mod names;
pub mod stream;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// The file is too short or does not start with the expected header.
    NotASave(&'static str),
    /// The Steam ID did not produce the expected check value, so the file is for another account.
    WrongSteamId,
    /// The file ended early, a checksum or length did not match, or inflating failed.
    Corrupt(&'static str),
    /// The class stream broke a rule at this byte offset.
    Stream { offset: usize, why: &'static str },
    /// The stream is fine but does not have the classes we need.
    Missing(&'static str),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::NotASave(w) => write!(f, "not a Wilds save: {w}"),
            SaveError::WrongSteamId => {
                write!(f, "the Steam ID does not match this save (wrong account?)")
            }
            SaveError::Corrupt(w) => write!(f, "damaged save: {w}"),
            SaveError::Stream { offset, why } => {
                write!(f, "unexpected save layout at byte {offset}: {why}")
            }
            SaveError::Missing(w) => write!(f, "save has no {w}"),
        }
    }
}

impl std::error::Error for SaveError {}
