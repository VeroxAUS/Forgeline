// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Opens the save container, following the recipe in `docs/research.md` (Phase 1 step 2 and its
//! follow-up, confirmed on a real save in step 3): a `DSSS` header, blocks encrypted with AES-128
//! in OFB mode using a key and IV drawn from a generator seeded by the 64-bit Steam ID, a tail
//! with the decrypted length, and inside the encryption a small wrapper around a raw deflate
//! stream that holds the class stream.
//!
//! The checksum at the very end of the file is not checked: the notes confirm a successful inflate
//! and an exact walk of the stream as the integrity test, and the checksum variant is not verified
//! on a real file. The proof value in the first block header is checked, which catches a wrong
//! Steam ID before any decrypting.

use crate::SaveError;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
/// Starting state of the block-size sequence (Wilds).
const SIZE_SEED: u64 = 0x7A36_9552_5526_6CED;
const UNIT: usize = 0x4000;
const BLOCK_HEADER: usize = 0x210;
const FILE_HEADER: usize = 0x10;
const TAIL_BLOB: usize = 0x80;
/// Tail: the blob, the decrypted length (u64) and the checksum (u32).
const TAIL: usize = TAIL_BLOB + 12;
/// Header flags: Deflate (0x8) plus Mandarin encryption (0x10).
const FLAGS: u32 = 0x18;
/// First 8 bytes of a block-1 header once the generator mask is removed.
const PROOF_PREFIX: [u8; 8] = [0xbb, 0x39, 0xc8, 0x17, 0x02, 0xd5, 0x49, 0x5b];
/// The class stream of a real save is 16.5 MB; anything past this is not a save.
const MAX_STREAM: usize = 256 * 1024 * 1024;

/// One step of the generator; the output is also the next state.
fn step(x: u64) -> u64 {
    let mut z = x.wrapping_add(GOLDEN);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Sizes (in 0x4000 units) of the real blocks for a decrypted length, and the generator state
/// after the size loop, which runs once per potential block, not only the real ones.
fn plan_blocks(decrypted_len: usize) -> (Vec<usize>, u64) {
    let potential = decrypted_len.div_ceil(UNIT);
    let mut x = SIZE_SEED;
    let mut real = Vec::new();
    let mut remaining = decrypted_len;
    let mut done = false;
    for _ in 0..potential {
        let size = (x & 7) as usize + 1;
        x = step(x);
        if !done {
            real.push(size);
            if remaining <= size * UNIT {
                done = true;
            } else {
                remaining -= size * UNIT;
            }
        }
    }
    (real, x)
}

fn aes_ofb(key: [u8; 16], iv: [u8; 16], data: &mut [u8]) {
    let cipher = Aes128::new(&GenericArray::from(key));
    let mut ks = GenericArray::from(iv);
    for chunk in data.chunks_mut(16) {
        cipher.encrypt_block(&mut ks);
        for (d, k) in chunk.iter_mut().zip(ks.iter()) {
            *d ^= k;
        }
    }
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&b[at..at + 8]);
    u64::from_le_bytes(a)
}

/// Decrypts and inflates a save file. `steam_id64` is the full 64-bit Steam ID. Returns the class
/// stream.
pub fn open(file: &[u8], steam_id64: u64) -> Result<Vec<u8>, SaveError> {
    if file.len() < FILE_HEADER + TAIL || &file[..4] != b"DSSS" {
        return Err(SaveError::NotASave("missing DSSS header"));
    }
    if u32_at(file, 4) != 2 || u32_at(file, 8) != FLAGS {
        return Err(SaveError::NotASave("unknown version or flags"));
    }
    let n = file.len();
    let dec_len = u64_at(file, n - 12);
    if dec_len == 0 || dec_len > file.len() as u64 {
        return Err(SaveError::Corrupt("decrypted length is out of range"));
    }
    let dec_len = dec_len as usize;

    let (sizes, mut x) = plan_blocks(dec_len);
    let blocks_len: usize = sizes.iter().map(|s| BLOCK_HEADER + s * UNIT).sum();
    if FILE_HEADER + blocks_len + TAIL != n {
        return Err(SaveError::Corrupt("block sizes do not add up to the file"));
    }
    x = x.wrapping_add(!steam_id64);

    // Cheap check of the Steam ID: the first block's header, with the generator mask removed,
    // starts with a known value. The mask is the low bytes of 8 outputs after the 16 key draws.
    let mut probe = x;
    for _ in 0..16 {
        probe = step(probe);
    }
    for want in PROOF_PREFIX
        .iter()
        .enumerate()
        .map(|(i, w)| (FILE_HEADER + i, w))
    {
        probe = step(probe);
        if file[want.0] ^ probe as u8 != *want.1 {
            return Err(SaveError::WrongSteamId);
        }
    }

    let mut inner = Vec::with_capacity(dec_len);
    let mut remaining = dec_len;
    let mut pos = FILE_HEADER;
    for size in sizes {
        let block_bytes = size * UNIT;
        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        for j in 0..16 {
            x = step(x);
            key[j] = x as u8;
            iv[j] = (x >> 8) as u8;
        }
        for _ in 0..BLOCK_HEADER {
            x = step(x);
        }
        let start = pos + BLOCK_HEADER;
        let take = remaining.min(block_bytes);
        let mut data = file[start..start + block_bytes].to_vec();
        aes_ofb(key, iv, &mut data);
        inner.extend_from_slice(&data[..take]);
        remaining -= take;
        pos = start + block_bytes;
    }
    inflate(&inner)
}

/// The wrapper inside the encryption: `u64 compressed+0x10`, `u32 1`, `u32 compressed`,
/// `u64 decompressed`, then a raw deflate stream.
fn inflate(inner: &[u8]) -> Result<Vec<u8>, SaveError> {
    if inner.len() < 0x18 {
        return Err(SaveError::Corrupt("inner wrapper is too short"));
    }
    let comp = u32_at(inner, 12) as usize;
    let dec = u64_at(inner, 16);
    if u64_at(inner, 0) != comp as u64 + 0x10 || u32_at(inner, 8) != 1 {
        return Err(SaveError::Corrupt("inner wrapper sizes disagree"));
    }
    if comp > inner.len() - 0x18 || dec == 0 || dec > MAX_STREAM as u64 {
        return Err(SaveError::Corrupt("inner wrapper sizes are out of range"));
    }
    let out =
        miniz_oxide::inflate::decompress_to_vec_with_limit(&inner[0x18..0x18 + comp], dec as usize)
            .map_err(|_| SaveError::Corrupt("the compressed data does not inflate"))?;
    if out.len() as u64 != dec {
        return Err(SaveError::Corrupt("inflated size differs from the header"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_sizes_start_as_the_notes_say() {
        // "For this seed the sizes begin 6, 6, 2, 2, 5, 1, 1, 5."
        let (sizes, _) = plan_blocks(8 * UNIT * 8);
        assert_eq!(&sizes[..2], &[6, 6]);
        let mut x = SIZE_SEED;
        let seq: Vec<u64> = (0..8)
            .map(|_| {
                let s = (x & 7) + 1;
                x = step(x);
                s
            })
            .collect();
        assert_eq!(seq, [6, 6, 2, 2, 5, 1, 1, 5]);
    }

    #[test]
    fn rejects_short_and_foreign_files_without_panicking() {
        assert!(matches!(open(&[], 1), Err(SaveError::NotASave(_))));
        assert!(matches!(open(&[0; 400], 1), Err(SaveError::NotASave(_))));
        let mut f = vec![0u8; 400];
        f[..4].copy_from_slice(b"DSSS");
        f[4] = 2;
        f[8] = 0x18;
        assert!(matches!(open(&f, 1), Err(SaveError::Corrupt(_))));
    }
}
