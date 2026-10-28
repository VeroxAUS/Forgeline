// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Lowers the priority of this process, for the update child process: heavy work must never
//! make the game stutter.

/// Sets the current process to below-normal priority. Returns false if the system refused (the
/// work then runs at normal priority) and does nothing off Windows.
#[cfg(windows)]
pub fn set_below_normal() -> bool {
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS,
    };
    // SAFETY: the pseudo handle from `GetCurrentProcess` needs no closing and the class is a
    // valid constant.
    unsafe { SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS) != 0 }
}

#[cfg(not(windows))]
pub fn set_below_normal() -> bool {
    false
}
