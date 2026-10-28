// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Keeps personal data and Capcom data out of git and keeps source files uniform.
//! Runs over the tracked files (`git ls-files`), so it sees exactly what a commit would publish.
//!
//! Set `HYGIENE_RANGE` (for example `origin/main..HEAD`) to also check the identity of every
//! commit in a pull request. CI sets it; locally the commit check is skipped without it.

use forgeline_core::results::validate;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The only identities that may appear in the history: the maintainer's GitHub no-reply address,
/// Claude's no-reply address, and GitHub's own.
const ALLOWED_EMAILS: [&str; 3] = [
    "3145728+veroxaus@users.noreply.github.com",
    "noreply@anthropic.com",
    "noreply@github.com",
];

const NOTICE_MARKER: &str = "License, v. 2.0. If a copy of the MPL";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tracked_files() -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(repo_root())
        .output()
        .expect("git must be available to run the hygiene test");
    assert!(out.status.success(), "git ls-files failed");
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Problems with the tracked file names: save, pack, dump and extracted-data files, and fixtures
/// that are not listed as synthetic in `fixtures/README.md`.
fn check_file_names(files: &[String], fixtures_readme: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for f in files {
        let lower = f.to_lowercase();
        let name = lower.rsplit('/').next().unwrap_or(&lower);
        let in_fixtures = lower.starts_with("fixtures/");
        let forbidden = (lower.ends_with(".bin") && !in_fixtures)
            || lower.ends_with(".pak")
            || lower.ends_with(".user.3")
            || name.contains(".msg.")
            || name.contains("dump")
            || lower.starts_with("dumps/")
            || lower.starts_with("extracted/")
            || lower.starts_with("extraction/")
            || lower.starts_with("savecopy/")
            || lower.contains("win64_save")
            || lower.starts_with("fixtures/private/");
        if forbidden {
            problems.push(format!(
                "{f}: save, pack, dump or extracted-data file must not be tracked"
            ));
        } else if in_fixtures && f != "fixtures/README.md" {
            let base = f.rsplit('/').next().unwrap_or(f);
            if !fixtures_readme.contains(base) {
                problems.push(format!(
                    "{f}: fixture is not listed as synthetic in fixtures/README.md"
                ));
            }
        }
    }
    problems
}

/// Source files without the MPL-2.0 Exhibit A notice near the top.
fn check_notices(files: &[String], read: impl Fn(&str) -> String) -> Vec<String> {
    files
        .iter()
        .filter(|f| f.ends_with(".rs"))
        .filter(|f| {
            let head: String = read(f).lines().take(6).collect::<Vec<_>>().join("\n");
            !head.contains(NOTICE_MARKER)
        })
        .map(|f| format!("{f}: missing the MPL-2.0 notice"))
        .collect()
}

/// `git log --format=%ae%n%ce` output; every address must be one of the allowed ones.
fn check_identities(emails: &str) -> Vec<String> {
    let mut bad: Vec<String> = emails
        .lines()
        .map(str::trim)
        .filter(|e| !e.is_empty() && !ALLOWED_EMAILS.contains(&e.to_lowercase().as_str()))
        .map(|e| format!("commit identity {e} is not allowed"))
        .collect();
    bad.sort();
    bad.dedup();
    bad
}

fn read_text(rel: &str) -> String {
    std::fs::read_to_string(repo_root().join(rel)).unwrap_or_default()
}

fn fail_if_any(problems: Vec<String>) {
    assert!(problems.is_empty(), "\n{}\n", problems.join("\n"));
}

#[test]
fn no_save_pack_dump_or_extracted_data_is_tracked() {
    let files = tracked_files();
    fail_if_any(check_file_names(&files, &read_text("fixtures/README.md")));
}

#[test]
fn every_source_file_has_the_mpl_notice() {
    fail_if_any(check_notices(&tracked_files(), read_text));
}

#[test]
fn every_committed_results_file_is_valid_and_holds_no_path() {
    let problems: Vec<String> = tracked_files()
        .iter()
        .filter(|f| f.starts_with("bench/results/") && f.ends_with(".json"))
        .filter_map(|f| validate(&read_text(f)).err().map(|e| format!("{f}: {e}")))
        .collect();
    fail_if_any(problems);
}

#[test]
fn every_commit_in_the_pull_request_uses_an_allowed_identity() {
    let range = std::env::var("HYGIENE_RANGE").unwrap_or_default();
    if range.is_empty() {
        eprintln!("HYGIENE_RANGE not set: commit identities not checked");
        return;
    }
    let out = Command::new("git")
        .args(["log", "--format=%ae%n%ce", &range])
        .current_dir(repo_root())
        .output()
        .expect("git log");
    assert!(out.status.success(), "git log {range} failed");
    fail_if_any(check_identities(&String::from_utf8_lossy(&out.stdout)));
}

// The checks above are only worth anything if they can fail, so each one is tried on bad input.

#[test]
fn a_tracked_bin_file_outside_fixtures_fails() {
    let files = vec![
        "crates/x/data001Slot.bin".to_string(),
        "README.md".to_string(),
    ];
    let p = check_file_names(&files, "");
    assert_eq!(p.len(), 1, "{p:?}");
}

#[test]
fn packs_dumps_and_extracted_data_fail() {
    for bad in [
        "re_chunk_000.pak.sub_000.pak",
        "decoded-dump.json",
        "dumps/x.txt",
        "extracted/natives/a.user.3",
        "data/skills.msg.23",
    ] {
        let p = check_file_names(&[bad.to_string()], "");
        assert_eq!(p.len(), 1, "{bad}: {p:?}");
    }
}

#[test]
fn fixtures_must_be_listed_as_synthetic() {
    let files = vec![
        "fixtures/README.md".to_string(),
        "fixtures/saves/listed.bin".to_string(),
        "fixtures/saves/unlisted.bin".to_string(),
    ];
    let p = check_file_names(&files, "listed.bin: invented");
    assert_eq!(p.len(), 1, "{p:?}");
    assert!(p[0].contains("unlisted.bin"));
}

#[test]
fn a_source_file_without_the_notice_fails() {
    let files = vec!["a.rs".to_string(), "b.rs".to_string(), "c.md".to_string()];
    let p = check_notices(&files, |f| {
        if f == "a.rs" {
            format!("// {NOTICE_MARKER}\nfn main() {{}}")
        } else {
            "fn main() {}".to_string()
        }
    });
    assert_eq!(p, vec!["b.rs: missing the MPL-2.0 notice".to_string()]);
}

#[test]
fn a_personal_email_in_the_history_fails() {
    let ok =
        "3145728+VeroxAUS@users.noreply.github.com\nnoreply@anthropic.com\nnoreply@github.com\n";
    assert!(check_identities(ok).is_empty());
    let bad = format!("{ok}someone@example.com\nsomeone@example.com\n");
    assert_eq!(check_identities(&bad).len(), 1);
}
