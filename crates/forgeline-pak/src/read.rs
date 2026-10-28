// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reads one entry out of a pack: stored, deflate or zstd, including chunked entries. Never reads
//! more than the entry itself and never writes.

use crate::index::{EntryRef, PackSet, ATTR_CHUNKED};
use crate::PakError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// No file we need is anywhere near this size; a bigger number is a corrupt table asking for
/// gigabytes.
pub const MAX_ENTRY: u32 = 256 * 1024 * 1024;

fn corrupt(what: &str) -> PakError {
    PakError::Corrupt(what.to_string())
}

fn read_at(f: &mut File, offset: u64, len: usize) -> Result<Vec<u8>, PakError> {
    f.seek(SeekFrom::Start(offset))
        .map_err(|e| PakError::Io("seek".into(), e))?;
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)
        .map_err(|e| PakError::Io("read".into(), e))?;
    Ok(buf)
}

fn zstd_decode(raw: &[u8], limit: usize) -> Result<Vec<u8>, PakError> {
    let mut out = Vec::new();
    let dec = zstd::stream::read::Decoder::new(raw).map_err(|_| corrupt("zstd header"))?;
    dec.take(limit as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|_| corrupt("zstd data"))?;
    if out.len() > limit {
        return Err(corrupt("zstd data longer than the entry says"));
    }
    Ok(out)
}

impl PackSet {
    /// The unpacked bytes of an entry.
    pub fn read(&self, e: &EntryRef) -> Result<Vec<u8>, PakError> {
        if e.size > MAX_ENTRY || e.stored > MAX_ENTRY {
            return Err(corrupt("entry size is over the limit"));
        }
        let pack = self
            .packs
            .get(e.pack())
            .ok_or_else(|| corrupt("entry points at a pack that is not indexed"))?;
        let mut f = File::open(&pack.path)
            .map_err(|err| PakError::Io(pack.path.display().to_string(), err))?;
        let size = e.size as usize;

        if e.attr & ATTR_CHUNKED != 0 {
            let chunks = pack
                .chunks
                .as_ref()
                .ok_or_else(|| corrupt("chunked entry in a pack without a chunk table"))?;
            // The size is the unpacked size; with none, the stored size stands in for it.
            let total = if size != 0 { size } else { e.stored as usize };
            let mut out = Vec::with_capacity(total);
            let mut i = e.offset as usize;
            while out.len() < total {
                let &(start, len) = chunks
                    .list
                    .get(i)
                    .ok_or_else(|| corrupt("chunk index out of range"))?;
                if len > MAX_ENTRY {
                    return Err(corrupt("chunk size is over the limit"));
                }
                let raw = read_at(&mut f, start, len as usize)?;
                if len == chunks.block {
                    out.extend_from_slice(&raw);
                } else {
                    out.extend(zstd_decode(&raw, chunks.block.max(1) as usize)?);
                }
                i += 1;
            }
            out.truncate(total);
            return Ok(out);
        }

        let raw = read_at(&mut f, e.offset, e.stored as usize)?;
        let limit = if size != 0 { size } else { MAX_ENTRY as usize };
        let out = match e.attr & 0xF {
            0 => raw,
            1 => miniz_oxide::inflate::decompress_to_vec_with_limit(&raw, limit)
                .map_err(|_| corrupt("deflate data"))?,
            2 => zstd_decode(&raw, limit)?,
            _ => return Err(PakError::Unsupported("compression type".to_string())),
        };
        if size != 0 && out.len() != size {
            return Err(corrupt("unpacked size differs from the table"));
        }
        Ok(out)
    }
}
