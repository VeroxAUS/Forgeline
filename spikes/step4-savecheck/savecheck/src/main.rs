//! Throwaway spike (Phase 1 step 4). Read-only: reads a save COPY. Prints structure and counts only,
//! never hunter IDs or other personal values. Written from docs/research.md notes only.
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

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

/// Recipe from docs/research.md (step 2 follow-up, confirmed in step 3).
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
    Some(o)
}

// ---- class stream tree ----
#[derive(Debug)]
enum V {
    Scalar(i32, u64),       // type code, value (<= 8 bytes)
    Raw(i32, Vec<u8>),      // struct or wide value
    Str(String),
    Class(Box<Node>),
    Arr(i32, u32, Vec<u8>), // member type, member size, packed bytes (scalars/structs)
    Classes(Vec<Node>),
    List(Vec<V>),           // arrays of strings / nested arrays
}
#[derive(Debug, Default)]
struct Node {
    hash: u32,
    fields: Vec<(u32, V)>,
}
impl Node {
    fn get(&self, h: u32) -> Option<&V> { self.fields.iter().find(|f| f.0 == h).map(|f| &f.1) }
}

struct P<'a> { b: &'a [u8], p: usize }
type R<T> = Result<T, String>;
impl<'a> P<'a> {
    fn al(&mut self, a: usize) { let r = self.p % a; if r != 0 { self.p += a - r; } }
    fn u32(&mut self) -> R<u32> {
        let v = self.b.get(self.p..self.p + 4).ok_or(format!("eof at {}", self.p))?;
        self.p += 4;
        Ok(u32::from_le_bytes(v.try_into().unwrap()))
    }
    fn class(&mut self) -> R<Node> {
        self.al(4);
        let cnt = self.u32()?;
        let hash = self.u32()?;
        if cnt > 10000 { return Err(format!("bad field count {cnt}")); }
        let mut n = Node { hash, fields: vec![] };
        for _ in 0..cnt {
            self.al(4);
            let fh = self.u32()?;
            let t = self.u32()? as i32;
            let v = self.value(t)?;
            self.al(4);
            n.fields.push((fh, v));
        }
        Ok(n)
    }
    fn value(&mut self, t: i32) -> R<V> {
        Ok(match t {
            1..=14 | 16 => {
                self.al(4);
                let sz = self.u32()? as usize;
                if sz > 0x100000 { return Err(format!("bad size {sz}")); }
                if t != 16 && [1, 2, 4, 8].contains(&sz) { self.al(sz); }
                if t == 16 && sz == 16 { self.al(16); }
                let d = self.b.get(self.p..self.p + sz).ok_or("eof")?.to_vec();
                self.p += sz;
                if sz <= 8 && t != 16 {
                    let mut a = [0u8; 8];
                    a[..sz].copy_from_slice(&d);
                    V::Scalar(t, u64::from_le_bytes(a))
                } else { V::Raw(t, d) }
            }
            15 => {
                self.al(4);
                let n = self.u32()? as usize;
                let d = self.b.get(self.p..self.p + n * 2).ok_or("eof")?;
                self.p += n * 2;
                V::Str(String::from_utf16_lossy(&d.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()))
            }
            17 => V::Class(Box::new(self.class()?)),
            -1 => {
                self.al(4);
                let mt = self.u32()? as i32;
                let ms = self.u32()?;
                let n = self.u32()? as usize;
                let kind = self.u32()?;
                if n > 10_000_000 { return Err(format!("bad array {n}")); }
                let v = if kind == 1 {
                    if self.b.get(self.p..self.p + 4) == Some(&0xFFEEFFEEu32.to_le_bytes()) { self.p += 4 + 4 * n; }
                    let mut cs = Vec::with_capacity(n);
                    for _ in 0..n { cs.push(self.class()?); }
                    V::Classes(cs)
                } else if mt == 15 || mt == 17 || mt == -1 {
                    let mut l = Vec::with_capacity(n);
                    for _ in 0..n { l.push(self.value(mt)?); }
                    V::List(l)
                } else {
                    if ms == 16 { self.al(16); }
                    let len = n * ms as usize;
                    let d = self.b.get(self.p..self.p + len).ok_or("eof")?.to_vec();
                    self.p += len;
                    V::Arr(mt, ms, d)
                };
                self.al(4);
                v
            }
            _ => return Err(format!("bad type {t} at {}", self.p)),
        })
    }
}

fn parse(d: &[u8]) -> R<Vec<Node>> {
    let mut p = P { b: d, p: 0 };
    let mut out = vec![];
    while p.p + 7 < d.len() {
        p.u32()?;
        out.push(p.class()?);
    }
    Ok(out)
}

struct Names { f: HashMap<u32, String>, c: HashMap<u32, String> }
impl Names {
    fn field(&self, h: u32) -> String { self.f.get(&h).cloned().unwrap_or(format!("#{h:08x}")) }
    fn class(&self, h: u32) -> String { self.c.get(&h).cloned().unwrap_or(format!("#{h:08x}")) }
}

fn load_names(exe_tsv: &str, class_tsv: &str) -> Names {
    let rd = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
    let tab = |s: String, keep: &dyn Fn(&str) -> bool| -> HashMap<u32, String> {
        s.lines().filter_map(|l| { let (h, n) = l.split_once('\t')?; if keep(n) { Some((u32::from_str_radix(h, 16).ok()?, n.to_string())) } else { None } }).collect()
    };
    let mut cls = tab(rd(class_tsv), &|_| true);
    for l in rd(class_tsv).lines() { if let Some((h, n)) = l.split_once('\t') { if n.starts_with("app.") { if let Ok(h) = u32::from_str_radix(h, 16) { cls.insert(h, n.to_string()); } } } }
    Names { f: tab(rd(exe_tsv), &|n| n.starts_with('_')), c: cls }
}

fn kind(v: &V) -> String {
    match v {
        V::Scalar(t, x) => format!("t{t}={}", if *t == 2 { (*x != 0).to_string() } else { x.to_string() }),
        V::Raw(t, d) => format!("t{t} raw{}", d.len()),
        V::Str(s) => format!("str({})", s.chars().count()),
        V::Class(n) => format!("class"),
        V::Arr(t, s, d) => format!("arr<t{t},{s}>[{}]", d.len() / (*s as usize).max(1)),
        V::Classes(c) => format!("classes[{}]", c.len()),
        V::List(l) => format!("list[{}]", l.len()),
    }
}

fn schema(n: &Node, names: &Names, depth: usize, max_depth: usize, show_scalars: bool) {
    let pad = "  ".repeat(depth);
    println!("{pad}{}", names.class(n.hash));
    for (fh, v) in &n.fields {
        let nm = names.field(*fh);
        match v {
            V::Class(c) if depth < max_depth => { println!("{pad}  .{nm}:"); schema(c, names, depth + 2, max_depth, show_scalars); }
            V::Classes(c) if depth < max_depth && !c.is_empty() => { println!("{pad}  .{nm}: classes[{}] (first element:)", c.len()); schema(&c[0], names, depth + 2, max_depth, show_scalars); }
            V::Scalar(..) if !show_scalars => println!("{pad}  .{nm}: {}", kind(v).split('=').next().unwrap()),
            _ => println!("{pad}  .{nm}: {}", kind(v)),
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    // savecheck <save copy> <steam id> <exe_hashes.tsv> <class_names.tsv> <cmd> [args]
    if a[1].to_lowercase().contains("win64_save") { panic!("refusing live save folder; use a copy"); }
    let f = std::fs::read(&a[1]).unwrap();
    let d = decrypt(&f, a[2].parse().unwrap()).expect("decrypt");
    let p = inflate(&d).expect("inflate");
    let top = parse(&p).unwrap_or_else(|e| panic!("parse: {e}"));
    let names = load_names(&a[3], &a[4]);
    match a[5].as_str() {
        "top" => {
            for n in &top { schema(n, &names, 0, 0, false); }
        }
        "slot" => { // slot <index> <depth>: schema of one character slot, no scalar values
            let idx: usize = a[6].parse().unwrap();
            let depth: usize = a[7].parse().unwrap();
            let _ = (idx, depth, BTreeMap::<u8, u8>::new());
            for n in &top { schema(n, &names, 0, depth, false); }
        }
        "scalars" => { // small scalars (types 1-7, 11) of each slot's top level and of the other top-level classes
            for n in &top {
                println!("{}", names.class(n.hash));
                let slots: Vec<&Node> = match n.get(0).map(|_| ()) { _ => vec![] };
                let _ = slots;
                for (fh, v) in &n.fields {
                    if let V::Scalar(t, x) = v { if (1..=7).contains(t) || *t == 11 { println!("  .{} t{t} = {x}", names.field(*fh)); } }
                    if let V::Classes(cs) = v {
                        for (i, c) in cs.iter().enumerate() {
                            print!("  slot {i}:");
                            for (fh2, v2) in &c.fields { if let V::Scalar(t, x) = v2 { if (1..=7).contains(t) || *t == 11 { print!(" {}=t{t}:{x}", names.field(*fh2)); } } }
                            println!();
                        }
                    }
                }
            }
        }
        "fields" => { // fields <slot> <depth> <field name>...: schema of the named top-level fields of one slot
            let idx: usize = a[6].parse().unwrap();
            let depth: usize = a[7].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            for (fh, v) in &slots[idx].fields {
                let nm = names.field(*fh);
                if !a[8..].contains(&nm) { continue; }
                println!("== {nm}");
                match v { V::Class(c) => schema(c, &names, 1, depth, false),
                          V::Classes(cs) => { println!("classes[{}]", cs.len()); if let Some(c) = cs.first() { schema(c, &names, 1, depth, false); } }
                          _ => println!("{}", kind(v)) }
            }
        }
        "stats" => { // stats <slot> <path:Field/Field/...>: per-field value stats of a class array (no strings, no 64-bit ids)
            let idx: usize = a[6].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            let mut cur: &Node = &slots[idx];
            let parts: Vec<&str> = a[7].split('/').collect();
            let mut arr: &Vec<Node> = &vec![];
            let mut found = false;
            for (i, p) in parts.iter().enumerate() {
                let Some((_, v)) = cur.fields.iter().find(|(h, _)| names.field(*h) == *p) else { panic!("no field {p}") };
                match v { V::Class(c) => cur = c, V::Classes(cs) if i == parts.len() - 1 => { arr = cs; found = true; } _ => panic!("bad path") }
            }
            assert!(found);
            println!("{} elements", arr.len());
            let mut cols: BTreeMap<u32, (i32, Vec<u64>)> = BTreeMap::new();
            for n in arr { for (fh, v) in &n.fields { match v {
                V::Scalar(t, x) => cols.entry(*fh).or_insert((*t, vec![])).1.push(*x),
                V::Arr(t, ms, d) => { let e = cols.entry(*fh).or_insert((1000 + *t, vec![])); for c in d.chunks(*ms as usize) { let mut b = [0u8; 8]; b[..c.len().min(8)].copy_from_slice(&c[..c.len().min(8)]); e.1.push(u64::from_le_bytes(b)); } }
                _ => {} } } }
            for (fh, (t, vals)) in &cols {
                let mut h: BTreeMap<u64, usize> = BTreeMap::new();
                for v in vals { *h.entry(*v).or_default() += 1; }
                let nz = vals.iter().filter(|v| **v != 0).count();
                let wide = *t == 10;
                let mut top: Vec<(&u64, &usize)> = h.iter().collect();
                top.sort_by(|x, y| y.1.cmp(x.1));
                let tops: Vec<String> = top.iter().take(4).map(|(v, c)| if wide { format!("*x{c}") } else { format!("{v}x{c}") }).collect();
                println!("{} t{t}: n={} nonzero={nz} distinct={} min={} max={} top: {}", names.field(*fh), vals.len(), h.len(),
                    if wide { "-".into() } else { h.keys().next().unwrap().to_string() }, if wide { "-".into() } else { h.keys().last().unwrap().to_string() }, tops.join(" "));
            }
        }
        "equip" => { // equip <slot>: cross-tab of the non-empty equipment box entries (aggregate counts only)
            let idx: usize = a[6].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            let sc = |n: &Node, h: u32| -> i64 { match n.get(h) { Some(V::Scalar(_, x)) => *x as i64, _ => -1 } };
            let eq = |h: &str| names.f.iter().find(|(_, n)| *n == h).map(|x| *x.0);
            let _ = eq;
            let Some(V::Class(e)) = slots[idx].fields.iter().find(|(h, _)| names.field(*h) == "_Equip").map(|x| &x.1) else { panic!() };
            let Some(V::Classes(bx)) = e.fields.iter().find(|(h, _)| names.field(*h) == "_EquipBox").map(|x| &x.1) else { panic!() };
            let (fc, fi, fb, fl) = (0xd595c544u32, 0xefccb875u32, 0xc5a99e69u32, 0x7545297fu32);
            let mut tab: BTreeMap<(i64, i64), (usize, i64, i64)> = BTreeMap::new();
            for n in bx.iter() {
                let c = sc(n, fc); if c == 141 { continue; }
                let e = tab.entry((c, sc(n, fb))).or_insert((0, i64::MAX, i64::MIN));
                let i = sc(n, fi);
                e.0 += 1; e.1 = e.1.min(i); e.2 = e.2.max(i);
            }
            for ((c, b), (n, lo, hi)) in &tab { println!("cat {c} sub {b}: {n} entries, id {lo}..{hi}"); }
            let _ = fl;
        }
        "ids" => { // ids <slot>: json of ids per category (item box ids; equipment ids per category/sub), no personal values
            let idx: usize = a[6].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            let sc = |n: &Node, h: u32| -> i64 { match n.get(h) { Some(V::Scalar(_, x)) => *x as i64, _ => -1 } };
            fn sub<'a>(names: &Names, n: &'a Node, f: &str) -> Option<&'a Node> { match n.fields.iter().find(|(h, _)| names.field(*h) == f).map(|x| &x.1) { Some(V::Class(c)) => Some(&**c), _ => None } }
            fn arr<'a>(names: &Names, n: &'a Node, f: &str) -> Option<&'a Vec<Node>> { match n.fields.iter().find(|(h, _)| names.field(*h) == f).map(|x| &x.1) { Some(V::Classes(c)) => Some(c), _ => None } }
            let it = sub(&names, &slots[idx], "_Item").unwrap();
            let ids: Vec<String> = arr(&names, it, "_BoxItem").unwrap().iter().filter(|n| sc(n, 0xddf90b11) != 0 || sc(n, 0x620075e0) != 0).map(|n| format!("[{},{}]", sc(n, 0xddf90b11), sc(n, 0x620075e0))).collect();
            println!("{{\"box_items\":[{}],", ids.join(","));
            let eq = sub(&names, &slots[idx], "_Equip").unwrap();
            let mut m: BTreeMap<(i64, i64), Vec<i64>> = BTreeMap::new();
            for n in arr(&names, eq, "_EquipBox").unwrap() { let c = sc(n, 0xd595c544); if c == 141 { continue; } m.entry((c, sc(n, 0xc5a99e69))).or_default().push(sc(n, 0xefccb875)); }
            let parts: Vec<String> = m.iter().map(|((c, b), v)| format!("\"{c}_{b}\":{:?}", v)).collect();
            println!("\"equip\":{{{}}}}}", parts.join(","));
        }
        "bits" => { // bits <slot> <path>...: positions of set bits in a flag array (._Value) at each path like _Equip/_WeaponFlagParam/_LightBowgunCreateBit
            let idx: usize = a[6].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            for path in &a[7..] {
                let mut cur: &Node = &slots[idx];
                for p in path.split('/') {
                    match cur.fields.iter().find(|(h, _)| names.field(*h) == p).map(|x| &x.1) { Some(V::Class(c)) => cur = c, _ => panic!("no {p}") }
                }
                let Some((_, V::Arr(_, _, d))) = cur.fields.iter().find(|(h, _)| names.field(*h) == "_Value") else { panic!("no _Value") };
                let set: Vec<usize> = (0..d.len() * 8).filter(|b| d[b / 8] >> (b % 8) & 1 == 1).collect();
                println!("{path}: {} bits capacity, {} set: {:?}", d.len() * 8, set.len(), set);
            }
        }
        "rows" => { // rows <slot> <cat>: all small fields of the equipment box entries of one category (no 64-bit ids)
            let idx: usize = a[6].parse().unwrap();
            let cat: u64 = a[7].parse().unwrap();
            let V::Classes(slots) = top[0].get(top[0].fields[0].0).unwrap() else { panic!() };
            let Some(V::Class(e)) = slots[idx].fields.iter().find(|(h, _)| names.field(*h) == "_Equip").map(|x| &x.1) else { panic!() };
            let Some(V::Classes(bx)) = e.fields.iter().find(|(h, _)| names.field(*h) == "_EquipBox").map(|x| &x.1) else { panic!() };
            for (i, n) in bx.iter().enumerate() {
                if !matches!(n.get(0xd595c544), Some(V::Scalar(_, c)) if *c == cat) { continue; }
                let mut line = format!("#{i}");
                for (fh, v) in &n.fields { match v {
                    V::Scalar(t, x) if *t != 10 => line += &format!(" {:08x}={}", fh, *x as i64),
                    V::Arr(_, ms, d) => line += &format!(" {:08x}={:?}", fh, d.chunks(*ms as usize).map(|c| i16::from_le_bytes([c[0], c[1]])).collect::<Vec<_>>()),
                    _ => {} } }
                println!("{line}");
            }
        }
        _ => eprintln!("usage"),
    }
}
