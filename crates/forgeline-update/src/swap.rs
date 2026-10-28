// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Moves folders inside the app's own data directory: staging becomes current, the old current is
//! kept as previous. Nothing here touches anything outside the data directory.
//!
//! A swap is two renames, so a crash between them leaves no `current` but a `previous`;
//! [`recover`] puts that back before the next update or read.

use forgeline_core::datadir::DataDir;
use std::io;

/// If `current` is missing and `previous` exists (a swap was cut short), moves `previous` back.
/// Returns true if it did.
pub fn recover(data: &DataDir) -> io::Result<bool> {
    let (current, previous) = (data.current(), data.previous());
    if !current.exists() && previous.exists() {
        std::fs::rename(&previous, &current)?;
        return Ok(true);
    }
    Ok(false)
}

/// Removes a half-made or rejected staging folder, if there is one.
pub fn clear_staging(data: &DataDir) -> io::Result<()> {
    match std::fs::remove_dir_all(data.staging()) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// Makes the staged data current and keeps the old current as previous (the one before that is
/// deleted). If the second rename fails the old current is put back.
pub fn commit(data: &DataDir) -> io::Result<()> {
    let (current, staging, previous) = (data.current(), data.staging(), data.previous());
    if !staging.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "there is no staged data to make current",
        ));
    }
    match std::fs::remove_dir_all(&previous) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
        _ => {}
    }
    let had_current = current.exists();
    if had_current {
        std::fs::rename(&current, &previous)?;
    }
    if let Err(e) = std::fs::rename(&staging, &current) {
        if had_current {
            let _ = std::fs::rename(&previous, &current);
        }
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn dir(tag: &str) -> (PathBuf, DataDir) {
        let root =
            std::env::temp_dir().join(format!("forgeline-swap-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("data")).unwrap();
        let d = DataDir::open(Some(root.clone())).unwrap();
        (root, d)
    }

    fn put(dir: &Path, text: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("file.txt"), text).unwrap();
    }

    fn read(dir: &Path) -> String {
        fs::read_to_string(dir.join("file.txt")).unwrap()
    }

    #[test]
    fn commit_keeps_the_old_current_as_previous_and_drops_the_one_before() {
        let (root, d) = dir("commit");
        put(&d.current(), "one");
        put(&d.previous(), "zero");
        put(&d.staging(), "two");
        commit(&d).unwrap();
        assert_eq!(read(&d.current()), "two");
        assert_eq!(read(&d.previous()), "one");
        assert!(!d.staging().exists());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn the_first_commit_has_no_previous() {
        let (root, d) = dir("first");
        put(&d.staging(), "one");
        commit(&d).unwrap();
        assert_eq!(read(&d.current()), "one");
        assert!(!d.previous().exists());
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn commit_without_staging_changes_nothing() {
        let (root, d) = dir("nostage");
        put(&d.current(), "one");
        put(&d.previous(), "zero");
        assert!(commit(&d).is_err());
        assert_eq!(read(&d.current()), "one");
        assert_eq!(read(&d.previous()), "zero");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn recover_restores_current_after_a_cut_short_swap() {
        let (root, d) = dir("recover");
        put(&d.previous(), "one");
        assert!(recover(&d).unwrap());
        assert_eq!(read(&d.current()), "one");
        assert!(!recover(&d).unwrap());
        // With a current present, previous stays where it is.
        put(&d.previous(), "old");
        assert!(!recover(&d).unwrap());
        assert_eq!(read(&d.current()), "one");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn clear_staging_is_fine_when_there_is_none() {
        let (root, d) = dir("clear");
        clear_staging(&d).unwrap();
        put(&d.staging(), "x");
        clear_staging(&d).unwrap();
        assert!(!d.staging().exists());
        fs::remove_dir_all(root).ok();
    }
}
