// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! How much memory this process uses: private bytes (what the budget limits), the peak of that,
//! and the working set. Read from the operating system; nothing is sent anywhere.

/// All three values are in bytes. A value is `None` when the system does not report it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryUse {
    pub private_bytes: Option<u64>,
    pub peak_private_bytes: Option<u64>,
    pub working_set_bytes: Option<u64>,
}

#[cfg(windows)]
pub fn read() -> MemoryUse {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    // SAFETY: the struct is plain data, `cb` is its true size, and the pseudo handle from
    // `GetCurrentProcess` needs no closing.
    unsafe {
        let mut c: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let ok = GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut c as *mut PROCESS_MEMORY_COUNTERS_EX as *mut PROCESS_MEMORY_COUNTERS,
            c.cb,
        );
        if ok == 0 {
            return MemoryUse::default();
        }
        MemoryUse {
            private_bytes: Some(c.PrivateUsage as u64),
            peak_private_bytes: Some(c.PeakPagefileUsage as u64),
            working_set_bytes: Some(c.WorkingSetSize as u64),
        }
    }
}

/// On Linux "private bytes" is the resident set, which is the closest match (used only for
/// development and the Linux CI job; the product is Windows only).
#[cfg(not(windows))]
pub fn read() -> MemoryUse {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    let kb = |key: &str| -> Option<u64> {
        status
            .lines()
            .find_map(|l| l.strip_prefix(key))
            .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
            .map(|k| k * 1024)
    };
    MemoryUse {
        private_bytes: kb("VmRSS:"),
        peak_private_bytes: kb("VmHWM:"),
        working_set_bytes: kb("VmRSS:"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn this_process_reports_some_memory() {
        let m = super::read();
        assert!(m.private_bytes.unwrap_or(0) > 0);
    }
}
