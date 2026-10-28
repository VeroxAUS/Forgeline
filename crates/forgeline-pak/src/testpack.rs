// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Builds tiny synthetic packs (invented paths and bytes) for tests in this and other crates.
//! Behind the `test-support` feature; never part of the app. The table is encrypted with the
//! reader's own XOR routine, so tests using it check the table layout and the rules around it, not
//! the key constant (only the real install proves that).

use crate::index::table_xor;
use crate::paths::hash_pair;

pub const KEY: [u8; 128] = {
    let mut k = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        k[i] = (i as u8).wrapping_mul(7).wrapping_add(3);
        i += 1;
    }
    k
};

pub enum Data {
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

pub struct Item {
    pub path: String,
    pub data: Data,
}

pub struct Chunk {
    pub bytes: Vec<u8>,
}

pub struct Spec {
    pub features: u16,
    pub items: Vec<Item>,
    pub block: u32,
    pub chunks: Vec<Chunk>,
    pub encrypt: bool,
}

pub fn build(spec: &Spec) -> Vec<u8> {
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

pub fn plain(items: Vec<Item>) -> Spec {
    Spec {
        features: 0x08,
        items,
        block: 0,
        chunks: vec![],
        encrypt: true,
    }
}
