// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Walks the class stream without building a tree of it. The format is in `docs/research.md`
//! (Phase 1 step 2 section 4 with the two corrections from step 3):
//!
//! - everything is little endian and 4-byte aligned unless stated;
//! - the stream is a list of top-level entries: a 32-bit hash, then one class;
//! - a class is a 32-bit field count, a 32-bit class hash, then the fields;
//! - a field is a 32-bit name hash, a 32-bit type code, the value, then padding to 4 bytes;
//! - scalars and enums have a 32-bit size prefix (1, 2, 4 or 8) and are aligned to their size;
//! - a struct has a size prefix; a 16-byte struct's bytes start on a 16-byte boundary;
//! - a string is a character count and that many UTF-16 units;
//! - an array has a member type, member size, count and kind (0 values, 1 classes); values are
//!   packed without per-element prefixes (strings excepted); class arrays may have a `0xFFEEFFEE`
//!   marker, then one class hash per element, then the classes.
//!
//! Classes carry no byte length, so skipping one means walking its fields. Iterators therefore
//! skip lazily: the end of a nested class is only worked out when the caller moves on to the next
//! field, and not at all for a class the caller never asks about. Nothing is allocated.
//! Corrupt input gives [`SaveError::Stream`], never a panic.

use crate::SaveError;

const TYPE_ARRAY: i32 = -1;
const TYPE_CLASS: u32 = 17;
const CLASS_ARRAY_MARKER: u32 = 0xFFEE_FFEE;
/// Deepest class nesting accepted; real saves nest a handful of levels.
const MAX_DEPTH: u32 = 64;
/// Largest scalar or struct value accepted (the notes' reference walker uses the same limit).
const MAX_VALUE: usize = 0x10_0000;

type Res<T> = Result<T, SaveError>;

fn bad<T>(offset: usize, why: &'static str) -> Res<T> {
    Err(SaveError::Stream { offset, why })
}

fn align(pos: usize, n: usize) -> usize {
    pos.div_ceil(n) * n
}

fn u32_at(buf: &[u8], pos: usize) -> Res<u32> {
    match pos.checked_add(4).and_then(|end| buf.get(pos..end)) {
        Some(b) => Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        None => bad(pos, "stream ends early"),
    }
}

fn slice(buf: &[u8], pos: usize, len: usize) -> Res<&[u8]> {
    match pos.checked_add(len) {
        Some(end) if end <= buf.len() => Ok(&buf[pos..end]),
        _ => bad(pos, "stream ends early"),
    }
}

/// A scalar or enum value, zero-extended to 64 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scalar {
    /// The type code from the stream (2 bool, 4 u8, 7 s32, 11 f32 and so on; 1 is an enum).
    pub code: u32,
    /// Width in bytes: 1, 2, 4 or 8.
    pub size: u8,
    pub bits: u64,
}

impl Scalar {
    pub fn as_u64(&self) -> u64 {
        self.bits
    }

    /// Sign-extended for the signed type codes (3, 5, 7, 9), plain for the rest.
    pub fn as_i64(&self) -> i64 {
        let signed = matches!(self.code, 3 | 5 | 7 | 9);
        match (signed, self.size) {
            (true, 1) => self.bits as u8 as i8 as i64,
            (true, 2) => self.bits as u16 as i16 as i64,
            (true, 4) => self.bits as u32 as i32 as i64,
            _ => self.bits as i64,
        }
    }

    pub fn as_f32(&self) -> Option<f32> {
        (self.code == 11).then(|| f32::from_bits(self.bits as u32))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    Scalar(Scalar),
    /// UTF-16 little-endian bytes. Never printed by the reports.
    Str(&'a [u8]),
    Struct(&'a [u8]),
    Class(ClassView<'a>),
    Array(ArrayView<'a>),
}

impl Value<'_> {
    pub fn scalar(&self) -> Option<Scalar> {
        match self {
            Value::Scalar(s) => Some(*s),
            _ => None,
        }
    }
}

/// Where the next field starts, worked out only when the caller moves on.
#[derive(Clone, Copy)]
enum Resume<'a> {
    At(usize),
    SkipClass(usize),
    SkipArray(ArrayView<'a>),
}

impl Resume<'_> {
    fn resolve(self, buf: &[u8], depth: u32) -> Res<usize> {
        match self {
            Resume::At(p) => Ok(p),
            Resume::SkipClass(p) => skip_class(buf, p, depth + 1),
            Resume::SkipArray(a) => a.end(),
        }
    }
}

/// A class in the stream, at its field-count word.
#[derive(Debug, Clone, Copy)]
pub struct ClassView<'a> {
    buf: &'a [u8],
    at: usize,
    depth: u32,
}

impl<'a> ClassView<'a> {
    /// Hash of the class name.
    pub fn class_hash(&self) -> Res<u32> {
        u32_at(self.buf, self.at + 4)
    }

    pub fn field_count(&self) -> Res<u32> {
        u32_at(self.buf, self.at)
    }

    pub fn fields(&self) -> Fields<'a> {
        Fields {
            buf: self.buf,
            left: self.field_count().ok(),
            depth: self.depth,
            resume: Resume::At(self.at + 8),
            failed: self.field_count().is_err(),
        }
    }

    /// The first field with this name hash.
    pub fn field(&self, name_hash: u32) -> Res<Option<Value<'a>>> {
        for f in self.fields() {
            let f = f?;
            if f.name_hash == name_hash {
                return Ok(Some(f.value));
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    pub name_hash: u32,
    pub value: Value<'a>,
}

pub struct Fields<'a> {
    buf: &'a [u8],
    left: Option<u32>,
    depth: u32,
    resume: Resume<'a>,
    failed: bool,
}

impl<'a> Iterator for Fields<'a> {
    type Item = Res<Field<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let left = self.left?;
        if left == 0 {
            return None;
        }
        let step = (|| {
            let mut pos = self.resume.resolve(self.buf, self.depth)?;
            pos = align(pos, 4);
            let name_hash = u32_at(self.buf, pos)?;
            let (value, resume) = parse_value(self.buf, pos + 4, self.depth)?;
            self.resume = resume;
            Ok(Field { name_hash, value })
        })();
        match step {
            Ok(f) => {
                self.left = Some(left - 1);
                Some(Ok(f))
            }
            Err(e) => {
                self.failed = true;
                Some(Err(e))
            }
        }
    }
}

/// An array field. For classes (`kind == 1`) the elements are walked with [`ArrayView::classes`];
/// for plain values the packed bytes are available with [`ArrayView::raw`]. Arrays of strings,
/// of classes with `kind == 0` and of arrays are lists whose elements are walked one by one.
#[derive(Debug, Clone, Copy)]
pub struct ArrayView<'a> {
    buf: &'a [u8],
    pub member_type: u32,
    pub member_size: u32,
    pub count: u32,
    pub kind: u32,
    /// Where the per-element class hashes start, when the array has them (class arrays that carry
    /// the `0xFFEEFFEE` marker; without the marker there are no hashes).
    hashes_at: Option<usize>,
    /// Start of the first element or of the packed values.
    data_at: usize,
    depth: u32,
}

impl<'a> ArrayView<'a> {
    /// True when the elements are plain values stored back to back with no per-element prefix.
    fn packed(&self) -> bool {
        self.kind != 1 && !matches!(self.member_type as i32, 15 | 17 | TYPE_ARRAY)
    }

    /// Hashes of the element classes (empty when the array has none).
    pub fn class_hashes(&self) -> impl Iterator<Item = u32> + 'a {
        let (buf, at) = (self.buf, self.hashes_at);
        let n = if at.is_some() { self.count as usize } else { 0 };
        (0..n).filter_map(move |i| u32_at(buf, at? + 4 * i).ok())
    }

    /// The elements of a class array, one after the other.
    pub fn classes(&self) -> Classes<'a> {
        Classes {
            buf: self.buf,
            next_at: self.data_at,
            left: if self.kind == 1 || self.member_type == TYPE_CLASS {
                self.count
            } else {
                0
            },
            depth: self.depth,
            pending: None,
            failed: false,
        }
    }

    /// The packed bytes of a scalar or struct array.
    pub fn raw(&self) -> Res<&'a [u8]> {
        if !self.packed() {
            return bad(self.data_at, "not a packed array");
        }
        let len = (self.count as usize).saturating_mul(self.member_size as usize);
        slice(self.buf, self.data_at, len)
    }

    /// Scalar members, zero-extended (use [`Scalar`] rules for signs yourself if needed).
    pub fn scalars(&self) -> Res<impl Iterator<Item = u64> + 'a> {
        let size = self.member_size as usize;
        if !matches!(size, 1 | 2 | 4 | 8) {
            return bad(self.data_at, "member size is not a scalar size");
        }
        let raw = self.raw()?;
        Ok(raw.chunks_exact(size).map(move |c| {
            let mut a = [0u8; 8];
            a[..size].copy_from_slice(c);
            u64::from_le_bytes(a)
        }))
    }

    /// Where the array ends, which for lists and class arrays means walking every element.
    fn end(&self) -> Res<usize> {
        if self.packed() {
            return Ok(self.data_at + self.raw()?.len());
        }
        let mut pos = self.data_at;
        for _ in 0..self.count {
            pos = match (self.kind, self.member_type as i32) {
                (1, _) | (_, 17) => skip_class(self.buf, align(pos, 4), self.depth + 1)?,
                (_, 15) => skip_string(self.buf, pos)?,
                _ => {
                    // An array inside an array: its header follows, aligned.
                    let (inner, _) = parse_array(self.buf, align(pos, 4), self.depth + 1)?;
                    inner.end()?
                }
            };
        }
        Ok(pos)
    }
}

pub struct Classes<'a> {
    buf: &'a [u8],
    next_at: usize,
    left: u32,
    depth: u32,
    pending: Option<usize>,
    failed: bool,
}

impl<'a> Iterator for Classes<'a> {
    type Item = Res<ClassView<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed || self.left == 0 {
            return None;
        }
        if let Some(p) = self.pending.take() {
            match skip_class(self.buf, p, self.depth + 1) {
                Ok(end) => self.next_at = end,
                Err(e) => {
                    self.failed = true;
                    return Some(Err(e));
                }
            }
        }
        self.left -= 1;
        let at = align(self.next_at, 4);
        if u32_at(self.buf, at).is_err() {
            self.failed = true;
            return Some(bad(at, "stream ends early"));
        }
        self.pending = Some(at);
        Some(Ok(ClassView {
            buf: self.buf,
            at,
            depth: self.depth + 1,
        }))
    }
}

/// Parses the value after a field-name hash. Returns it and where to resume afterwards.
fn parse_value(buf: &[u8], pos: usize, depth: u32) -> Res<(Value<'_>, Resume<'_>)> {
    let code = u32_at(buf, pos)?;
    let pos = pos + 4;
    match code as i32 {
        1..=14 => {
            let pos = align(pos, 4);
            let size = u32_at(buf, pos)? as usize;
            if size > MAX_VALUE {
                return bad(pos, "value size is out of range");
            }
            let at = if matches!(size, 1 | 2 | 4 | 8) {
                align(pos + 4, size)
            } else {
                pos + 4
            };
            let b = slice(buf, at, size)?;
            let v = if size <= 8 {
                let mut a = [0u8; 8];
                a[..size].copy_from_slice(b);
                Value::Scalar(Scalar {
                    code,
                    size: size as u8,
                    bits: u64::from_le_bytes(a),
                })
            } else {
                Value::Struct(b)
            };
            Ok((v, Resume::At(at + size)))
        }
        15 => {
            let pos = align(pos, 4);
            let units = u32_at(buf, pos)? as usize;
            let bytes = slice(buf, pos + 4, units.saturating_mul(2))?;
            Ok((Value::Str(bytes), Resume::At(pos + 4 + bytes.len())))
        }
        16 => {
            let pos = align(pos, 4);
            let len = u32_at(buf, pos)? as usize;
            if len > MAX_VALUE {
                return bad(pos, "value size is out of range");
            }
            let at = if len == 16 {
                align(pos + 4, 16)
            } else {
                pos + 4
            };
            let b = slice(buf, at, len)?;
            Ok((Value::Struct(b), Resume::At(at + len)))
        }
        17 => {
            if depth >= MAX_DEPTH {
                return bad(pos, "classes nest too deeply");
            }
            u32_at(buf, pos)?;
            let c = ClassView {
                buf,
                at: pos,
                depth: depth + 1,
            };
            Ok((Value::Class(c), Resume::SkipClass(pos)))
        }
        TYPE_ARRAY => {
            let (a, resume) = parse_array(buf, align(pos, 4), depth)?;
            Ok((Value::Array(a), resume))
        }
        _ => bad(pos - 4, "unknown type code"),
    }
}

/// Parses an array from its member-type word (already 4-aligned): member type, member size,
/// count, kind, then the elements. Kind 1 is an array of classes, with an optional marker and
/// per-element hashes; every other kind is values, packed unless the members are strings, classes
/// or arrays, which are walked one by one.
fn parse_array(buf: &[u8], pos: usize, depth: u32) -> Res<(ArrayView<'_>, Resume<'_>)> {
    if depth >= MAX_DEPTH {
        return bad(pos, "arrays nest too deeply");
    }
    let member_type = u32_at(buf, pos)?;
    let member_size = u32_at(buf, pos + 4)?;
    let count = u32_at(buf, pos + 8)?;
    let kind = u32_at(buf, pos + 12)?;
    let mut data_at = pos + 16;
    let mut hashes_at = None;
    if kind == 1 {
        if u32_at(buf, data_at).ok() == Some(CLASS_ARRAY_MARKER) {
            hashes_at = Some(data_at + 4);
            data_at += 4 + (count as usize).saturating_mul(4);
            if data_at > buf.len() {
                return bad(pos, "stream ends early");
            }
        }
    } else if member_size == 16 && !matches!(member_type as i32, 15 | 17 | TYPE_ARRAY) {
        data_at = align(data_at, 16);
    }
    let a = ArrayView {
        buf,
        member_type,
        member_size,
        count,
        kind,
        hashes_at,
        data_at,
        depth,
    };
    if a.packed() {
        let end = data_at + a.raw()?.len();
        return Ok((a, Resume::At(end)));
    }
    Ok((a, Resume::SkipArray(a)))
}

fn skip_string(buf: &[u8], pos: usize) -> Res<usize> {
    let pos = align(pos, 4);
    let units = u32_at(buf, pos)? as usize;
    Ok(pos + 4 + slice(buf, pos + 4, units.saturating_mul(2))?.len())
}

/// Walks one class from its field-count word and returns where it ends.
fn skip_class(buf: &[u8], pos: usize, depth: u32) -> Res<usize> {
    if depth >= MAX_DEPTH {
        return bad(pos, "classes nest too deeply");
    }
    let count = u32_at(buf, pos)?;
    u32_at(buf, pos + 4)?;
    let mut pos = pos + 8;
    for _ in 0..count {
        pos = align(pos, 4);
        u32_at(buf, pos)?;
        let (_, resume) = parse_value(buf, pos + 4, depth)?;
        pos = align(resume.resolve(buf, depth)?, 4);
    }
    Ok(pos)
}

/// A whole class stream.
#[derive(Debug, Clone, Copy)]
pub struct Stream<'a> {
    buf: &'a [u8],
}

impl<'a> Stream<'a> {
    pub fn new(buf: &'a [u8]) -> Stream<'a> {
        Stream { buf }
    }

    /// The top-level entries. Each item is the class and the entry hash before it.
    pub fn top_level(&self) -> TopLevel<'a> {
        TopLevel {
            buf: self.buf,
            pos: 0,
            pending: None,
            failed: false,
        }
    }

    /// Walks every byte of the stream and checks it ends exactly where the rules say. Returns the
    /// number of top-level entries. This is the integrity check of the whole read.
    pub fn check(&self) -> Res<usize> {
        let mut pos = 0;
        let mut entries = 0;
        loop {
            pos = align(pos, 4);
            if pos + 7 >= self.buf.len() {
                // Fewer than 8 bytes left is padding at the end, as in the reference notes.
                return Ok(entries);
            }
            u32_at(self.buf, pos)?;
            pos = skip_class(self.buf, pos + 4, 0)?;
            entries += 1;
        }
    }
}

pub struct TopLevel<'a> {
    buf: &'a [u8],
    pos: usize,
    pending: Option<usize>,
    failed: bool,
}

impl<'a> Iterator for TopLevel<'a> {
    type Item = Res<(u32, ClassView<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        if let Some(p) = self.pending.take() {
            match skip_class(self.buf, p, 0) {
                Ok(end) => self.pos = end,
                Err(e) => {
                    self.failed = true;
                    return Some(Err(e));
                }
            }
        }
        let pos = align(self.pos, 4);
        if pos + 7 >= self.buf.len() {
            return None;
        }
        let hash = match u32_at(self.buf, pos) {
            Ok(h) => h,
            Err(e) => {
                self.failed = true;
                return Some(Err(e));
            }
        };
        self.pending = Some(pos + 4);
        Some(Ok((
            hash,
            ClassView {
                buf: self.buf,
                at: pos + 4,
                depth: 0,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct B(Vec<u8>);
    impl B {
        fn u32(&mut self, v: u32) -> &mut Self {
            self.0.extend_from_slice(&v.to_le_bytes());
            self
        }
        fn align(&mut self, n: usize) -> &mut Self {
            while !self.0.len().is_multiple_of(n) {
                self.0.push(0);
            }
            self
        }
    }

    /// A hand-built class that uses what the fixture builder does not write: an array of strings
    /// and an array of 16-byte structs, followed by a plain field.
    fn odd_stream() -> Vec<u8> {
        let mut b = B(Vec::new());
        b.u32(0xAAAA_0001).u32(3).u32(0xBBBB_0001);
        // Field A: array of two strings, "ab" and "c".
        b.u32(0xA).u32(-1i32 as u32).u32(15).u32(0).u32(2).u32(0);
        b.u32(2).0.extend_from_slice(&[b'a', 0, b'b', 0]);
        b.align(4).u32(1).0.extend_from_slice(&[b'c', 0]);
        b.align(4);
        // Field B: array of one 16-byte struct, aligned to 16.
        b.u32(0xB).u32(-1i32 as u32).u32(16).u32(16).u32(1).u32(0);
        b.align(16).0.extend_from_slice(&[7; 16]);
        // Field C: a u8.
        b.align(4).u32(0xC).u32(4).u32(1);
        b.0.push(9);
        b.align(4);
        b.0
    }

    #[test]
    fn string_and_struct_arrays_are_walked_to_the_exact_end() {
        let s = odd_stream();
        assert_eq!(Stream::new(&s).check(), Ok(1));
        let (_, class) = Stream::new(&s).top_level().next().unwrap().unwrap();
        let c = class.field(0xC).unwrap().unwrap().scalar().unwrap();
        assert_eq!(c.as_u64(), 9);
        let Some(Value::Array(a)) = class.field(0xB).unwrap() else {
            panic!("no array")
        };
        assert_eq!(a.raw().unwrap(), &[7; 16]);
    }

    #[test]
    fn a_stream_cut_anywhere_is_an_error_not_a_panic() {
        let s = odd_stream();
        for cut in 8..s.len() - 3 {
            assert!(Stream::new(&s[..cut]).check().is_err(), "cut at {cut}");
        }
    }

    #[test]
    fn signed_scalars_are_sign_extended() {
        let s = Scalar {
            code: 7,
            size: 4,
            bits: 0xFFFF_FFFF,
        };
        assert_eq!(s.as_i64(), -1);
        let u = Scalar {
            code: 8,
            size: 4,
            bits: 0xFFFF_FFFF,
        };
        assert_eq!(u.as_i64(), 0xFFFF_FFFF);
    }

    #[test]
    fn deep_nesting_is_refused() {
        // A class whose only field is another class, repeated far past the limit.
        let mut b = B(Vec::new());
        b.u32(1);
        for _ in 0..200 {
            b.u32(1).u32(2).u32(3).u32(17);
        }
        assert!(Stream::new(&b.0).check().is_err());
    }

    /// The first words of a real save, as the maintainer's PC printed them (structure only):
    /// root entry, a three-element class array with no marker and no hashes, and the first
    /// element's first fields. Rebuilt here with invented values.
    #[test]
    fn a_class_array_with_no_marker_and_no_hashes_walks_like_the_real_save() {
        let mut b = B(Vec::new());
        // entry hash, 1 field, root class hash; field _Data: array of 3 classes (member size 8).
        b.u32(0xDBE3_F199).u32(1).u32(0x827F_CCDC);
        b.u32(0x695A_3627)
            .u32(-1i32 as u32)
            .u32(17)
            .u32(8)
            .u32(3)
            .u32(1);
        for i in 0..3u32 {
            // class: 2 fields: a u8 (Active) and a string with two characters.
            b.u32(2).u32(0xD32F_821C);
            b.u32(0x69DB_AE94).u32(4).u32(1);
            b.0.push(u8::from(i == 1));
            b.align(4).u32(0x043F_3BB4).u32(15).u32(2);
            b.0.extend_from_slice(&[b'o', 0, b'k', 0]);
            b.align(4);
        }
        let s = b.0;
        assert_eq!(Stream::new(&s).check(), Ok(1));
        let (_, root) = Stream::new(&s).top_level().next().unwrap().unwrap();
        let Some(Value::Array(a)) = root.field(0x695A_3627).unwrap() else {
            panic!("no array")
        };
        let active: Vec<u64> = a
            .classes()
            .map(|c| {
                c.unwrap()
                    .field(0x69DB_AE94)
                    .unwrap()
                    .unwrap()
                    .scalar()
                    .unwrap()
                    .as_u64()
            })
            .collect();
        assert_eq!(active, [0, 1, 0]);
        assert_eq!(a.class_hashes().count(), 0);
    }

    #[test]
    fn a_class_array_with_the_marker_has_hashes_that_are_skipped() {
        let mut b = B(Vec::new());
        b.u32(0xAAAA_0001).u32(1).u32(0xBBBB_0001);
        b.u32(0xA).u32(-1i32 as u32).u32(17).u32(8).u32(2).u32(1);
        b.u32(CLASS_ARRAY_MARKER).u32(0x1111_1111).u32(0x2222_2222);
        for _ in 0..2 {
            b.u32(1).u32(0x1111_1111);
            b.u32(0xB).u32(7).u32(4).u32(5);
        }
        let s = b.0;
        assert_eq!(Stream::new(&s).check(), Ok(1));
        let (_, root) = Stream::new(&s).top_level().next().unwrap().unwrap();
        let Some(Value::Array(a)) = root.field(0xA).unwrap() else {
            panic!("no array")
        };
        assert_eq!(
            a.class_hashes().collect::<Vec<_>>(),
            [0x1111_1111, 0x2222_2222]
        );
        assert_eq!(a.classes().count(), 2);
    }

    #[test]
    fn scalars_of_an_odd_size_are_read_as_they_are_stored() {
        // The reference walker accepts any size for scalar types; only 1, 2, 4 and 8 are aligned.
        let mut b = B(Vec::new());
        b.u32(0xAAAA_0002).u32(2).u32(0xBBBB_0002);
        b.u32(0xA).u32(8).u32(3);
        b.0.extend_from_slice(&[1, 2, 3]);
        b.align(4).u32(0xB).u32(8).u32(12);
        b.0.extend_from_slice(&[9; 12]);
        b.align(4);
        let s = b.0;
        assert_eq!(Stream::new(&s).check(), Ok(1));
        let (_, c) = Stream::new(&s).top_level().next().unwrap().unwrap();
        let v = c.field(0xA).unwrap().unwrap().scalar().unwrap();
        assert_eq!((v.size, v.as_u64()), (3, 0x03_0201));
    }

    #[test]
    fn a_list_of_classes_stored_as_values_walks_to_the_end() {
        // Array kind 0 with member type 17: each element is a class, back to back.
        let mut b = B(Vec::new());
        b.u32(0xAAAA_0003).u32(1).u32(0xBBBB_0003);
        b.u32(0xA).u32(-1i32 as u32).u32(17).u32(0).u32(2).u32(0);
        for _ in 0..2 {
            b.u32(1).u32(0xCCCC_0003);
            b.u32(0xB).u32(7).u32(4).u32(5);
        }
        assert_eq!(Stream::new(&b.0).check(), Ok(1));
    }
}
