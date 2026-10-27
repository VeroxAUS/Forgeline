//! Throwaway spike (Phase 1 step 6): Tauri candidate. Same content as the egui app.
#[tauri::command]
fn ready() { println!("READY"); }

#[tauri::command]
async fn run_job() {
    let _ = std::thread::spawn(|| {
        println!("JOB start");
        low_priority();
        let mut h = 0u64;
        let t = std::time::Instant::now();
        while t.elapsed().as_secs() < 5 { for i in 0..1_000_000u64 { h = h.wrapping_mul(31).wrapping_add(i); } }
        std::hint::black_box(h);
        println!("JOB end");
    }).join();
}

#[cfg(windows)]
fn low_priority() {
    use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_LOWEST};
    unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_LOWEST); }
}
#[cfg(not(windows))]
fn low_priority() {}

fn main() {
    if let Some(t) = std::env::var("UIBENCH_JOB_AT").ok().and_then(|v| v.parse::<u64>().ok()) {
        std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_secs(t)); let _ = std::thread::spawn(|| { println!("JOB start"); low_priority(); let mut h = 0u64; let t = std::time::Instant::now(); while t.elapsed().as_secs() < 5 { for i in 0..1_000_000u64 { h = h.wrapping_mul(31).wrapping_add(i); } } std::hint::black_box(h); println!("JOB end"); }).join(); });
    }
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ready, run_job])
        .run(tauri::generate_context!())
        .expect("tauri");
}
