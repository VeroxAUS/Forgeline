// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check engine` on a fixture save: first run has no cache, second shows it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(data: &Path) -> String {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(["engine", "--data-dir"])
        .arg(data)
        .arg("--file")
        .arg(fixtures.join("one-slot.bin"))
        .args(["--steam-id", "76561197960269970", "--steam-dir"])
        .arg(data)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn second_run_starts_from_the_cache_and_prints_no_hunter_data() {
    let data: PathBuf =
        std::env::temp_dir().join(format!("forgeline-check-engine-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    std::fs::create_dir_all(&data).unwrap();
    let first = run(&data);
    assert!(first.contains("nothing cached yet"), "{first}");
    assert!(first.contains("3 slot(s), active 1"), "{first}");
    let second = run(&data);
    assert!(second.contains("from the cache"), "{second}");
    for out in [&first, &second] {
        assert!(!out.contains("Fixture Hunter") && !out.contains("FIXT-0001-NOPE"));
    }
    let _ = std::fs::remove_dir_all(&data);
}
