//! GPU utilisation per process from the Windows "GPU Engine" performance counters (read-only).
use windows::core::PCWSTR;
use windows::Win32::System::Performance::*;

pub struct Gpu { q: PDH_HQUERY, c: PDH_HCOUNTER }

impl Gpu {
    pub fn new() -> Option<Self> {
        unsafe {
            let mut q = PDH_HQUERY::default();
            if PdhOpenQueryW(PCWSTR::null(), 0, &mut q) != 0 { return None; }
            let path: Vec<u16> = "\\GPU Engine(*)\\Utilization Percentage\0".encode_utf16().collect();
            let mut c = PDH_HCOUNTER::default();
            if PdhAddEnglishCounterW(q, PCWSTR(path.as_ptr()), 0, &mut c) != 0 { return None; }
            PdhCollectQueryData(q);
            Some(Gpu { q, c })
        }
    }

    /// Sum of utilisation over all engines of the given pids since the previous call.
    pub fn percent(&mut self, pids: &[u32]) -> f64 {
        unsafe {
            if PdhCollectQueryData(self.q) != 0 { return 0.0; }
            let (mut size, mut count) = (0u32, 0u32);
            // first call reports the buffer size (PDH_MORE_DATA)
            PdhGetFormattedCounterArrayW(self.c, PDH_FMT_DOUBLE, &mut size, &mut count, None);
            if size == 0 { return 0.0; }
            let mut buf = vec![0u8; size as usize];
            let items = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
            if PdhGetFormattedCounterArrayW(self.c, PDH_FMT_DOUBLE, &mut size, &mut count, Some(items)) != 0 { return 0.0; }
            let mut sum = 0.0;
            for i in 0..count as usize {
                let it = &*items.add(i);
                let name = it.szName.to_string().unwrap_or_default(); // "pid_1234_luid_..._engtype_3D"
                if let Some(rest) = name.strip_prefix("pid_") {
                    if let Some(pid) = rest.split('_').next().and_then(|p| p.parse::<u32>().ok()) {
                        if pids.contains(&pid) || std::env::var("SAMPLER_GPU_ALL").is_ok() { sum += it.FmtValue.Anonymous.doubleValue; }
                    }
                }
            }
            sum
        }
    }
}
