// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Shared code for Forgeline: where its data lives, how much memory the process uses, and the
//! machine-readable results files that every measurement writes.

pub mod datadir;
pub mod discovery;
pub mod memory;
pub mod priority;
pub mod results;
pub mod vdf;
