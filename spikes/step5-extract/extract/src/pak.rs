//! Spike pak reader (KPKA v4.x, entry table encrypted). Written from the format facts in
//! docs/research.md (credit: eigeen/ree-pak-rs, MIT, for the format and the table key constants).
use num_bigint::BigUint;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// Pak table modulus constant, copied from eigeen/ree-pak-rs (MIT licence, https://github.com/eigeen/ree-pak-rs)
// with credit. See THIRD-PARTY-NOTICES.md on main.
const MODULUS: [u8; 129] = [
    0x7D, 0x0B, 0xF8, 0xC1, 0x7C, 0x23, 0xFD, 0x3B, 0xD4, 0x75, 0x16, 0xD2, 0x33, 0x21, 0xD8, 0x10, 0x71, 0xF9, 0x7C,
    0xD1, 0x34, 0x93, 0xBA, 0x77, 0x26, 0xFC, 0xAB, 0x2C, 0xEE, 0xDA, 0xD9, 0x1C, 0x89, 0xE7, 0x29, 0x7B, 0xDD, 0x8A,
    0xAE, 0x50, 0x39, 0xB6, 0x01, 0x6D, 0x21, 0x89, 0x5D, 0xA5, 0xA1, 0x3E, 0xA2, 0xC0, 0x8C, 0x93, 0x13, 0x36, 0x65,
    0xEB, 0xE8, 0xDF, 0x06, 0x17, 0x67, 0x96, 0x06, 0x2B, 0xAC, 0x23, 0xED, 0x8C, 0xB7, 0x8B, 0x90, 0xAD, 0xEA, 0x71,
    0xC4, 0x40, 0x44, 0x9D, 0x1C, 0x7B, 0xBA, 0xC4, 0xB6, 0x2D, 0xD6, 0xD2, 0x4B, 0x62, 0xD6, 0x26, 0xFC, 0x74, 0x20,
    0x07, 0xEC, 0xE3, 0x59, 0x9A, 0xE6, 0xAF, 0xB9, 0xA8, 0x35, 0x8B, 0xE0, 0xE8, 0xD3, 0xCD, 0x45, 0x65, 0xB0, 0x91,
    0xC4, 0x95, 0x1B, 0xF3, 0x23, 0x1E, 0xC6, 0x71, 0xCF, 0x3E, 0x35, 0x2D, 0x6B, 0xE3, 0x00,
];

#[derive(Clone, Debug)]
pub struct Entry {
    pub lower: u32,
    pub upper: u32,
    pub offset: u64,
    pub csize: u64,
    pub usize_: u64,
    pub attr: u64,
}

impl Entry {
    pub fn comp(&self) -> u8 { (self.attr & 0xF) as u8 }
    pub fn key(&self) -> u64 { ((self.upper as u64) << 32) | self.lower as u64 }
}

fn decrypt_table(data: &mut [u8], enc_key: &[u8]) {
    let mut k = enc_key.to_vec();
    k.resize(129, 0);
    let n = BigUint::from_bytes_le(&MODULUS);
    let mut key = BigUint::from_bytes_le(&k).modpow(&BigUint::from(0x10001u32), &n).to_bytes_le();
    key.resize(32, 0);
    for (i, b) in data.iter_mut().enumerate() {
        *b ^= (i.wrapping_add(key[i % 32] as usize * key[i % 29] as usize)) as u8;
    }
}

/// Reads the entry table. Returns (feature flags, entries).
pub struct Chunks { pub block: u32, pub list: Vec<(u64, u32)> }

pub fn read_table(f: &mut File) -> Result<(u16, Vec<Entry>, Option<Chunks>), String> {
    let mut h = [0u8; 16];
    f.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    f.read_exact(&mut h).map_err(|e| e.to_string())?;
    if &h[0..4] != b"KPKA" { return Err("bad magic".into()); }
    let (major, minor) = (h[4], h[5]);
    if major != 4 { return Err(format!("unsupported version {major}.{minor}")); }
    let feat = u16::from_le_bytes([h[6], h[7]]);
    if feat & !0x28 != 0 { return Err(format!("unsupported feature flags {feat:#x}")); }
    let total = u32::from_le_bytes(h[8..12].try_into().unwrap()) as usize;
    let mut t = vec![0u8; total * 48];
    f.read_exact(&mut t).map_err(|e| e.to_string())?;
    if feat & 0x08 != 0 {
        let mut key = [0u8; 128];
        f.read_exact(&mut key).map_err(|e| e.to_string())?;
        decrypt_table(&mut t, &key);
    }
    let r64 = |b: &[u8]| u64::from_le_bytes(b.try_into().unwrap());
    let v = t.chunks_exact(48).map(|c| Entry {
        lower: u32::from_le_bytes(c[0..4].try_into().unwrap()),
        upper: u32::from_le_bytes(c[4..8].try_into().unwrap()),
        offset: r64(&c[8..16]), csize: r64(&c[16..24]), usize_: r64(&c[24..32]), attr: r64(&c[32..40]),
    }).collect();
    let ch = if feat & 0x20 != 0 {
        let mut hb = [0u8; 8];
        f.read_exact(&mut hb).map_err(|e| e.to_string())?;
        let (block, count) = (u32::from_le_bytes(hb[0..4].try_into().unwrap()), u32::from_le_bytes(hb[4..8].try_into().unwrap()) as usize);
        let mut raw = vec![0u8; count * 8];
        f.read_exact(&mut raw).map_err(|e| e.to_string())?;
        // start offsets are u32 and wrap at 4 GiB; rebuild the high bits from monotonic growth
        let (mut high, mut prev) = (0u64, u32::from_le_bytes(raw[0..4].try_into().unwrap()));
        let list = raw.chunks_exact(8).map(|c| {
            let lo = u32::from_le_bytes(c[0..4].try_into().unwrap());
            if lo < prev { high += 1 << 32; }
            prev = lo;
            (high | lo as u64, u32::from_le_bytes(c[4..8].try_into().unwrap()) >> 10)
        }).collect();
        Some(Chunks { block, list })
    } else { None };
    Ok((feat, v, ch))
}

/// Reads and decompresses a single entry (never more than this entry).
pub fn read_entry(f: &mut File, e: &Entry, ch: Option<&Chunks>) -> Result<Vec<u8>, String> {
    if e.attr & (1 << 24) != 0 {
        let ch = ch.ok_or("chunked entry without chunk table")?;
        let total = if e.usize_ != 0 { e.usize_ } else { e.csize } as usize;
        let mut out = Vec::with_capacity(total);
        let mut i = e.offset as usize;
        while out.len() < total {
            let (start, clen) = *ch.list.get(i).ok_or("chunk index out of range")?;
            f.seek(SeekFrom::Start(start)).map_err(|x| x.to_string())?;
            let mut raw = vec![0u8; clen as usize];
            f.read_exact(&mut raw).map_err(|x| x.to_string())?;
            if clen == ch.block { out.extend_from_slice(&raw); }
            else { out.extend(zstd::stream::decode_all(&raw[..]).map_err(|x| x.to_string())?); }
            i += 1;
        }
        out.truncate(total);
        return Ok(out);
    }
    f.seek(SeekFrom::Start(e.offset)).map_err(|x| x.to_string())?;
    let mut raw = vec![0u8; e.csize as usize];
    f.read_exact(&mut raw).map_err(|x| x.to_string())?;
    match e.comp() {
        0 => Ok(raw),
        1 => { let mut o = Vec::with_capacity(e.usize_ as usize);
               flate2::read::DeflateDecoder::new(&raw[..]).read_to_end(&mut o).map_err(|x| x.to_string())?; Ok(o) }
        2 => zstd::stream::decode_all(&raw[..]).map_err(|x| x.to_string()),
        c => Err(format!("compression {c}")),
    }
}
