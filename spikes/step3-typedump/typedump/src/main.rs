//! Throwaway spike (Phase 1 step 3). Read-only: reads a save COPY and the game .exe on disk.
//! Written from docs/research.md notes only.
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

fn murmur3(data: &[u8], seed: u32) -> u32 {
    let (c1, c2) = (0xcc9e2d51u32, 0x1b873593u32);
    let mut h = seed;
    let mut chunks = data.chunks_exact(4);
    for c in &mut chunks {
        let mut k = u32::from_le_bytes(c.try_into().unwrap());
        k = k.wrapping_mul(c1).rotate_left(15).wrapping_mul(c2);
        h ^= k;
        h = h.rotate_left(13).wrapping_mul(5).wrapping_add(0xe6546b64);
    }
    let t = chunks.remainder();
    let mut k = 0u32;
    for (i, b) in t.iter().enumerate() {
        k |= (*b as u32) << (8 * i);
    }
    if !t.is_empty() {
        k = k.wrapping_mul(c1).rotate_left(15).wrapping_mul(c2);
        h ^= k;
    }
    h ^= data.len() as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^ (h >> 16)
}

#[allow(dead_code)]
struct Sm(u64);
impl Sm {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

fn modes(mode: u32, key: [u8; 16], iv: [u8; 16], data: &mut [u8]) {
    if mode == 0 { return ofb(key, iv, data); }
    let c = Aes128::new(&key.into());
    let mut prev = iv;
    let mut ctr = u128::from_be_bytes(iv);
    let orig = data.to_vec();
    for (i, ch) in data.chunks_mut(16).enumerate() {
        let mut b = if mode == 1 { ctr.to_be_bytes() } else if mode == 3 { ctr.to_le_bytes() } else { prev }.into();
        c.encrypt_block(&mut b);
        let ks: [u8; 16] = b.into();
        for (d, k) in ch.iter_mut().zip(ks.iter()) { *d ^= k; }
        ctr = ctr.wrapping_add(1);
        let st = i * 16;
        prev = orig[st..(st + 16).min(orig.len())].iter().copied().chain(std::iter::repeat(0)).take(16).collect::<Vec<_>>().try_into().unwrap();
    }
}

fn ofb(key: [u8; 16], iv: [u8; 16], data: &mut [u8]) {
    let c = Aes128::new(&key.into());
    let mut s = iv;
    for ch in data.chunks_mut(16) {
        let mut b = s.into();
        c.encrypt_block(&mut b);
        s = b.into();
        for (d, k) in ch.iter_mut().zip(s.iter()) {
            *d ^= k;
        }
    }
}

fn step(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Recipe from docs/research.md step 2 follow-up. Oracle: checked before any AES.
fn decrypt(f: &[u8], steam_id: u64) -> Option<Vec<u8>> {
    let n = f.len();
    let dec_len = u64::from_le_bytes(f[n - 12..n - 4].try_into().unwrap()) as usize;
    let enc = &f[0x10..n - 4];
    let mut x = 0x7A36955255266CEDu64;
    let mut sizes = vec![];
    for _ in 0..dec_len.div_ceil(0x4000) {
        sizes.push(((x & 7) as usize + 1) * 0x4000);
        x = step(x);
    }
    x = x.wrapping_add(!steam_id);
    {
        let mut t = x;
        for _ in 0..16 { t = step(t); }
        let mut m = [0u8; 8];
        for b in m.iter_mut() { t = step(t); *b = t as u8; }
        let o: Vec<u8> = enc[..8].iter().zip(m).map(|(a, b)| a ^ b).collect();
        eprintln!("oracle {:02x?} (want bb39c81702d5495b)", o);
    }
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut remaining = dec_len;
    for sz in sizes {
        let mut k = [0u8; 16];
        let mut iv = [0u8; 16];
        for i in 0..16 {
            x = step(x);
            k[i] = x as u8;
            iv[i] = (x >> 8) as u8;
        }
        for _ in 0..0x210 { x = step(x); }
        let mut d = enc.get(pos + 0x210..pos + 0x210 + sz)?.to_vec();
        ofb(k, iv, &mut d);
        let take = remaining.min(sz);
        out.extend_from_slice(&d[..take]);
        pos += 0x210 + sz;
        if remaining <= sz { break; }
        remaining -= sz;
    }
    Some(out)
}

fn inflate(d: &[u8]) -> Option<Vec<u8>> {
    let csz = u32::from_le_bytes(d.get(12..16)?.try_into().ok()?) as usize;
    let usz = u64::from_le_bytes(d.get(16..24)?.try_into().ok()?) as usize;
    let body = d.get(24..24 + csz)?;
    let mut o = Vec::with_capacity(usz);
    flate2::read::DeflateDecoder::new(body).read_to_end(&mut o).ok()?;
    eprintln!("inflated {} (header says {})", o.len(), usz);
    Some(o)
}

struct W<'a> {
    b: &'a [u8],
    p: usize,
    base: usize,
    extra: u64,
    classes: BTreeMap<u32, (u64, BTreeMap<u32, (i32, u64)>)>, // class -> (count, field -> (type,count))
}
type R<T> = Result<T, String>;
impl<'a> W<'a> {
    fn al(&mut self, a: usize) {
        let r = (self.p + self.base) % a;
        if r != 0 {
            self.p += a - r;
        }
    }
    fn u32(&mut self) -> R<u32> {
        let v = self.b.get(self.p..self.p + 4).ok_or(format!("eof at {}", self.p))?;
        self.p += 4;
        Ok(u32::from_le_bytes(v.try_into().unwrap()))
    }
    fn class(&mut self, depth: u32) -> R<u32> {
        self.al(4);
        let cnt = self.u32()?;
        let ch = self.u32()?;
        if cnt > 10000 {
            return Err(format!("bad field count {cnt} at {}", self.p));
        }
        let e = self.classes.entry(ch).or_default();
        e.0 += 1;
        for _ in 0..cnt {
            self.al(4);
            let fh = self.u32()?;
            let t = self.u32()? as i32;
            let e = self.classes.get_mut(&ch).unwrap().1.entry(fh).or_insert((t, 0));
            e.1 += 1;
            self.value(t, depth)?;
            self.al(4);
        }
        Ok(ch)
    }
    fn value(&mut self, t: i32, depth: u32) -> R<()> {
        match t {
            1..=14 | 16 => {
                self.al(4);
                let sz = self.u32()? as usize;
                if sz > 0x100000 {
                    return Err(format!("bad size {sz} at {}", self.p));
                }
                if t != 16 && [1, 2, 4, 8].contains(&sz) {
                    self.al(sz);
                }
                if t == 16 && sz == 16 {
                    self.al(16); // observed: 16-byte structs are 16-aligned
                }
                self.p += sz;
            }
            15 => {
                self.al(4);
                let n = self.u32()? as usize;
                self.p += n * 2;
            }
            17 => {
                self.class(depth + 1)?;
            }
            -1 => {
                self.al(4);
                let mt = self.u32()? as i32;
                let _ms = self.u32()?;
                let n = self.u32()? as usize;
                let kind = self.u32()?;
                if n > 10_000_000 {
                    return Err(format!("bad array {n} at {}", self.p));
                }
                if kind == 1 {
                    if self.b.get(self.p..self.p + 4) == Some(&0xFFEEFFEEu32.to_le_bytes()) {
                        self.p += 4;
                        self.p += 4 * n;
                    }
                    for _ in 0..n {
                        self.class(depth + 1)?;
                    }
                } else {
                    if mt == 15 || mt == 17 || mt == -1 {
                        for _ in 0..n {
                            self.value(mt, depth)?;
                        }
                    } else {
                        // observed: scalar/struct array elements are packed, no per-element size prefix
                        if _ms == 16 { self.al(16); }
                        self.p += n * _ms as usize;
                    }
                }
                self.al(4);
            }
            _ => return Err(format!("bad type {t} at {}", self.p)),
        }
        Ok(())
    }
}

fn walk(d: &[u8], base: usize) -> (W<'_>, Option<String>) {
    let mut w = W { b: d, p: 0, base, extra: 0, classes: BTreeMap::new() };
    let mut err = None;
    while w.p + 7 < d.len() {
        if let Err(e) = w.u32().and_then(|_| w.class(0)) {
            err = Some(e);
            break;
        }
    }
    (w, err)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(|s| s.as_str()) {
        Some("walk") => {
            // walk <save copy> <steam id>   (refuses the live save folder)
            if a[2].to_lowercase().contains("win64_save") {
                panic!("refusing live save folder; use a copy");
            }
            let f = std::fs::read(&a[2]).unwrap();
            let id: u64 = a[3].parse().unwrap();
            let d = decrypt(&f, id).expect("decrypt");
            let p = inflate(&d).expect("inflate");
            std::fs::write(&a[4], &p).unwrap();
            eprintln!("OK");
        }
        Some("names") => {
            // names <exe tsv> <classes tsv> <out tsv>: rebuild dotted class names as namespace + "." + short name
            let rd = |p: &str| std::fs::read_to_string(p).unwrap();
            let tsv = rd(&a[2]);
            let want: std::collections::HashSet<u32> =
                rd(&a[3]).lines().filter_map(|l| u32::from_str_radix(l.split('\t').next()?, 16).ok()).collect();
            let ident = |s: &str| s.len() <= 80 && s.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            let mut ns: Vec<&str> = vec![];
            let mut nm: Vec<&str> = vec![];
            for l in tsv.lines() {
                let Some((_, n)) = l.split_once('\t') else { continue };
                if (n.starts_with("app.") || n.starts_with("via.") || n == "app" || n == "via") && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.') { ns.push(n); }
                if ident(n) { nm.push(n); }
            }
            eprintln!("{} namespaces, {} names", ns.len(), nm.len());
            let mut out = String::new();
            let mut buf = Vec::new();
            for n in &ns {
                for m in &nm {
                    buf.clear();
                    buf.extend_from_slice(n.as_bytes());
                    buf.push(b'.');
                    buf.extend_from_slice(m.as_bytes());
                    let h = murmur3(&buf, 0xFFFFFFFF);
                    if want.contains(&h) { out.push_str(&format!("{h:08x}	{n}.{m}
")); }
                }
            }
            // also plain strings (no namespace)
            for m in tsv.lines().filter_map(|l| l.split_once('\t')) {
                let h = u32::from_str_radix(m.0, 16).unwrap();
                if want.contains(&h) { out.push_str(&format!("{h:08x}	{}
", m.1)); }
            }
            std::fs::write(&a[4], out).unwrap();
        }
        Some("classes") => {
            // classes <payload> <exe hashes tsv>
            let p = std::fs::read(&a[2]).unwrap();
            let names: HashMap<u32, String> = std::fs::read_to_string(&a[3])
                .map(|s| {
                    s.lines()
                        .filter_map(|l| {
                            let (h, n) = l.split_once('\t')?;
                            Some((u32::from_str_radix(h, 16).ok()?, n.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut best = None;
            for base in 0..8 {
                let (w, e) = walk(&p, base);
                eprintln!("base {base}: {} classes, err {:?}, pos {}/{} extra {}", w.classes.len(), e, w.p, p.len(), w.extra);
                if e.is_none() {
                    best = Some(base);
                    for (h, (c, fs)) in &w.classes {
                        let nm = names.get(h).map(|s| s.as_str()).unwrap_or("?");
                        let known = fs.keys().filter(|k| names.contains_key(k)).count();
                        println!("{h:08x}	{c}	{nm}	fields {} named {}", fs.len(), known);
                    }
                    break;
                }
            }
            eprintln!("best {:?}", best);
        }
        Some("exe") => {
            // exe <exe> <out tsv>: hash every identifier-like ASCII string (and UTF-16) found
            let f = std::fs::File::open(&a[2]).unwrap();
            let mut r = std::io::BufReader::with_capacity(1 << 24, f);
            let mut out: HashMap<u32, String> = HashMap::new();
            let mut cur: Vec<u8> = vec![];
            let mut buf = vec![0u8; 1 << 24];
            let mut total = 0usize;
            let mut flush = |cur: &mut Vec<u8>, out: &mut HashMap<u32, String>| {
                if cur.len() >= 3 && cur.len() <= 200 {
                    let s = std::str::from_utf8(cur).unwrap();
                    out.entry(murmur3(cur, 0xFFFFFFFF)).or_insert_with(|| s.to_string());
                }
                cur.clear();
            };
            loop {
                let n = r.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                total += n;
                for &b in &buf[..n] {
                    if (0x21..0x7f).contains(&b) {
                        cur.push(b);
                    } else {
                        flush(&mut cur, &mut out);
                    }
                }
            }
            eprintln!("scanned {total} bytes, {} distinct strings", out.len());
            let s: String = out.iter().map(|(h, n)| format!("{h:08x}	{n}\n")).collect();
            std::fs::write(&a[3], s).unwrap();
        }
        _ => eprintln!("usage: typedump walk|classes|exe ..."),
    }
}
