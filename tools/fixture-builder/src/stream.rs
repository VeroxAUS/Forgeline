// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Writes the self-describing class stream (`docs/research.md`, Phase 1 step 2 section 4 with the
//! two corrections from step 3): classes of hashed fields, 4-byte aligned, scalars and enums with a
//! size prefix, 16-byte structs aligned to 16, scalar and struct arrays packed, class arrays with
//! the optional `0xFFEEFFEE` marker and one class hash per element.

use crate::crypto::name_hash;

const TYPE_ARRAY: i32 = -1;
const TYPE_BOOL: u32 = 2;
const TYPE_U8: u32 = 4;
const TYPE_S16: u32 = 5;
const TYPE_S32: u32 = 7;
const TYPE_U32: u32 = 8;
const TYPE_F32: u32 = 11;
const TYPE_STRING: u32 = 15;
const TYPE_STRUCT: u32 = 16;
const TYPE_CLASS: u32 = 17;

const CLASS_ARRAY_MARKER: u32 = 0xFFEE_FFEE;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub fields: Vec<(String, Value)>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    U8(u8),
    S32(i32),
    U32(u32),
    F32(f32),
    Str(String),
    /// Sixteen raw bytes, aligned to 16 (invented content only).
    Struct16([u8; 16]),
    Class(Class),
    ArrayS16(Vec<i16>),
    ArrayU32(Vec<u32>),
    ArrayClass(Vec<Class>),
}

impl Class {
    pub fn new(name: &str) -> Class {
        Class {
            name: name.to_string(),
            fields: Vec::new(),
        }
    }
    pub fn with(mut self, field: &str, value: Value) -> Class {
        self.fields.push((field.to_string(), value));
        self
    }
}

struct Out(Vec<u8>);

impl Out {
    fn align(&mut self, n: usize) {
        while !self.0.len().is_multiple_of(n) {
            self.0.push(0);
        }
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    /// Size prefix, then the value aligned to its own size.
    fn sized(&mut self, bytes: &[u8]) {
        self.align(4);
        self.u32(bytes.len() as u32);
        self.align(bytes.len());
        self.0.extend_from_slice(bytes);
    }

    fn class(&mut self, c: &Class) {
        self.u32(c.fields.len() as u32);
        self.u32(name_hash(&c.name));
        for (name, value) in &c.fields {
            self.align(4);
            self.u32(name_hash(name));
            self.field_value(value);
            self.align(4);
        }
    }

    fn string(&mut self, s: &str) {
        self.align(4);
        let units: Vec<u16> = s.encode_utf16().collect();
        self.u32(units.len() as u32);
        for u in units {
            self.0.extend_from_slice(&u.to_le_bytes());
        }
    }

    fn field_value(&mut self, v: &Value) {
        match v {
            Value::Bool(b) => {
                self.u32(TYPE_BOOL);
                self.sized(&[u8::from(*b)]);
            }
            Value::U8(x) => {
                self.u32(TYPE_U8);
                self.sized(&[*x]);
            }
            Value::S32(x) => {
                self.u32(TYPE_S32);
                self.sized(&x.to_le_bytes());
            }
            Value::U32(x) => {
                self.u32(TYPE_U32);
                self.sized(&x.to_le_bytes());
            }
            Value::F32(x) => {
                self.u32(TYPE_F32);
                self.sized(&x.to_le_bytes());
            }
            Value::Str(s) => {
                self.u32(TYPE_STRING);
                self.string(s);
            }
            Value::Struct16(bytes) => {
                self.u32(TYPE_STRUCT);
                self.align(4);
                self.u32(16);
                self.align(16);
                self.0.extend_from_slice(bytes);
            }
            Value::Class(c) => {
                self.u32(TYPE_CLASS);
                self.class(c);
            }
            Value::ArrayS16(v) => self.packed_array(
                TYPE_S16,
                2,
                v.len(),
                v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            ),
            Value::ArrayU32(v) => self.packed_array(
                TYPE_U32,
                4,
                v.len(),
                v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            ),
            Value::ArrayClass(items) => {
                self.i32(TYPE_ARRAY);
                self.align(4);
                self.u32(TYPE_CLASS);
                self.u32(0); // member size: not known for classes
                self.u32(items.len() as u32);
                self.u32(1); // kind 1 = classes
                self.u32(CLASS_ARRAY_MARKER);
                for c in items {
                    self.u32(name_hash(&c.name));
                }
                for c in items {
                    self.class(c);
                }
            }
        }
    }

    fn packed_array(&mut self, member_type: u32, member_size: u32, count: usize, bytes: Vec<u8>) {
        self.i32(TYPE_ARRAY);
        self.align(4);
        self.u32(member_type);
        self.u32(member_size);
        self.u32(count as u32);
        self.u32(0); // kind 0 = values
        self.0.extend_from_slice(&bytes);
    }
}

/// A whole stream: top-level entries, each a hash of the class name followed by the class.
pub fn write_stream(top: &[Class]) -> Vec<u8> {
    let mut out = Out(Vec::new());
    for c in top {
        out.align(4);
        out.u32(name_hash(&c.name));
        out.class(c);
    }
    out.align(4);
    out.0
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A walker that follows the same rules, to prove the writer's output can be walked to the
    /// exact end. Returns the number of classes seen.
    pub fn walk(stream: &[u8]) -> usize {
        struct W<'a> {
            s: &'a [u8],
            p: usize,
            classes: usize,
        }
        impl W<'_> {
            fn u32(&mut self) -> u32 {
                let v = u32::from_le_bytes(self.s[self.p..self.p + 4].try_into().unwrap());
                self.p += 4;
                v
            }
            fn align(&mut self, n: usize) {
                self.p = self.p.div_ceil(n) * n;
            }
            fn class(&mut self) {
                self.classes += 1;
                let count = self.u32();
                let _hash = self.u32();
                for _ in 0..count {
                    self.align(4);
                    let _name = self.u32();
                    self.field();
                    self.align(4);
                }
            }
            fn sized(&mut self) {
                self.align(4);
                let size = self.u32() as usize;
                assert!(matches!(size, 1 | 2 | 4 | 8), "scalar size {size}");
                self.align(size);
                self.p += size;
            }
            fn field(&mut self) {
                let t = self.u32() as i32;
                match t {
                    1..=14 => self.sized(),
                    15 => {
                        self.align(4);
                        let n = self.u32() as usize;
                        self.p += n * 2;
                    }
                    16 => {
                        self.align(4);
                        let n = self.u32() as usize;
                        assert_eq!(n, 16);
                        self.align(16);
                        self.p += n;
                    }
                    17 => self.class(),
                    -1 => {
                        self.align(4);
                        let member = self.u32();
                        let size = self.u32() as usize;
                        let count = self.u32() as usize;
                        let kind = self.u32();
                        if kind == 1 {
                            assert_eq!(member, 17);
                            let marker =
                                u32::from_le_bytes(self.s[self.p..self.p + 4].try_into().unwrap());
                            if marker == CLASS_ARRAY_MARKER {
                                self.p += 4;
                            }
                            self.p += 4 * count;
                            for _ in 0..count {
                                self.class();
                            }
                        } else {
                            self.p += count * size;
                        }
                    }
                    other => panic!("unknown type code {other}"),
                }
            }
        }
        let mut w = W {
            s: stream,
            p: 0,
            classes: 0,
        };
        while w.p + 7 < stream.len() {
            w.align(4);
            let _hash = w.u32();
            w.class();
        }
        assert!(
            w.p <= stream.len() && stream.len() - w.p < 8,
            "walk ended at {} of {}",
            w.p,
            stream.len()
        );
        w.classes
    }

    #[test]
    fn scalars_are_sized_and_aligned() {
        let c = Class::new("app.test.cOne")
            .with("_A", Value::U8(7))
            .with("_B", Value::S32(-2));
        let s = write_stream(&[c]);
        // entry hash, count 2, class hash, then field _A: name hash, type 4, size 1, value 7 padded.
        assert_eq!(u32::from_le_bytes(s[4..8].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(s[16..20].try_into().unwrap()), 4);
        assert_eq!(u32::from_le_bytes(s[20..24].try_into().unwrap()), 1);
        assert_eq!(s[24], 7);
        assert_eq!(s.len() % 4, 0);
        assert_eq!(walk(&s), 1);
    }

    #[test]
    fn structs_are_16_aligned_and_arrays_are_packed() {
        let c = Class::new("app.test.cTwo")
            .with("_Flag", Value::Bool(true))
            .with("_Vec", Value::Struct16([9; 16]))
            .with("_Ids", Value::ArrayS16(vec![1, -1, 3]))
            .with("_Name", Value::Str("abc".into()));
        let s = write_stream(&[c]);
        let at = s.windows(16).position(|w| w == [9; 16]).unwrap();
        assert_eq!(at % 16, 0, "struct bytes sit on a 16-byte boundary");
        assert_eq!(walk(&s), 1);
    }

    #[test]
    fn class_arrays_walk_and_count_every_class() {
        let item = |n: i32| Class::new("app.test.cItem").with("_N", Value::S32(n));
        let root = Class::new("app.test.cRoot")
            .with("_Items", Value::ArrayClass(vec![item(1), item(2), item(3)]));
        assert_eq!(walk(&write_stream(&[root])), 4);
    }
}
