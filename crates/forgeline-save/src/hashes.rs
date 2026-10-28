// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Collects every class hash and field-name hash that occurs in a class stream. The exe scan
//! (`forgeline-pak`) takes this set so it only rebuilds the names that matter for this save.

use crate::stream::{ClassView, Stream, Value};
use crate::SaveError;
use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SaveHashes {
    pub classes: BTreeSet<u32>,
    pub fields: BTreeSet<u32>,
}

impl SaveHashes {
    /// Every hash, classes and fields together.
    pub fn all(&self) -> BTreeSet<u32> {
        self.classes.union(&self.fields).copied().collect()
    }
}

/// Walks the whole stream once. Elements of arrays inside arrays are not entered (the walker has
/// no view of them), so a class that only occurs there is missed and stays unnamed.
pub fn collect(stream: &[u8]) -> Result<SaveHashes, SaveError> {
    let mut out = SaveHashes::default();
    for entry in Stream::new(stream).top_level() {
        let (_, class) = entry?;
        class_into(&class, &mut out)?;
    }
    Ok(out)
}

fn class_into(c: &ClassView, out: &mut SaveHashes) -> Result<(), SaveError> {
    out.classes.insert(c.class_hash()?);
    for f in c.fields() {
        let f = f?;
        out.fields.insert(f.name_hash);
        match f.value {
            Value::Class(inner) => class_into(&inner, out)?,
            Value::Array(a) => {
                out.classes.extend(a.class_hashes());
                for inner in a.classes() {
                    class_into(&inner?, out)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}
