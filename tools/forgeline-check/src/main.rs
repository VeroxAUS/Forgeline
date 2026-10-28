// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Developer report for PC test rounds. Prints what the foundation sees as counts and names only:
//! never a hunter name or ID, never file contents. Reads nothing from the save or the install
//! beyond folder listings and two small Steam text files, and writes nothing at all.

use forgeline_core::discovery::{self, DiscoveryEnv, SaveAccount};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: forgeline-check discover [--steam-dir <folder>]";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("discover") => discover(args.collect()),
        Some("-h" | "--help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn discover(args: Vec<String>) -> ExitCode {
    let mut steam_dir: Option<PathBuf> = None;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--steam-dir" => match it.next() {
                Some(d) => steam_dir = Some(PathBuf::from(d)),
                None => {
                    eprintln!("--steam-dir needs a folder\n{USAGE}");
                    return ExitCode::from(2);
                }
            },
            _ => {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(steam_root) = steam_dir.or_else(discovery::find_steam_root) else {
        eprintln!("Steam was not found; pass --steam-dir <folder>");
        return ExitCode::FAILURE;
    };

    println!("steam folder: {}", shown(&steam_root));
    let found = match discovery::discover(&DiscoveryEnv { steam_root }) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("discovery failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("library:      {}", shown(&found.library));
    println!("install:      {}", shown(&found.install));
    println!("build id:     {}", found.build_id);

    match found.save_account() {
        SaveAccount::None => {
            println!("saves:        no Steam account folder with Wilds saves found")
        }
        SaveAccount::One(_) => println!("saves:        1 Steam account folder with Wilds saves"),
        SaveAccount::Several(many) => println!(
            "saves:        {} Steam account folders with Wilds saves (the window will ask which)",
            many.len()
        ),
    }
    for (n, a) in found.accounts.iter().enumerate() {
        // Only the tail of the Steam ID is shown, so a pasted report does not identify the account.
        let id = a.steam_id64.to_string();
        println!(
            "  account {}: Steam ID ending {}, {} slot file(s): {}",
            n + 1,
            &id[id.len().saturating_sub(4)..],
            a.slot_files.len(),
            a.slot_files
                .iter()
                .map(|f| format!("{} ({} bytes)", f.name, f.size))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if found.mod_markers.is_empty() {
        println!("mod markers:  none found in the install (listed by name only, never opened)");
    } else {
        println!(
            "mod markers:  {} (found by name in the install; none of them was opened or read)",
            found.mod_markers.join(", ")
        );
    }
    ExitCode::SUCCESS
}

/// A path for display with the user's home folder replaced, so a pasted report holds no user name.
fn shown(p: &Path) -> String {
    let text = p.display().to_string();
    for var in ["USERPROFILE", "HOME"] {
        if let Some(home) = std::env::var_os(var).filter(|h| !h.is_empty()) {
            let home = home.to_string_lossy().into_owned();
            if let Some(rest) = text.strip_prefix(&home) {
                return format!("~{rest}");
            }
        }
    }
    text
}
