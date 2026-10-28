// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Lists every value in a class stream with a readable path, for comparing two saves. Used by
//! `save-diff`. Text and raw bytes are never reported as content, only as a length and a
//! fingerprint, so a report cannot show a hunter name or ID. Fields can be skipped by name hash
//! (the report skips `_BasicData`, where the hunter's identity lives).

use crate::hash::murmur3_32;
use crate::names::TypeNames;
use crate::stream::{ClassView, Stream, Value};
use crate::SaveError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaf {
    Int(i64),
    /// A 32-bit float, kept as its bits so equality is exact.
    F32(u32),
    F64(u64),
    /// A string: its length in characters and a fingerprint, never the text.
    Text {
        chars: usize,
        fingerprint: u32,
    },
    Bytes {
        len: usize,
        fingerprint: u32,
    },
}

impl fmt::Display for Leaf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Leaf::Int(v) => write!(f, "{v}"),
            Leaf::F32(b) => write!(f, "{}", f32::from_bits(*b)),
            Leaf::F64(b) => write!(f, "{}", f64::from_bits(*b)),
            Leaf::Text { chars, .. } => write!(f, "text of {chars} characters"),
            Leaf::Bytes { len, .. } => write!(f, "{len} raw bytes"),
        }
    }
}

struct Walk<'a> {
    names: &'a TypeNames,
    skip: &'a [u32],
    path: String,
    skipped: usize,
    visit: &'a mut dyn FnMut(&str, Leaf),
}

/// Visits every leaf value. Returns how many skipped fields were left out.
pub fn flatten(
    stream: &[u8],
    names: &TypeNames,
    skip: &[u32],
    visit: &mut dyn FnMut(&str, Leaf),
) -> Result<usize, SaveError> {
    let mut w = Walk {
        names,
        skip,
        path: String::new(),
        skipped: 0,
        visit,
    };
    for entry in Stream::new(stream).top_level() {
        let (_, class) = entry?;
        let keep = w.path.len();
        w.path.push_str(&names.show(class.class_hash()?));
        w.class(&class)?;
        w.path.truncate(keep);
    }
    Ok(w.skipped)
}

impl Walk<'_> {
    fn class(&mut self, c: &ClassView) -> Result<(), SaveError> {
        for f in c.fields() {
            let f = f?;
            if self.skip.contains(&f.name_hash) {
                self.skipped += 1;
                continue;
            }
            let keep = self.path.len();
            self.path.push('.');
            self.path.push_str(&self.names.show(f.name_hash));
            self.value(&f.value)?;
            self.path.truncate(keep);
        }
        Ok(())
    }

    fn value(&mut self, v: &Value) -> Result<(), SaveError> {
        match v {
            Value::Scalar(s) => {
                let leaf = match s.code {
                    11 => Leaf::F32(s.bits as u32),
                    12 => Leaf::F64(s.bits),
                    _ => Leaf::Int(s.as_i64()),
                };
                (self.visit)(&self.path, leaf);
            }
            Value::Str(b) => {
                let leaf = Leaf::Text {
                    chars: b.len() / 2,
                    fingerprint: murmur3_32(b, 0),
                };
                (self.visit)(&self.path, leaf);
            }
            Value::Struct(b) => {
                let leaf = Leaf::Bytes {
                    len: b.len(),
                    fingerprint: murmur3_32(b, 0),
                };
                (self.visit)(&self.path, leaf);
            }
            Value::Class(c) => self.class(c)?,
            Value::Array(a) => {
                if a.kind == 1 {
                    for (i, c) in a.classes().enumerate() {
                        let keep = self.path.len();
                        self.path.push_str(&format!("[{i}]"));
                        self.class(&c?)?;
                        self.path.truncate(keep);
                    }
                } else if a.member_type == 15 {
                    // Arrays of strings carry names and the like; report only that they exist.
                    let leaf = Leaf::Int(i64::from(a.count));
                    let keep = self.path.len();
                    self.path.push_str(".count");
                    (self.visit)(&self.path, leaf);
                    self.path.truncate(keep);
                } else if let Ok(values) = a.scalars() {
                    let signed = matches!(a.member_type, 3 | 5 | 7 | 9);
                    for (i, bits) in values.enumerate() {
                        let keep = self.path.len();
                        self.path.push_str(&format!("[{i}]"));
                        let leaf = match (a.member_type, a.member_size) {
                            (11, 4) => Leaf::F32(bits as u32),
                            (12, 8) => Leaf::F64(bits),
                            _ if signed => Leaf::Int(match a.member_size {
                                1 => bits as u8 as i8 as i64,
                                2 => bits as u16 as i16 as i64,
                                4 => bits as u32 as i32 as i64,
                                _ => bits as i64,
                            }),
                            _ => Leaf::Int(bits as i64),
                        };
                        (self.visit)(&self.path, leaf);
                        self.path.truncate(keep);
                    }
                } else if let Ok(raw) = a.raw() {
                    // Arrays of structs: bytes only.
                    let leaf = Leaf::Bytes {
                        len: raw.len(),
                        fingerprint: murmur3_32(raw, 0),
                    };
                    (self.visit)(&self.path, leaf);
                }
            }
        }
        Ok(())
    }
}
