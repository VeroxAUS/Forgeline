// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The view cache: the derived summary on disk, so the window can show the last view at once.
//!
//! It holds counts, the slot table and the build IDs only: never the raw save, and no hunter name
//! or ID. A missing, damaged or old-schema file is ignored, and the view is rebuilt.

use crate::snapshot::Snapshot;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Bumped when the layout of `Snapshot` changes.
pub const SCHEMA: u32 = 1;

#[derive(Serialize, Deserialize)]
struct CacheFile {
    schema: u32,
    snapshot: Snapshot,
}

/// Reads the cache. The result is marked as refreshing until the engine has checked it against
/// the save as it is now.
pub fn load(path: &Path) -> Option<Snapshot> {
    let text = std::fs::read_to_string(path).ok()?;
    let file: CacheFile = serde_json::from_str(&text).ok()?;
    (file.schema == SCHEMA).then(|| {
        let mut s = file.snapshot;
        s.refreshing = true;
        s
    })
}

/// Writes the cache through a temporary file. Failure is ignored: the cache is a convenience.
pub fn store(path: &Path, snapshot: &Snapshot) {
    let file = CacheFile {
        schema: SCHEMA,
        snapshot: snapshot.clone(),
    };
    let Ok(text) = serde_json::to_string(&file) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("json.tmp");
    if std::fs::write(&tmp, text).is_ok() && std::fs::rename(&tmp, path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("forgeline-cache-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("view-cache.json")
    }

    #[test]
    fn round_trips_and_marks_the_loaded_view_as_refreshing() {
        let p = path("round");
        let mut s = Snapshot::loading();
        s.refreshing = false;
        s.save_error = Some("x".into());
        store(&p, &s);
        let back = load(&p).unwrap();
        assert!(back.refreshing);
        assert_eq!(back.save_error, s.save_error);
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }

    #[test]
    fn a_missing_corrupt_or_old_schema_file_is_ignored() {
        let p = path("bad");
        assert!(load(&p).is_none());
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "{ not json").unwrap();
        assert!(load(&p).is_none());
        let mut s = Snapshot::loading();
        s.refreshing = false;
        store(&p, &s);
        let text =
            std::fs::read_to_string(&p)
                .unwrap()
                .replacen("\"schema\":1", "\"schema\":99", 1);
        std::fs::write(&p, text).unwrap();
        assert!(load(&p).is_none());
        let _ = std::fs::remove_dir_all(p.parent().unwrap());
    }
}
