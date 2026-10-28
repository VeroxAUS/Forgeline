// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Smoke test: the binary runs `--headless-check`, exits cleanly, and writes a valid results file.

use forgeline_core::results::validate;
use std::path::PathBuf;
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("forgeline-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn forgeline() -> Command {
    Command::new(env!("CARGO_BIN_EXE_forgeline"))
}

#[test]
fn headless_check_passes_and_writes_a_valid_results_file() {
    let out = scratch("results");
    let fixtures = scratch("fixtures");
    let status = forgeline()
        .arg("--headless-check")
        .arg(&fixtures)
        .arg("--results-dir")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success());
    let files: Vec<_> = std::fs::read_dir(&out).unwrap().flatten().collect();
    assert_eq!(files.len(), 1);
    let r = validate(&std::fs::read_to_string(files[0].path()).unwrap()).unwrap();
    assert_eq!(r.scenario, "ci-headless");
    assert!(r.median > 0.0);
    let _ = std::fs::remove_dir_all(out);
    let _ = std::fs::remove_dir_all(fixtures);
}

#[test]
fn headless_check_fails_when_over_the_limit() {
    let out = scratch("over");
    let status = forgeline()
        .args(["--headless-check", "--limit-mb", "0"])
        .arg("--results-dir")
        .arg(&out)
        .status()
        .unwrap();
    assert!(!status.success());
    let _ = std::fs::remove_dir_all(out);
}

#[test]
fn a_missing_fixture_folder_is_a_usage_error() {
    let status = forgeline()
        .args(["--headless-check", "no-such-fixture-folder"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
}

#[test]
fn no_arguments_prints_usage() {
    let status = forgeline().status().unwrap();
    assert_eq!(status.code(), Some(2));
}
