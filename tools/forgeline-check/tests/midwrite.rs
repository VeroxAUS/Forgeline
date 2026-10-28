// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `forgeline-check midwrite` on a fixture save: partial writes (and on Windows a real lock) are
//! seen as errors, never as a wrong view, and the engine recovers.

use std::path::Path;
use std::process::Command;

#[test]
fn a_half_written_save_never_gives_a_wrong_view_and_the_engine_recovers() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves/one-slot.bin");
    let work = std::env::temp_dir().join(format!("forgeline-midwrite-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let out = Command::new(env!("CARGO_BIN_EXE_forgeline-check"))
        .args(["midwrite", "--file"])
        .arg(&fixture)
        .args(["--steam-id", "76561197960269970", "--work-dir"])
        .arg(&work)
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{text}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("result:       PASS"), "{text}");
    assert!(
        text.contains("final view:   original summary, no error"),
        "{text}"
    );
    assert!(!text.contains("Fixture Hunter") && !text.contains("FIXT-0001-NOPE"));
    let _ = std::fs::remove_dir_all(&work);
}
