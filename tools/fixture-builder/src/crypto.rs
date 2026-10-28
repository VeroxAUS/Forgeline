// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The save container, written from the notes in `docs/research.md` (Phase 1 step 2 and its
//! follow-up): `DSSS` header, Mandarin-style block encryption keyed by the Steam ID, a raw-deflate
//! wrapper, a tail with a length and a murmur3 checksum.
//!
//! This is the writing direction. It is deliberately a separate implementation from the reader that
//! step 4 builds, so the two check each other. Block headers carry the first 8 bytes of the key/IV
//! proof only (the rest is zero) and the per-block checksum is left zero: the reader ignores both.

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;

const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
/// Per-game starting state of the block-size sequence (Wilds).
const SIZE_SEED: u64 = 0x7A36_9552_5526_6CED;
const UNIT: usize = 0x4000;
const BLOCK_HEADER: usize = 0x210;
const BLOB_LEN: usize = 0x80;
/// First 8 bytes of the key/IV proof, which the notes give as the check value for the generator.
pub const PROOF_PREFIX: [u8; 8] = [0xbb, 0x39, 0xc8, 0x17, 0x02, 0xd5, 0x49, 0x5b];
/// Header flags: Deflate (0x8) plus Mandarin (0x10).
const FLAGS: u32 = 0x18;
pub const STEAM_ID_BASE: u64 = 0x0110_0001_0000_0000;

/// One step of the generator. The output is also the next state.
pub fn step(x: u64) -> u64 {
    let mut z = x.wrapping_add(GOLDEN);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// MurmurHash3, x86 32-bit variant.
pub fn murmur3_32(data: &[u8], seed: u32) -> u32 {
    const C1: u32 = 0xcc9e_2d51;
    const C2: u32 = 0x1b87_3593;
    let mut h = seed;
    let mut chunks = data.chunks_exact(4);
    for c in &mut chunks {
        let mut k = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
        h ^= k;
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe654_6b64);
    }
    let tail = chunks.remainder();
    if !tail.is_empty() {
        let mut k = 0u32;
        for (i, b) in tail.iter().enumerate() {
            k |= u32::from(*b) << (8 * i);
        }
        k = k.wrapping_mul(C1).rotate_left(15).wrapping_mul(C2);
        h ^= k;
    }
    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// Hash used for class and field names: murmur3 with seed 0xFFFFFFFF over the name's bytes.
pub fn name_hash(name: &str) -> u32 {
    murmur3_32(name.as_bytes(), 0xFFFF_FFFF)
}

/// The deflate wrapper that sits inside the encryption: sizes, then a raw deflate stream.
pub fn deflate_wrapper(stream: &[u8]) -> Vec<u8> {
    let compressed = miniz_oxide::deflate::compress_to_vec(stream, 6);
    let mut out = Vec::with_capacity(compressed.len() + 0x18);
    out.extend_from_slice(&(compressed.len() as u64 + 0x10).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(stream.len() as u64).to_le_bytes());
    out.extend_from_slice(&compressed);
    out
}

/// Block sizes (in 0x4000 units) of the real blocks for `decrypted_len`, and the generator state
/// after the size loop, which covers every potential block (not only the real ones).
pub fn plan_blocks(decrypted_len: usize) -> (Vec<usize>, u64) {
    let potential = decrypted_len.div_ceil(UNIT);
    let mut x = SIZE_SEED;
    let mut sizes = Vec::with_capacity(potential);
    for _ in 0..potential {
        sizes.push((x & 7) as usize + 1);
        x = step(x);
    }
    let mut real = Vec::new();
    let mut remaining = decrypted_len;
    for &s in &sizes {
        real.push(s);
        if remaining <= s * UNIT {
            break;
        }
        remaining -= s * UNIT;
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

/// Builds a complete save file from the deflate wrapper (`inner`) for a 64-bit Steam ID.
pub fn build_file(inner: &[u8], steam_id64: u64) -> Vec<u8> {
    let (sizes, mut x) = plan_blocks(inner.len());
    x = x.wrapping_add(!steam_id64);

    let mut file = Vec::new();
    file.extend_from_slice(b"DSSS");
    file.extend_from_slice(&2u32.to_le_bytes());
    file.extend_from_slice(&FLAGS.to_le_bytes());
    file.extend_from_slice(&[0; 4]);

    let mut remaining = inner.len();
    let mut at = 0;
    for size in sizes {
        let block_bytes = size * UNIT;
        let take = remaining.min(block_bytes);

        let mut key = [0u8; 16];
        let mut iv = [0u8; 16];
        for j in 0..16 {
            x = step(x);
            key[j] = x as u8;
            iv[j] = (x >> 8) as u8;
        }
        let mut header = [0u8; BLOCK_HEADER];
        header[..8].copy_from_slice(&PROOF_PREFIX);
        for h in header.iter_mut() {
            x = step(x);
            *h ^= x as u8;
        }
        let mut data = vec![0u8; block_bytes];
        data[..take].copy_from_slice(&inner[at..at + take]);
        aes_ofb(key, iv, &mut data);

        file.extend_from_slice(&header);
        file.extend_from_slice(&data);
        at += take;
        remaining -= take;
    }

    // Tail: the 0x80-byte blob (an ElGamal-style value the reader ignores; a fixed pattern here),
    // the decrypted length, then the checksum of everything before it.
    file.extend((0..BLOB_LEN).map(|i| (i as u8).wrapping_mul(7).wrapping_add(3)));
    file.extend_from_slice(&(inner.len() as u64).to_le_bytes());
    let sum = murmur3_32(&file, 0xFFFF_FFFF);
    file.extend_from_slice(&sum.to_le_bytes());
    file
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Reads a file back by following the recipe in the notes step by step. Returns the deflate
    /// wrapper's inner bytes. Panics with a clear message if a check fails.
    pub fn decrypt(file: &[u8], steam_id64: u64) -> Vec<u8> {
        assert_eq!(&file[..4], b"DSSS");
        assert_eq!(u32::from_le_bytes(file[4..8].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(file[8..12].try_into().unwrap()), FLAGS);
        let n = file.len();
        let sum = murmur3_32(&file[..n - 4], 0xFFFF_FFFF);
        assert_eq!(
            sum,
            u32::from_le_bytes(file[n - 4..].try_into().unwrap()),
            "file checksum"
        );
        let dec_len = u64::from_le_bytes(file[n - 12..n - 4].try_into().unwrap()) as usize;

        let (sizes, mut x) = plan_blocks(dec_len);
        x = x.wrapping_add(!steam_id64);

        // The cheap oracle from the notes: the first block's header, with the generator mask
        // removed, must start with the proof prefix. (Mask = low bytes of 8 outputs after 16 draws.)
        let mut probe = x;
        for _ in 0..16 {
            probe = step(probe);
        }
        let mut mask = [0u8; 8];
        for m in &mut mask {
            probe = step(probe);
            *m = probe as u8;
        }
        let got: Vec<u8> = file[0x10..0x18]
            .iter()
            .zip(mask)
            .map(|(a, b)| a ^ b)
            .collect();
        assert_eq!(got, PROOF_PREFIX, "oracle value");

        let mut out = Vec::new();
        let mut remaining = dec_len;
        let mut pos = 0x10;
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
            let mut data = file[pos + BLOCK_HEADER..pos + BLOCK_HEADER + block_bytes].to_vec();
            aes_ofb(key, iv, &mut data);
            out.extend_from_slice(&data[..remaining.min(block_bytes)]);
            remaining = remaining.saturating_sub(block_bytes);
            pos += BLOCK_HEADER + block_bytes;
        }
        assert_eq!(pos + BLOB_LEN + 12, n, "blocks run exactly up to the tail");
        assert_eq!(out.len(), dec_len);
        out
    }

    pub fn inflate(inner: &[u8]) -> Vec<u8> {
        let comp = u32::from_le_bytes(inner[12..16].try_into().unwrap()) as usize;
        let dec = u64::from_le_bytes(inner[16..24].try_into().unwrap()) as usize;
        assert_eq!(
            u64::from_le_bytes(inner[..8].try_into().unwrap()) as usize,
            comp + 0x10
        );
        assert_eq!(u32::from_le_bytes(inner[8..12].try_into().unwrap()), 1);
        let out =
            miniz_oxide::inflate::decompress_to_vec(&inner[0x18..0x18 + comp]).expect("inflate");
        assert_eq!(out.len(), dec);
        out
    }

    #[test]
    fn murmur3_matches_known_vectors() {
        assert_eq!(murmur3_32(b"", 0), 0);
        assert_eq!(murmur3_32(b"", 1), 0x514e_28b7);
        assert_eq!(murmur3_32(b"", 0xffff_ffff), 0x81f1_6f39);
        assert_eq!(murmur3_32(b"test", 0), 0xba6b_d213);
        assert_eq!(murmur3_32(b"Hello, world!", 0), 0xc036_3e43);
        assert_eq!(
            murmur3_32(b"The quick brown fox jumps over the lazy dog", 0),
            0x2e4f_f723
        );
    }

    #[test]
    fn size_sequence_starts_as_the_notes_say() {
        // "For this seed the sizes begin 6, 6, 2, 2, 5, 1, 1, 5."
        let mut x = SIZE_SEED;
        let sizes: Vec<u64> = (0..8)
            .map(|_| {
                let s = (x & 7) + 1;
                x = step(x);
                s
            })
            .collect();
        assert_eq!(sizes, [6, 6, 2, 2, 5, 1, 1, 5]);
    }

    #[test]
    fn aes_matches_the_fips_197_vector() {
        // FIPS-197 appendix C.1 block, run as OFB over one block: output = plaintext ^ AES(iv).
        let key: [u8; 16] = core::array::from_fn(|i| i as u8);
        let pt: [u8; 16] = core::array::from_fn(|i| (i as u8) * 0x11);
        let mut data = pt;
        aes_ofb(key, pt, &mut data);
        let aes_of_pt: Vec<u8> = data.iter().zip(pt).map(|(a, b)| a ^ b).collect();
        let want: [u8; 16] = [
            0x69, 0xc4, 0xe0, 0xd8, 0x6a, 0x7b, 0x04, 0x30, 0xd8, 0xcd, 0xb7, 0x80, 0x70, 0xb4,
            0xc5, 0x5a,
        ];
        assert_eq!(aes_of_pt, want);
    }

    #[test]
    fn a_file_round_trips_through_the_recipe() {
        let id = STEAM_ID_BASE + 4242;
        let stream: Vec<u8> = (0..70_000u32)
            .flat_map(|i| (i % 251).to_le_bytes())
            .collect();
        let inner = deflate_wrapper(&stream);
        let file = build_file(&inner, id);
        assert_eq!(decrypt(&file, id), inner);
        assert_eq!(inflate(&decrypt(&file, id)), stream);
    }

    #[test]
    #[should_panic(expected = "oracle value")]
    fn a_wrong_steam_id_fails_the_oracle() {
        let inner = deflate_wrapper(b"hello");
        let file = build_file(&inner, STEAM_ID_BASE + 1);
        decrypt(&file, STEAM_ID_BASE + 2);
    }
}
