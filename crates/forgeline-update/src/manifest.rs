// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The record of an extraction: which game build it came from, which files were taken (with size
//! and a hash for change detection), which needed files were not found and which class names were
//! rebuilt. Written next to the extracted files as `manifest.json`.

use serde::{Deserialize, Serialize};

/// Bumped when the layout of `manifest.json` changes. An unknown number is treated as "no last
/// good data to compare with", never as a crash.
pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    /// Path below `natives/stm/`, without the version suffix (as in the needed-files list).
    pub path: String,
    /// The file-version suffix that matched in the packs.
    pub version: u32,
    pub category: String,
    pub size: u64,
    /// FNV-1a 64 of the bytes, in hex. Only for noticing changes, not a security check.
    pub hash: String,
    /// True if the file starts like the container it should be (see `validate`).
    pub opens: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub build_id: String,
    pub exe_size: u64,
    /// Seconds since 1970, if the system said.
    pub exe_modified: Option<u64>,
    pub tool_version: String,
    /// When the extraction was made (UTC).
    pub created: String,
    pub files: Vec<FileRecord>,
    /// Needed files that were not found in the packs.
    pub missing: Vec<String>,
    /// Full class names rebuilt from the exe, sorted.
    pub classes: Vec<String>,
}

impl Manifest {
    /// True if both extractions hold the same data: same build, files, missing list and classes.
    /// The time, the tool version and the exe's modified time do not count, so an update that
    /// finds nothing new says so.
    pub fn same_data_as(&self, other: &Manifest) -> bool {
        self.build_id == other.build_id
            && self.files == other.files
            && self.missing == other.missing
            && self.classes == other.classes
    }

    /// Files per category.
    pub fn category_counts(&self) -> std::collections::BTreeMap<&str, usize> {
        let mut m = std::collections::BTreeMap::new();
        for f in &self.files {
            *m.entry(f.category.as_str()).or_insert(0) += 1;
        }
        m
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("a manifest always serialises")
    }

    /// Parses a manifest; `None` for damaged text or an unknown schema number.
    pub fn from_json(text: &str) -> Option<Manifest> {
        let m: Manifest = serde_json::from_str(text).ok()?;
        (m.schema == SCHEMA).then_some(m)
    }

    /// True if the text is JSON with a schema number that is not ours: a manifest from another
    /// version of the app, which is not the same as a damaged one.
    pub fn is_other_schema(text: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(text)
            .ok()
            .and_then(|v| v["schema"].as_u64())
            .is_some_and(|n| n != u64::from(SCHEMA))
    }
}

/// FNV-1a, 64 bit.
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in data {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn sample() -> Manifest {
        Manifest {
            schema: SCHEMA,
            build_id: "100".into(),
            exe_size: 10,
            exe_modified: Some(5),
            tool_version: "0.2.0".into(),
            created: "2026-01-01T00:00:00Z".into(),
            files: vec![
                FileRecord {
                    path: "A/One.user".into(),
                    version: 3,
                    category: "skills".into(),
                    size: 4,
                    hash: "00".into(),
                    opens: true,
                },
                FileRecord {
                    path: "A/Two.user".into(),
                    version: 3,
                    category: "skills".into(),
                    size: 4,
                    hash: "01".into(),
                    opens: true,
                },
            ],
            missing: vec![],
            classes: vec!["app.savedata.cA".into()],
        }
    }

    #[test]
    fn fnv_matches_published_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn json_round_trips_and_unknown_schema_is_ignored() {
        let m = sample();
        assert_eq!(Manifest::from_json(&m.to_json()), Some(m.clone()));
        let newer = m.to_json().replace("\"schema\": 1", "\"schema\": 99");
        assert_eq!(Manifest::from_json(&newer), None);
        assert_eq!(Manifest::from_json("{ not json"), None);
        assert_eq!(Manifest::from_json(""), None);
    }

    #[test]
    fn same_data_ignores_time_tool_version_and_exe_time() {
        let a = sample();
        let mut b = sample();
        b.created = "2027-01-01T00:00:00Z".into();
        b.tool_version = "0.3.0".into();
        b.exe_modified = Some(99);
        assert!(a.same_data_as(&b));
        b.files[0].hash = "ff".into();
        assert!(!a.same_data_as(&b));
        let mut c = sample();
        c.classes.push("app.savedata.cB".into());
        assert!(!a.same_data_as(&c));
    }

    #[test]
    fn counts_files_per_category() {
        let m = sample();
        let counts = m.category_counts();
        assert_eq!(counts["skills"], 2);
    }
}
