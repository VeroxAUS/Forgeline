// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builds tiny synthetic packs (invented paths and bytes, written by this test) and reads them
//! back: an encrypted table, stored, deflate and zstd entries, a chunked entry, later packs
//! overriding earlier ones, and damaged input. The encryption here uses the reader's own XOR
//! function, so these tests check the table layout and the rules around it; whether the key
//! constant is right is only proven by the real install (`forgeline-check pak`).

use forgeline_pak::index::{original_pack_files, table_xor, PackSet};
use forgeline_pak::paths::{hash_pair, resolve, NEEDED};
use forgeline_pak::PakError;
use std::path::{Path, PathBuf};

const KEY: [u8; 128] = {
    let mut k = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        k[i] = (i as u8).wrapping_mul(7).wrapping_add(3);
        i += 1;
    }
    k
};

enum Data {
    Stored(Vec<u8>),
    Deflate(Vec<u8>),
    Zstd(Vec<u8>),
    /// Three stored bytes whose table entry claims this unpacked size.
    Claims(u64),
    /// Deflate data whose table entry is one byte off.
    WrongSize(Vec<u8>),
    /// Chunked: the chunks' stored forms are built by the test.
    Chunked {
        first_chunk: usize,
        total: usize,
    },
}

struct Item {
    path: String,
    data: Data,
}

struct Chunk {
    bytes: Vec<u8>,
}

struct Spec {
    features: u16,
    items: Vec<Item>,
    block: u32,
    chunks: Vec<Chunk>,
    encrypt: bool,
}

fn build(spec: &Spec) -> Vec<u8> {
    let n = spec.items.len();
    let mut head = Vec::new();
    head.extend_from_slice(b"KPKA");
    head.extend_from_slice(&[4, 1]);
    head.extend_from_slice(&spec.features.to_le_bytes());
    head.extend_from_slice(&(n as u32).to_le_bytes());
    head.extend_from_slice(&[0; 4]);
    let table_len = n * 48;
    let key_len = if spec.features & 0x08 != 0 { 128 } else { 0 };
    let chunk_len = if spec.features & 0x20 != 0 {
        8 + spec.chunks.len() * 8
    } else {
        0
    };
    let mut at = (16 + table_len + key_len + chunk_len) as u64;

    let mut blob: Vec<u8> = Vec::new();
    let mut table = Vec::new();
    for it in &spec.items {
        let (lower, upper) = hash_pair(&it.path);
        let (offset, stored, size, attr, bytes): (u64, u64, u64, u64, Vec<u8>) = match &it.data {
            Data::Stored(b) => (at, b.len() as u64, b.len() as u64, 0, b.clone()),
            Data::Deflate(b) => {
                let c = miniz_oxide::deflate::compress_to_vec(b, 6);
                (at, c.len() as u64, b.len() as u64, 1, c)
            }
            Data::Zstd(b) => {
                let c = zstd::bulk::compress(b, 3).unwrap();
                (at, c.len() as u64, b.len() as u64, 2, c)
            }
            Data::Claims(size) => (at, 3, *size, 0, vec![1, 2, 3]),
            Data::WrongSize(b) => {
                let c = miniz_oxide::deflate::compress_to_vec(b, 6);
                (at, c.len() as u64, b.len() as u64 + 1, 1, c)
            }
            Data::Chunked { first_chunk, total } => {
                (*first_chunk as u64, 0, *total as u64, 1 << 24 | 2, vec![])
            }
        };
        table.extend_from_slice(&lower.to_le_bytes());
        table.extend_from_slice(&upper.to_le_bytes());
        table.extend_from_slice(&offset.to_le_bytes());
        table.extend_from_slice(&stored.to_le_bytes());
        table.extend_from_slice(&size.to_le_bytes());
        table.extend_from_slice(&attr.to_le_bytes());
        table.extend_from_slice(&[0; 8]);
        at += bytes.len() as u64;
        blob.extend_from_slice(&bytes);
    }
    if spec.encrypt {
        table_xor(&mut table, &KEY);
    }
    let mut out = head;
    out.extend_from_slice(&table);
    if key_len != 0 {
        out.extend_from_slice(&KEY);
    }
    if chunk_len != 0 {
        out.extend_from_slice(&spec.block.to_le_bytes());
        out.extend_from_slice(&(spec.chunks.len() as u32).to_le_bytes());
        let mut cat = at;
        for c in &spec.chunks {
            out.extend_from_slice(&(cat as u32).to_le_bytes());
            out.extend_from_slice(&((c.bytes.len() as u32) << 10).to_le_bytes());
            cat += c.bytes.len() as u64;
        }
    }
    out.extend_from_slice(&blob);
    for c in &spec.chunks {
        out.extend_from_slice(&c.bytes);
    }
    out
}

fn plain(items: Vec<Item>) -> Spec {
    Spec {
        features: 0x08,
        items,
        block: 0,
        chunks: vec![],
        encrypt: true,
    }
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("forgeline-pak-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn sample() -> Vec<u8> {
    (0..300u32).map(|i| (i % 17) as u8 + b'a').collect()
}

fn path_of(stem: &str, version: u32) -> String {
    format!("natives/stm/{stem}.{version}")
}

#[test]
fn reads_stored_deflate_and_zstd_entries_from_an_encrypted_table() {
    let dir = temp_dir("kinds");
    let bytes = sample();
    let spec = plain(vec![
        Item {
            path: path_of("Test/Stored.user", 3),
            data: Data::Stored(bytes.clone()),
        },
        Item {
            path: path_of("Test/Deflate.user", 12),
            data: Data::Deflate(bytes.clone()),
        },
        Item {
            path: path_of("Test/Zstd.user", 70),
            data: Data::Zstd(bytes.clone()),
        },
    ]);
    let file = dir.join("re_chunk_000.pak");
    std::fs::write(&file, build(&spec)).unwrap();

    let set = PackSet::open(&dir).unwrap();
    assert_eq!(set.len(), 3);
    assert_eq!(set.pack_count(), 1);
    for (stem, version) in [
        ("Test/Stored.user", 3),
        ("Test/Deflate.user", 12),
        ("Test/Zstd.user", 70),
    ] {
        let (v, e) = resolve(&set, stem).unwrap_or_else(|| panic!("{stem} not found"));
        assert_eq!(v, version, "{stem}");
        assert_eq!(set.read(&e).unwrap(), bytes, "{stem}");
    }
    // Version 71 is outside the search; an unknown path is simply absent.
    assert!(resolve(&set, "Test/Missing.user").is_none());
    let (l, u) = hash_pair(&path_of("Test/Stored.user", 71));
    assert!(set.entry(l, u).is_none());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_chunked_entry_mixes_stored_and_zstd_chunks() {
    let dir = temp_dir("chunks");
    let data = sample(); // 300 bytes in blocks of 128
    let block = 128usize;
    let mut chunks = Vec::new();
    for (i, part) in data.chunks(block).enumerate() {
        // The first chunk is stored (its length equals the block size); the others are zstd.
        let bytes = if i == 0 {
            part.to_vec()
        } else {
            zstd::bulk::compress(part, 3).unwrap()
        };
        assert_eq!(
            bytes.len() == block,
            i == 0,
            "chunk {i} would be mistaken for stored"
        );
        chunks.push(Chunk { bytes });
    }
    let spec = Spec {
        features: 0x08 | 0x20,
        items: vec![Item {
            path: path_of("Test/Chunked.user", 1),
            data: Data::Chunked {
                first_chunk: 0,
                total: data.len(),
            },
        }],
        block: block as u32,
        chunks,
        encrypt: true,
    };
    std::fs::write(dir.join("re_chunk_000.pak.patch_001.pak"), build(&spec)).unwrap();
    let set = PackSet::open(&dir).unwrap();
    let (_, e) = resolve(&set, "Test/Chunked.user").unwrap();
    assert_eq!(set.read(&e).unwrap(), data);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn later_packs_override_earlier_ones_in_load_order() {
    let dir = temp_dir("order");
    let one = |text: &str| {
        plain(vec![Item {
            path: path_of("Test/Same.user", 5),
            data: Data::Stored(text.as_bytes().to_vec()),
        }])
    };
    // Written out of order on purpose; the names decide the order.
    std::fs::write(
        dir.join("re_chunk_000.pak.patch_002.pak"),
        build(&one("patch two")),
    )
    .unwrap();
    std::fs::write(dir.join("re_chunk_000.pak"), build(&one("base"))).unwrap();
    std::fs::write(
        dir.join("re_chunk_000.pak.patch_001.pak"),
        build(&one("patch one")),
    )
    .unwrap();
    std::fs::write(
        dir.join("re_chunk_000.pak.patch_010.pak"),
        build(&one("patch ten")),
    )
    .unwrap();
    std::fs::write(dir.join("re_chunk_000.pak.sub_000.pak"), build(&one("sub"))).unwrap();
    std::fs::write(dir.join("re_dlc_stm_000.pak"), build(&one("dlc"))).unwrap();

    let names: Vec<String> = original_pack_files(&dir)
        .unwrap()
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        names,
        [
            "re_chunk_000.pak",
            "re_chunk_000.pak.patch_001.pak",
            "re_chunk_000.pak.patch_002.pak",
            "re_chunk_000.pak.patch_010.pak",
            "re_chunk_000.pak.sub_000.pak",
            "re_dlc_stm_000.pak",
        ]
    );
    let set = PackSet::open(&dir).unwrap();
    assert_eq!(set.len(), 1);
    assert_eq!(set.pack_count(), 6);
    let (_, e) = resolve(&set, "Test/Same.user").unwrap();
    assert_eq!(set.read(&e).unwrap(), b"dlc");
    assert_eq!(e.pack(), 5);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn only_original_packs_in_the_install_folder_are_opened() {
    let dir = temp_dir("mods");
    let pack = build(&plain(vec![Item {
        path: path_of("Test/A.user", 1),
        data: Data::Stored(vec![1, 2, 3]),
    }]));
    std::fs::write(dir.join("re_chunk_000.pak"), &pack).unwrap();
    // None of these may be opened: a mod pack folder, a file without the prefix or suffix, and
    // pack-like files in a sub folder. Their contents are junk, so opening one would fail the open.
    std::fs::create_dir_all(dir.join("pak_mods")).unwrap();
    std::fs::write(dir.join("pak_mods").join("re_chunk_999.pak"), b"junk").unwrap();
    std::fs::write(dir.join("mod.pak"), b"junk").unwrap();
    std::fs::write(dir.join("re_chunk_001.pak.bak"), b"junk").unwrap();
    std::fs::write(dir.join("re_notes.txt"), b"junk").unwrap();
    let set = PackSet::open(&dir).unwrap();
    assert_eq!(set.pack_count(), 1);
    assert_eq!(set.len(), 1);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn damaged_and_unexpected_packs_fail_cleanly() {
    let dir = temp_dir("bad");
    let good = build(&plain(vec![Item {
        path: path_of("Test/A.user", 1),
        data: Data::Deflate(sample()),
    }]));
    let open_with = |bytes: &[u8]| -> Result<PackSet, PakError> {
        std::fs::write(dir.join("re_chunk_000.pak"), bytes).unwrap();
        PackSet::open(&dir)
    };

    assert!(matches!(
        open_with(b"KPKX0000000000000000"),
        Err(PakError::Unsupported(_))
    ));
    let mut v = good.clone();
    v[4] = 5;
    assert!(matches!(open_with(&v), Err(PakError::Unsupported(_))));
    let mut v = good.clone();
    v[6] = 0x10; // an unknown feature flag
    assert!(matches!(open_with(&v), Err(PakError::Unsupported(_))));
    let mut v = good.clone();
    v[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(open_with(&v), Err(PakError::Corrupt(_))));
    // Cut anywhere inside the header, table or key: an error, never a panic.
    for cut in 0..(16 + 48 + 128) {
        assert!(open_with(&good[..cut]).is_err(), "cut at {cut}");
    }
    assert!(open_with(&good).is_ok());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn damaged_entries_fail_cleanly_when_read() {
    let dir = temp_dir("badentry");
    let read = |spec: Spec,
                stem: &str,
                mutate: &dyn Fn(Vec<u8>) -> Vec<u8>|
     -> Result<Vec<u8>, PakError> {
        std::fs::write(dir.join("re_chunk_000.pak"), mutate(build(&spec))).unwrap();
        let set = PackSet::open(&dir)?;
        let (_, e) = resolve(&set, stem).expect("entry present");
        set.read(&e)
    };
    let item = |data| Item {
        path: path_of("Test/E.user", 1),
        data,
    };
    let keep = |b: Vec<u8>| b;

    // Truncated data: the entry runs past the end of the file.
    let cut = |mut b: Vec<u8>| {
        b.truncate(b.len() - 5);
        b
    };
    assert!(read(
        plain(vec![item(Data::Deflate(sample()))]),
        "Test/E.user",
        &cut
    )
    .is_err());
    assert!(read(plain(vec![item(Data::Zstd(sample()))]), "Test/E.user", &cut).is_err());
    assert!(read(
        plain(vec![item(Data::Stored(sample()))]),
        "Test/E.user",
        &cut
    )
    .is_err());
    // Garbage where deflate or zstd data should be.
    let junk = |mut b: Vec<u8>| {
        let n = b.len();
        for x in &mut b[n - 8..] {
            *x ^= 0xA5;
        }
        b
    };
    assert!(read(
        plain(vec![item(Data::Zstd(sample()))]),
        "Test/E.user",
        &junk
    )
    .is_err());
    // A chunked entry in a pack with no chunk table.
    let spec = plain(vec![item(Data::Chunked {
        first_chunk: 0,
        total: 10,
    })]);
    assert!(matches!(
        read(spec, "Test/E.user", &keep),
        Err(PakError::Corrupt(_))
    ));
    // A chunk index past the end of the list.
    let spec = Spec {
        features: 0x28,
        items: vec![item(Data::Chunked {
            first_chunk: 9,
            total: 10,
        })],
        block: 128,
        chunks: vec![Chunk {
            bytes: vec![0; 128],
        }],
        encrypt: true,
    };
    assert!(matches!(
        read(spec, "Test/E.user", &keep),
        Err(PakError::Corrupt(_))
    ));
    // An entry that claims to be bigger than the limit is refused before any allocation, and an
    // unpacked size that disagrees with the table is an error, not a silent wrong file.
    let spec = plain(vec![item(Data::Claims(0x2000_0000))]);
    assert!(matches!(
        read(spec, "Test/E.user", &keep),
        Err(PakError::Corrupt(_))
    ));
    let spec = plain(vec![item(Data::WrongSize(sample()))]);
    assert!(read(spec, "Test/E.user", &keep).is_err());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_needed_list_resolves_against_a_pack_that_holds_it() {
    let dir = temp_dir("needed");
    let items = NEEDED
        .iter()
        .map(|n| Item {
            path: path_of(n.stem, 3),
            data: Data::Stored(n.stem.as_bytes().to_vec()),
        })
        .collect();
    std::fs::write(dir.join("re_chunk_000.pak"), build(&plain(items))).unwrap();
    let set = PackSet::open(&dir).unwrap();
    for n in NEEDED {
        let (v, e) = resolve(&set, n.stem).unwrap_or_else(|| panic!("{} missing", n.stem));
        assert_eq!(v, 3);
        assert_eq!(set.read(&e).unwrap(), n.stem.as_bytes());
    }
    let _: &Path = &dir;
    std::fs::remove_dir_all(&dir).ok();
}
