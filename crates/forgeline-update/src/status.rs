// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `update-status.json`: what the update is doing now and how the last one ended. The `--update`
//! process writes it; the window reads it to show progress. Written through a temporary file and
//! a rename so a reader never sees half a file.

use crate::validate::Finding;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SCHEMA: u32 = 1;

/// How an update ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Replaced,
    Unchanged,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastOutcome {
    pub kind: Kind,
    /// UTC time the update ended.
    pub at: String,
    pub build_id: Option<String>,
    pub findings: Vec<Finding>,
    /// The error text for `Failed`.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub schema: u32,
    /// True while an update is running.
    pub running: bool,
    /// What it is doing, for example "reading the packs".
    pub phase: String,
    pub done: u32,
    pub total: u32,
    pub last: Option<LastOutcome>,
}

impl UpdateStatus {
    pub fn idle(last: Option<LastOutcome>) -> UpdateStatus {
        UpdateStatus {
            schema: SCHEMA,
            running: false,
            phase: String::new(),
            done: 0,
            total: 0,
            last,
        }
    }

    pub fn from_json(text: &str) -> Option<UpdateStatus> {
        let s: UpdateStatus = serde_json::from_str(text).ok()?;
        (s.schema == SCHEMA).then_some(s)
    }

    /// Reads the file; `None` if it is missing, damaged or from an unknown schema.
    pub fn read(path: &Path) -> Option<UpdateStatus> {
        UpdateStatus::from_json(&std::fs::read_to_string(path).ok()?)
    }

    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        let tmp = path.with_extension("json.tmp");
        std::fs::write(
            &tmp,
            serde_json::to_string_pretty(self).expect("a status always serialises"),
        )?;
        std::fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::Severity;

    #[test]
    fn writes_and_reads_back_and_ignores_damage() {
        let dir = std::env::temp_dir().join(format!("forgeline-status-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("update-status.json");
        let s = UpdateStatus {
            schema: SCHEMA,
            running: true,
            phase: "reading the packs".into(),
            done: 3,
            total: 34,
            last: Some(LastOutcome {
                kind: Kind::Rejected,
                at: "2026-01-01T00:00:00Z".into(),
                build_id: Some("100".into()),
                findings: vec![Finding {
                    severity: Severity::Block,
                    message: "x".into(),
                }],
                error: None,
            }),
        };
        s.write(&path).unwrap();
        assert_eq!(UpdateStatus::read(&path), Some(s));
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::write(&path, "{ damaged").unwrap();
        assert_eq!(UpdateStatus::read(&path), None);
        assert_eq!(UpdateStatus::read(&dir.join("missing.json")), None);
        assert_eq!(
            UpdateStatus::from_json(&"{\"schema\":7}".replace("7", "9")),
            None
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
