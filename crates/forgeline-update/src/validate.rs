// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The structural half of the validation gate. It looks at two manifests (the new extraction and
//! the last good one) and returns findings. A `Block` finding stops the swap. The scoring half
//! (builds that get worse, false upgrades) needs the optimizer and arrives in Phase 4, so until
//! then the app says "structure checked, builds not re-scored".

use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Stops the swap; the last good data stays.
    Block,
    /// Shown to the user; the swap goes ahead.
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
}

impl Finding {
    fn block(message: String) -> Finding {
        Finding {
            severity: Severity::Block,
            message,
        }
    }

    fn warn(message: String) -> Finding {
        Finding {
            severity: Severity::Warn,
            message,
        }
    }
}

/// A category that grows by more than this factor is worth a look (not a block).
const LARGE_GROWTH: usize = 2;

/// Checks a new extraction against the last good one (`None` on the first update).
///
/// Blocks when: a needed file was not found in the packs; a file does not open as the container it
/// should be; the build is older than the last good one; a file of the last good data is not in the
/// new one; a category has fewer files; or the type-name table lost a class. Warns on a new build
/// and on a category that more than doubled.
pub fn validate(staged: &Manifest, last_good: Option<&Manifest>) -> Vec<Finding> {
    let mut out = Vec::new();

    for m in &staged.missing {
        out.push(Finding::block(format!(
            "needed file {m} was not found in the game's packs"
        )));
    }
    for f in staged.files.iter().filter(|f| !f.opens) {
        out.push(Finding::block(format!(
            "file {} does not open as the expected container",
            f.path
        )));
    }
    if staged.classes.is_empty() {
        out.push(Finding::block(
            "no class names were rebuilt from the game exe".to_string(),
        ));
    }

    let Some(old) = last_good else {
        return out;
    };

    for f in &old.files {
        if !staged.files.iter().any(|n| n.path == f.path) {
            out.push(Finding::block(format!(
                "file {} was in the last good data and is missing now",
                f.path
            )));
        }
    }
    let (old_counts, new_counts) = (old.category_counts(), staged.category_counts());
    for (cat, was) in &old_counts {
        let now = new_counts.get(cat).copied().unwrap_or(0);
        if now < *was {
            out.push(Finding::block(format!(
                "category {cat} shrank from {was} files to {now}"
            )));
        } else if now > was * LARGE_GROWTH {
            out.push(Finding::warn(format!(
                "category {cat} grew from {was} files to {now}"
            )));
        }
    }
    for c in &old.classes {
        if !staged.classes.contains(c) {
            out.push(Finding::block(format!(
                "class {c} was in the type-name table and is missing now"
            )));
        }
    }

    match (staged.build_id.parse::<u64>(), old.build_id.parse::<u64>()) {
        (Ok(new), Ok(was)) if new < was => out.push(Finding::block(format!(
            "game build {new} is older than the last good data's build {was}"
        ))),
        (Ok(new), Ok(was)) if new > was => out.push(Finding::warn(format!(
            "new game build {new} (the last good data is from build {was})"
        ))),
        (Ok(_), Ok(_)) => {}
        _ => out.push(Finding::warn(
            "a game build ID is not a number, so the builds were not compared".to_string(),
        )),
    }
    out
}

/// True if any finding stops the swap.
pub fn blocks(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::Block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::tests::sample;
    use crate::manifest::FileRecord;

    fn has(findings: &[Finding], sev: Severity, text: &str) -> bool {
        findings
            .iter()
            .any(|f| f.severity == sev && f.message.contains(text))
    }

    #[test]
    fn a_good_extraction_passes_with_and_without_a_last_good() {
        let m = sample();
        assert!(validate(&m, None).is_empty());
        assert!(validate(&m, Some(&m)).is_empty());
    }

    #[test]
    fn a_missing_needed_file_blocks_even_the_first_update() {
        let mut m = sample();
        m.missing.push("A/Gone.user".into());
        let f = validate(&m, None);
        assert!(has(&f, Severity::Block, "A/Gone.user"));
        assert!(blocks(&f));
    }

    #[test]
    fn a_file_that_does_not_open_blocks_and_is_named() {
        let mut m = sample();
        m.files[1].opens = false;
        let f = validate(&m, None);
        assert!(has(&f, Severity::Block, "A/Two.user"));
    }

    #[test]
    fn no_class_names_blocks() {
        let mut m = sample();
        m.classes.clear();
        assert!(blocks(&validate(&m, None)));
    }

    #[test]
    fn a_file_or_category_that_disappears_blocks() {
        let old = sample();
        let mut new = sample();
        new.files.pop();
        let f = validate(&new, Some(&old));
        assert!(has(
            &f,
            Severity::Block,
            "A/Two.user was in the last good data"
        ));
        assert!(has(
            &f,
            Severity::Block,
            "category skills shrank from 2 files to 1"
        ));
    }

    #[test]
    fn a_lost_class_blocks() {
        let old = sample();
        let mut new = sample();
        new.classes = vec!["app.savedata.cOther".into()];
        let f = validate(&new, Some(&old));
        assert!(has(&f, Severity::Block, "class app.savedata.cA"));
    }

    #[test]
    fn an_older_build_blocks_and_a_newer_one_warns() {
        let old = sample();
        let mut older = sample();
        older.build_id = "99".into();
        assert!(has(&validate(&older, Some(&old)), Severity::Block, "older"));
        let mut newer = sample();
        newer.build_id = "101".into();
        let f = validate(&newer, Some(&old));
        assert!(!blocks(&f));
        assert!(has(&f, Severity::Warn, "new game build 101"));
        let mut odd = sample();
        odd.build_id = "abc".into();
        let f = validate(&odd, Some(&old));
        assert!(!blocks(&f));
        assert!(has(&f, Severity::Warn, "not a number"));
    }

    #[test]
    fn large_growth_warns_but_does_not_block() {
        let old = sample();
        let mut new = sample();
        for i in 0..5 {
            new.files.push(FileRecord {
                path: format!("A/Extra{i}.user"),
                ..new.files[0].clone()
            });
        }
        let f = validate(&new, Some(&old));
        assert!(!blocks(&f));
        assert!(has(
            &f,
            Severity::Warn,
            "category skills grew from 2 files to 7"
        ));
    }
}
