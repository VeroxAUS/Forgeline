//! Throwaway spike (Phase 1 step 6): our own process sampler.
//!   sampler run <label> <idle_secs> <out_dir> <exe> [args...]
//! Launches the exe (its stdout must print READY once usable, and JOB start / JOB end around its background job),
//! measures cold start, then samples the whole process tree (RAM, CPU, GPU) once a second: idle window, then job window.
//! The cross-platform core uses sysinfo; GPU (Windows only) comes from performance counters. It reads figures only and
//! never touches another process's memory.
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[cfg(windows)]
mod gpu;
#[cfg(not(windows))]
mod gpu {
    pub struct Gpu;
    impl Gpu { pub fn new() -> Option<Self> { None } pub fn percent(&mut self, _p: &[u32]) -> f64 { 0.0 } }
}

fn tree(sys: &System, root: u32) -> Vec<u32> {
    let mut v = vec![root];
    let mut i = 0;
    while i < v.len() {
        for (pid, p) in sys.processes() {
            if p.parent().map(|x| x.as_u32()) == Some(v[i]) && !v.contains(&pid.as_u32()) { v.push(pid.as_u32()); }
        }
        i += 1;
    }
    v
}

fn median(v: &mut Vec<f64>) -> f64 {
    if v.is_empty() { return 0.0; }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.get(1).map(|s| s.as_str()) != Some("run") || a.len() < 6 { eprintln!("usage: sampler run <label> <idle_secs> <out_dir> <exe> [args]"); return; }
    let (label, idle, out) = (&a[2], a[3].parse::<u64>().unwrap(), &a[4]);
    let mut cmd = Command::new(&a[5]);
    cmd.args(&a[6..]).stdout(Stdio::piped()).env("UIBENCH_JOB_AT", (idle + 12).to_string());
    let t0 = Instant::now();
    let mut child = cmd.spawn().expect("spawn");
    let root = child.id();
    let (tx, rx) = mpsc::channel::<(String, Instant, std::time::SystemTime)>();
    let so = child.stdout.take().unwrap();
    std::thread::spawn(move || { for l in BufReader::new(so).lines().flatten() { let _ = tx.send((l, Instant::now(), std::time::SystemTime::now())); } });

    let mut cold_ms = f64::NAN;
    let mut markers: Vec<(String, std::time::SystemTime)> = vec![];
    while t0.elapsed() < Duration::from_secs(30) {
        if let Ok((l, at, wall)) = rx.recv_timeout(Duration::from_millis(50)) { markers.push((l.clone(), wall)); if l.trim() == "READY" { cold_ms = at.duration_since(t0).as_secs_f64() * 1000.0; break; } }
    }
    std::thread::sleep(Duration::from_secs(3)); // settle

    let mut sys = System::new();
    let mut gpu = gpu::Gpu::new();
    let ncpu = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as f64;
    let kind = ProcessRefreshKind::nothing().with_memory().with_cpu();
    let mut rows: Vec<(f64, &'static str, f64, f64, f64, f64)> = vec![]; // t, phase, ws MB, private MB, cpu% of one core, gpu%
    let mut last_cpu: Option<(Instant, u64)> = None;
    let mut phase = "idle";
    let start = Instant::now();
    let mut job_seen = false;
    let mut ended = false;
    loop {
        let t = start.elapsed().as_secs_f64();
        while let Ok((l, _, wall)) = rx.try_recv() {
            markers.push((l.clone(), wall));
            if l.starts_with("JOB start") { phase = "job"; job_seen = true; }
            if l.starts_with("JOB end") { phase = "after"; ended = true; }
        }
        if phase == "idle" && t > idle as f64 { phase = "idle-done"; }
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
        let pids = tree(&sys, root);
        if !pids.contains(&root) || sys.process(Pid::from_u32(root)).is_none() { break; }
        let (mut ws, mut pv, mut cpu_ms) = (0u64, 0u64, 0u64);
        for p in &pids { if let Some(pr) = sys.process(Pid::from_u32(*p)) { ws += pr.memory(); pv += pr.virtual_memory(); cpu_ms += pr.accumulated_cpu_time(); } }
        let now = Instant::now();
        let cpu = match last_cpu { Some((tt, c)) => (cpu_ms.saturating_sub(c)) as f64 / 1000.0 / now.duration_since(tt).as_secs_f64() * 100.0, None => 0.0 };
        last_cpu = Some((now, cpu_ms));
        let g = gpu.as_mut().map_or(0.0, |g| g.percent(&pids));
        if t > 1.0 { rows.push((t, phase, ws as f64 / 1048576.0, pv as f64 / 1048576.0, cpu, g)); }
        if (phase == "after" && ended && t > 0.0 && rows.iter().filter(|r| r.1 == "after").count() >= 5) || t > idle as f64 + 60.0 { break; }
        std::thread::sleep(Duration::from_millis(1000));
    }
    let _ = child.kill();
    let unix = |t: std::time::SystemTime| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs_f64();
    let _ = child.wait();

    std::fs::create_dir_all(out).unwrap();
    let mut csv = std::fs::File::create(format!("{out}/{label}.csv")).unwrap();
    writeln!(csv, "t_s,phase,working_set_mb,private_mb,cpu_pct_one_core,gpu_pct").unwrap();
    for r in &rows { writeln!(csv, "{:.1},{},{:.1},{:.1},{:.2},{:.2}", r.0, r.1, r.2, r.3, r.4, r.5).unwrap(); }
    {
        let mut m = std::fs::File::create(format!("{out}/{label}.markers")).unwrap();
        for (l, w) in &markers { writeln!(m, "{:.3} {}", unix(*w), l).unwrap(); }
    }
    let col = |ph: &str, f: &dyn Fn(&(f64, &str, f64, f64, f64, f64)) -> f64| -> f64 { median(&mut rows.iter().filter(|r| r.1 == ph).map(f).collect()) };
    println!("{label}: cold start {cold_ms:.0} ms; threads/cores {ncpu}");
    for ph in ["idle", "job", "after"] {
        println!("  {ph:5}: ram ws {:.1} MB, private {:.1} MB, cpu {:.2}% of one core ({:.2}% of all), gpu {:.2}%  (n={})", col(ph, &|r| r.2), col(ph, &|r| r.3), col(ph, &|r| r.4), col(ph, &|r| r.4) / ncpu, col(ph, &|r| r.5), rows.iter().filter(|r| r.1 == ph).count());
    }
    if !job_seen { println!("  (no JOB markers seen)"); }
}
