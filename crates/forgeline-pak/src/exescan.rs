// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Route A: rebuild class and field names from the strings in the game's own `.exe` on disk.
//!
//! The exe holds field names (`_Xxx`) and short class names (`cXxx`) as plain strings, and only the
//! namespace pieces of the full class names. A name is found by hashing a string the way the save
//! does (murmur3, seed `0xFFFFFFFF`) and checking the hash against the set the save uses. For
//! classes the full name is rebuilt as namespace + "." + short name, but only for the namespaces
//! asked for (`app.savedata`) and only for short names that look like classes, so the rebuild
//! takes a moment, not minutes. The exe is read in blocks, never all at once, and only read.

use crate::paths::murmur3;
use crate::PakError;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const SEED: u32 = 0xFFFF_FFFF;
const MIN_LEN: usize = 3;
const MAX_LEN: usize = 200;
const BLOCK: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan {
    /// Hash and name for every wanted hash that was found. If two names fit one hash, the
    /// shorter one is kept (the longer is almost certainly a coincidence of a 32-bit hash).
    pub names: BTreeMap<u32, String>,
    /// Printable strings looked at.
    pub strings: u64,
    /// Short class-like names found (candidates for the namespace rebuild).
    pub class_like: usize,
    pub exe_size: u64,
    /// Modified time of the exe, seconds since 1970, if the system says.
    pub exe_modified: Option<u64>,
}

/// A short class name: `c`, an upper-case letter, then letters, digits and underscores.
fn is_class_like(s: &[u8]) -> bool {
    s.len() >= 3
        && s[0] == b'c'
        && s[1].is_ascii_uppercase()
        && s.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_')
}

fn keep(names: &mut BTreeMap<u32, String>, hash: u32, name: &str) {
    match names.get(&hash) {
        Some(old) if old.len() <= name.len() => {}
        _ => {
            names.insert(hash, name.to_string());
        }
    }
}

/// Scans the exe. `wanted` is the set of hashes the save uses (see
/// `forgeline_save::hashes::SaveHashes::all`); `namespaces` are the namespaces to try for class
/// names, for example `["app.savedata"]`.
pub fn scan(exe: &Path, wanted: &HashSet<u32>, namespaces: &[&str]) -> Result<Scan, PakError> {
    let file_err = |e| PakError::Io(exe.display().to_string(), e);
    let mut f = File::open(exe).map_err(file_err)?;
    let meta = f.metadata().map_err(file_err)?;
    let mut scan = Scan {
        names: BTreeMap::new(),
        strings: 0,
        class_like: 0,
        exe_size: meta.len(),
        exe_modified: meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
    };

    let mut shorts: HashSet<Vec<u8>> = HashSet::new();
    let mut cur: Vec<u8> = Vec::with_capacity(MAX_LEN + 1);
    let mut block = vec![0u8; BLOCK];
    loop {
        let n = f.read(&mut block).map_err(file_err)?;
        if n == 0 {
            break;
        }
        for &b in &block[..n] {
            if (0x21..0x7f).contains(&b) {
                // A run longer than the limit is not a name; stop collecting it.
                if cur.len() <= MAX_LEN {
                    cur.push(b);
                }
            } else {
                flush(&mut cur, wanted, &mut scan, &mut shorts);
            }
        }
    }
    flush(&mut cur, wanted, &mut scan, &mut shorts);

    scan.class_like = shorts.len();
    let mut full = Vec::new();
    for ns in namespaces {
        for short in &shorts {
            full.clear();
            full.extend_from_slice(ns.as_bytes());
            full.push(b'.');
            full.extend_from_slice(short);
            let h = murmur3(&full, SEED);
            if wanted.contains(&h) {
                if let Ok(name) = std::str::from_utf8(&full) {
                    keep(&mut scan.names, h, name);
                }
            }
        }
    }
    Ok(scan)
}

fn flush(cur: &mut Vec<u8>, wanted: &HashSet<u32>, scan: &mut Scan, shorts: &mut HashSet<Vec<u8>>) {
    if (MIN_LEN..=MAX_LEN).contains(&cur.len()) {
        scan.strings += 1;
        let h = murmur3(cur, SEED);
        if wanted.contains(&h) {
            if let Ok(s) = std::str::from_utf8(cur) {
                keep(&mut scan.names, h, s);
            }
        }
        if is_class_like(cur) && !shorts.contains(cur.as_slice()) {
            shorts.insert(cur.clone());
        }
    }
    cur.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exe_with(strings: &[&str]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let mut bytes = vec![0u8, 0x90, 0xCC];
        for s in strings {
            bytes.extend_from_slice(s.as_bytes());
            bytes.extend_from_slice(&[0, 0, 0]);
        }
        let p = std::env::temp_dir().join(format!(
            "forgeline-exescan-{}-{}.bin",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&p, bytes).unwrap();
        p
    }

    fn h(name: &str) -> u32 {
        murmur3(name.as_bytes(), SEED)
    }

    #[test]
    fn finds_field_names_and_rebuilds_class_names() {
        let exe = exe_with(&[
            "_EquipBox",
            "app.savedata",
            "cEquipWork",
            "cUserSaveData",
            "junk",
        ]);
        let wanted: HashSet<u32> = [
            h("_EquipBox"),
            h("app.savedata.cEquipWork"),
            h("app.savedata.cUserSaveData"),
            h("app.other.cEquipWork"),
            h("_NotInThisExe"),
        ]
        .into();
        let s = scan(&exe, &wanted, &["app.savedata"]).unwrap();
        std::fs::remove_file(&exe).ok();
        assert_eq!(s.names[&h("_EquipBox")], "_EquipBox");
        assert_eq!(
            s.names[&h("app.savedata.cEquipWork")],
            "app.savedata.cEquipWork"
        );
        assert!(s.names.contains_key(&h("app.savedata.cUserSaveData")));
        // Another namespace was not asked for; a name not in the exe stays missing.
        assert!(!s.names.contains_key(&h("app.other.cEquipWork")));
        assert!(!s.names.contains_key(&h("_NotInThisExe")));
        assert_eq!(s.class_like, 2);
        assert!(s.exe_size > 0 && s.exe_modified.is_some());
    }

    #[test]
    fn strings_that_span_a_block_boundary_are_still_found() {
        // Put a name across the 8 MiB block edge.
        let mut bytes = vec![0u8; BLOCK - 4];
        bytes.extend_from_slice(b"_AcrossTheEdge");
        bytes.push(0);
        let p = std::env::temp_dir().join(format!("forgeline-exescan-edge-{}", std::process::id()));
        std::fs::write(&p, &bytes).unwrap();
        let wanted: HashSet<u32> = [h("_AcrossTheEdge")].into();
        let s = scan(&p, &wanted, &[]).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(s.names.contains_key(&h("_AcrossTheEdge")));
    }

    #[test]
    fn overlong_runs_and_missing_files_are_handled() {
        let long = "x".repeat(MAX_LEN + 50);
        let exe = exe_with(&[&long, "_Short"]);
        let wanted: HashSet<u32> = [h("_Short"), h(&long)].into();
        let s = scan(&exe, &wanted, &[]).unwrap();
        std::fs::remove_file(&exe).ok();
        assert!(s.names.contains_key(&h("_Short")));
        assert!(!s.names.contains_key(&h(&long)));
        assert!(scan(Path::new("/definitely/not/here.exe"), &wanted, &[]).is_err());
    }
}
