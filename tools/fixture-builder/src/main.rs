// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builds the invented fixture saves in the real container format, for tests of the save reader.
//! Offline, deterministic, writes only into the folder it is given (default `fixtures/saves`), and
//! refuses a folder inside Steam, the game install or `win64_save`.

mod crypto;
mod fixtures;
mod stream;

use forgeline_core::datadir::is_protected;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: fixture-builder [--out <folder>]   (default: fixtures/saves)";

fn main() -> ExitCode {
    let mut out = PathBuf::from("fixtures/saves");
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => match args.next() {
                Some(d) => out = PathBuf::from(d),
                None => return usage(),
            },
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ => return usage(),
        }
    }
    let absolute = std::path::absolute(&out).unwrap_or_else(|_| out.clone());
    if is_protected(&absolute, &[]) {
        eprintln!(
            "{} is inside Steam, the game install or the save folder; refusing to write there",
            out.display()
        );
        return ExitCode::from(2);
    }
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("cannot create {}: {e}", out.display());
        return ExitCode::FAILURE;
    }
    for f in fixtures::all() {
        let bin = out.join(format!("{}.bin", f.name));
        let expected = out.join(format!("{}.expected.json", f.name));
        let bytes = fixtures::save_bytes(&f);
        if let Err(e) = std::fs::write(&bin, &bytes)
            .and_then(|()| std::fs::write(&expected, fixtures::expected_json(&f)))
        {
            eprintln!("cannot write {}: {e}", out.display());
            return ExitCode::FAILURE;
        }
        println!("{}: {} bytes", bin.display(), bytes.len());
    }
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The files committed under `fixtures/saves` must be exactly what the builder produces now.
    #[test]
    fn committed_fixtures_match_the_builder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/saves");
        if !dir.is_dir() {
            return; // not generated yet
        }
        for f in fixtures::all() {
            let bin = std::fs::read(dir.join(format!("{}.bin", f.name)))
                .expect("fixture .bin is committed");
            assert!(
                bin == fixtures::save_bytes(&f),
                "{}.bin is out of date: run fixture-builder",
                f.name
            );
            let json = std::fs::read_to_string(dir.join(format!("{}.expected.json", f.name)))
                .expect("expected json is committed");
            assert_eq!(
                json,
                fixtures::expected_json(&f),
                "{}.expected.json is out of date",
                f.name
            );
        }
    }

    #[test]
    fn refuses_steam_and_save_folders() {
        assert!(is_protected(
            std::path::Path::new("/x/Steam/userdata/1/2246340/remote/win64_save"),
            &[]
        ));
    }
}
