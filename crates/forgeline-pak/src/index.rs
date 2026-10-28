// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reads the headers and entry tables of the original `re_*.pak` files and keeps one index over
//! all of them, where a later pack (a patch) overrides an earlier one. Only the tables are read,
//! never file data, and only files in the install folder itself whose names start with `re_` and
//! end in `.pak`: `pak_mods` and everything else is never opened.
//!
//! Pack layout (version 4, found on the real install; format facts credit eigeen/ree-pak-rs, MIT):
//! 16-byte header (`KPKA`, major, minor, feature flags, entry count, hash), the entry table
//! (48 bytes per entry), then, by feature flag, a 128-byte table key (`0x08`, the table is
//! XOR-encrypted) and a chunk table (`0x20`, used by the newest patches).

use crate::PakError;
use num_bigint::BigUint;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Feature flags this reader understands: encrypted table and chunk table.
const KNOWN_FEATURES: u16 = 0x08 | 0x20;
const FEATURE_ENCRYPTED: u16 = 0x08;
const FEATURE_CHUNKS: u16 = 0x20;
/// Attribute bit 24: the entry stores a chunk index instead of an offset.
pub(crate) const ATTR_CHUNKED: u32 = 1 << 24;
/// A table or chunk list bigger than this is a corrupt count, not a real pack.
const MAX_ENTRIES: usize = 5_000_000;

/// The table key's modulus (129 bytes, little endian), copied from eigeen/ree-pak-rs (MIT).
/// Credit and licence text: `THIRD-PARTY-NOTICES.md`.
const MODULUS: [u8; 129] = [
    0x7D, 0x0B, 0xF8, 0xC1, 0x7C, 0x23, 0xFD, 0x3B, 0xD4, 0x75, 0x16, 0xD2, 0x33, 0x21, 0xD8, 0x10,
    0x71, 0xF9, 0x7C, 0xD1, 0x34, 0x93, 0xBA, 0x77, 0x26, 0xFC, 0xAB, 0x2C, 0xEE, 0xDA, 0xD9, 0x1C,
    0x89, 0xE7, 0x29, 0x7B, 0xDD, 0x8A, 0xAE, 0x50, 0x39, 0xB6, 0x01, 0x6D, 0x21, 0x89, 0x5D, 0xA5,
    0xA1, 0x3E, 0xA2, 0xC0, 0x8C, 0x93, 0x13, 0x36, 0x65, 0xEB, 0xE8, 0xDF, 0x06, 0x17, 0x67, 0x96,
    0x06, 0x2B, 0xAC, 0x23, 0xED, 0x8C, 0xB7, 0x8B, 0x90, 0xAD, 0xEA, 0x71, 0xC4, 0x40, 0x44, 0x9D,
    0x1C, 0x7B, 0xBA, 0xC4, 0xB6, 0x2D, 0xD6, 0xD2, 0x4B, 0x62, 0xD6, 0x26, 0xFC, 0x74, 0x20, 0x07,
    0xEC, 0xE3, 0x59, 0x9A, 0xE6, 0xAF, 0xB9, 0xA8, 0x35, 0x8B, 0xE0, 0xE8, 0xD3, 0xCD, 0x45, 0x65,
    0xB0, 0x91, 0xC4, 0x95, 0x1B, 0xF3, 0x23, 0x1E, 0xC6, 0x71, 0xCF, 0x3E, 0x35, 0x2D, 0x6B, 0xE3,
    0x00,
];

/// Decrypts (or encrypts: it is a byte-wise XOR) an entry table in place. The 128-byte key from
/// the file is raised to the power 65537 modulo [`MODULUS`] and the first 32 bytes of the result
/// drive the XOR.
pub fn table_xor(data: &mut [u8], file_key: &[u8; 128]) {
    let mut k = file_key.to_vec();
    k.push(0);
    let n = BigUint::from_bytes_le(&MODULUS);
    let mut key = BigUint::from_bytes_le(&k)
        .modpow(&BigUint::from(0x10001u32), &n)
        .to_bytes_le();
    key.resize(32, 0);
    for (i, b) in data.iter_mut().enumerate() {
        *b ^= i.wrapping_add(usize::from(key[i % 32]) * usize::from(key[i % 29])) as u8;
    }
}

/// Where one entry lives. Sizes are kept as 32-bit numbers so the index stays small; a larger
/// entry is kept as `u32::MAX` and refused when read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryRef {
    pub(crate) pack: u16,
    pub(crate) offset: u64,
    pub(crate) stored: u32,
    pub(crate) size: u32,
    pub(crate) attr: u32,
}

impl EntryRef {
    /// Size of the file once unpacked (0 when the table does not say).
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Index of the pack it comes from, in load order.
    pub fn pack(&self) -> usize {
        usize::from(self.pack)
    }
}

/// Chunk list of one pack: block size and, per chunk, where it starts and how long it is.
#[derive(Debug, Default)]
pub(crate) struct Chunks {
    pub(crate) block: u32,
    pub(crate) list: Vec<(u64, u32)>,
}

#[derive(Debug)]
pub(crate) struct Pack {
    pub(crate) path: PathBuf,
    pub(crate) chunks: Option<Chunks>,
    pub(crate) entries: usize,
}

/// The index over every original pack of an install.
#[derive(Debug)]
pub struct PackSet {
    pub(crate) packs: Vec<Pack>,
    map: HashMap<u64, EntryRef>,
}

fn key(lower: u32, upper: u32) -> u64 {
    (u64::from(upper) << 32) | u64::from(lower)
}

fn io(path: &Path, e: std::io::Error) -> PakError {
    PakError::Io(path.display().to_string(), e)
}

fn le32(b: &[u8]) -> u32 {
    u32::from_le_bytes([b[0], b[1], b[2], b[3]])
}

fn le64(b: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[..8]);
    u64::from_le_bytes(a)
}

/// Load order of the original packs: the base packs, then each patch in number order, the
/// `sub_000` family after the base family and the DLC pack last. A later pack overrides an
/// earlier one.
fn load_order(name: &str) -> (bool, bool, u32, String) {
    let patch = name
        .rsplit_once("patch_")
        .and_then(|(_, n)| n.trim_end_matches(".pak").parse::<u32>().ok())
        .unwrap_or(0);
    (
        name.contains("re_dlc"),
        name.contains(".sub_000.pak"),
        patch,
        name.to_string(),
    )
}

/// Lists the original packs of an install: files directly in `install` named `re_*.pak`.
pub fn original_pack_files(install: &Path) -> Result<Vec<PathBuf>, PakError> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(install).map_err(|e| io(install, e))? {
        let entry = entry.map_err(|e| io(install, e))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
        if is_file && name.starts_with("re_") && name.ends_with(".pak") {
            files.push(entry.path());
        }
    }
    files.sort_by_key(|p| load_order(&p.file_name().unwrap_or_default().to_string_lossy()));
    Ok(files)
}

struct Table {
    entries: Vec<[u8; 48]>,
    chunks: Option<Chunks>,
}

/// Reads one pack's header, entry table and chunk list.
fn read_table(path: &Path) -> Result<Table, PakError> {
    let mut f = File::open(path).map_err(|e| io(path, e))?;
    let name = path.display().to_string();
    let mut h = [0u8; 16];
    f.read_exact(&mut h).map_err(|e| io(path, e))?;
    if &h[0..4] != b"KPKA" {
        return Err(PakError::Unsupported(format!("{name}: not a pack file")));
    }
    if h[4] != 4 {
        return Err(PakError::Unsupported(format!(
            "{name}: version {}.{}",
            h[4], h[5]
        )));
    }
    let features = u16::from_le_bytes([h[6], h[7]]);
    if features & !KNOWN_FEATURES != 0 {
        return Err(PakError::Unsupported(format!(
            "{name}: feature flags {features:#x}"
        )));
    }
    let total = le32(&h[8..12]) as usize;
    if total > MAX_ENTRIES {
        return Err(PakError::Corrupt(format!("{name}: {total} entries")));
    }
    let mut raw = vec![0u8; total * 48];
    f.read_exact(&mut raw).map_err(|e| io(path, e))?;
    if features & FEATURE_ENCRYPTED != 0 {
        let mut k = [0u8; 128];
        f.read_exact(&mut k).map_err(|e| io(path, e))?;
        table_xor(&mut raw, &k);
    }
    let entries = raw
        .chunks_exact(48)
        .map(|c| <[u8; 48]>::try_from(c).expect("48-byte chunk"))
        .collect();
    let chunks = if features & FEATURE_CHUNKS != 0 {
        let mut hb = [0u8; 8];
        f.read_exact(&mut hb).map_err(|e| io(path, e))?;
        let (block, count) = (le32(&hb[0..4]), le32(&hb[4..8]) as usize);
        if count > MAX_ENTRIES * 8 {
            return Err(PakError::Corrupt(format!("{name}: {count} chunks")));
        }
        let mut raw = vec![0u8; count * 8];
        f.read_exact(&mut raw).map_err(|e| io(path, e))?;
        // Start offsets are 32-bit and wrap at 4 GiB; the high bits are rebuilt from the fact
        // that the offsets only grow. The length sits in the upper 22 bits of the second word.
        let (mut high, mut prev) = (0u64, 0u32);
        let list = raw
            .chunks_exact(8)
            .map(|c| {
                let lo = le32(&c[0..4]);
                if lo < prev {
                    high += 1 << 32;
                }
                prev = lo;
                (high | u64::from(lo), le32(&c[4..8]) >> 10)
            })
            .collect();
        Some(Chunks { block, list })
    } else {
        None
    };
    Ok(Table { entries, chunks })
}

impl PackSet {
    /// Indexes the original packs of the game install.
    pub fn open(install: &Path) -> Result<PackSet, PakError> {
        PackSet::from_files(original_pack_files(install)?)
    }

    /// Indexes the given pack files in the given order (later overrides earlier).
    pub fn from_files(files: Vec<PathBuf>) -> Result<PackSet, PakError> {
        if files.len() > usize::from(u16::MAX) {
            return Err(PakError::Corrupt("too many packs".into()));
        }
        let mut set = PackSet {
            packs: Vec::with_capacity(files.len()),
            map: HashMap::new(),
        };
        for (i, path) in files.into_iter().enumerate() {
            let table = read_table(&path)?;
            set.map.reserve(table.entries.len());
            for e in &table.entries {
                let (stored, size, attr) = (le64(&e[16..24]), le64(&e[24..32]), le64(&e[32..40]));
                // A size above 4 GiB cannot be a file we need; it is kept as the maximum, which
                // the reader refuses. Only the low attribute bits are used.
                let to32 = |v: u64| u32::try_from(v).unwrap_or(u32::MAX);
                let (stored, size) = (to32(stored), to32(size));
                let attr = attr as u32;
                set.map.insert(
                    key(le32(&e[0..4]), le32(&e[4..8])),
                    EntryRef {
                        pack: i as u16,
                        offset: le64(&e[8..16]),
                        stored,
                        size,
                        attr,
                    },
                );
            }
            set.packs.push(Pack {
                path,
                chunks: table.chunks,
                entries: table.entries.len(),
            });
        }
        Ok(set)
    }

    /// The entry for a path's two hashes (see [`crate::paths::hash_pair`]); the last pack that
    /// holds it wins.
    pub fn entry(&self, lower: u32, upper: u32) -> Option<EntryRef> {
        self.map.get(&key(lower, upper)).copied()
    }

    /// Distinct files after later packs override earlier ones.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn pack_count(&self) -> usize {
        self.packs.len()
    }

    /// Pack file names in load order with their entry counts.
    pub fn pack_files(&self) -> impl Iterator<Item = (&Path, usize)> {
        self.packs.iter().map(|p| (p.path.as_path(), p.entries))
    }
}
