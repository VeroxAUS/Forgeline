//! Throwaway spike (Phase 1 step 6): Windows-only frame-time module. Test PC only, never shipped.
//! Listens to the Windows DXGI present events (ETW, the same source PresentMon uses) and records the gaps between
//! one process's present calls. It never opens, injects into or reads the memory of the game process: the events
//! are published by Windows and carry only a process id and a timestamp. Needs admin rights while running.
//!   frametime capture <image name> <secs> <out.csv>
//!   frametime stats <out.csv> <name>:<from_unix_s>-<to_unix_s> ...
#[cfg(not(windows))]
fn main() { eprintln!("frametime is Windows-only"); }

#[cfg(windows)]
fn main() {
    use ferrisetw::provider::Provider;
    use ferrisetw::trace::{TraceTrait, UserTrace};
    use ferrisetw::EventRecord;
    use std::collections::BTreeMap;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use sysinfo::{ProcessesToUpdate, System};

    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(|s| s.as_str()) {
        Some("capture") => {
            let (name, secs, out) = (a[2].to_lowercase(), a[3].parse::<u64>().unwrap(), &a[4]);
            let ev: Arc<Mutex<Vec<(u32, i128, u16)>>> = Arc::new(Mutex::new(Vec::new()));
            let ev2 = ev.clone();
            // Microsoft-Windows-DXGI; event 42 = Present start, 45 = PresentMultiplaneOverlay start
            let p = Provider::by_guid("CA11C036-0102-4A2D-A6AD-F03CFED5D3C9")
                .add_callback(move |r: &EventRecord, _| {
                    let id = r.event_id();
                    if id == 42 || id == 45 {
                        ev2.lock().unwrap().push((r.process_id(), (r.raw_timestamp() as i128 - 116_444_736_000_000_000) * 100, id));
                    }
                })
                .build();
            let trace = UserTrace::new().named("forgeline-frametime".into()).enable(p).start_and_process();
            let Ok(trace) = trace else { eprintln!("could not start the trace (run as administrator)"); std::process::exit(2) };
            let begin = std::time::SystemTime::now();
            std::thread::sleep(std::time::Duration::from_secs(secs));
            drop(trace);
            let mut sys = System::new();
            sys.refresh_processes(ProcessesToUpdate::All, true);
            let pids: Vec<u32> = sys.processes().iter()
                .filter(|(_, p)| p.name().to_string_lossy().to_lowercase() == name).map(|(p, _)| p.as_u32()).collect();
            let evs = ev.lock().unwrap();
            let mut per: BTreeMap<u32, usize> = BTreeMap::new();
            for e in evs.iter() { *per.entry(e.0).or_default() += 1; }
            let target = pids.iter().copied().max_by_key(|p| per.get(p).copied().unwrap_or(0));
            println!("capture started {:?}; {} present events from {} processes; matching pids {:?}", begin, evs.len(), per.len(), pids);
            let Some(t) = target else { println!("no events for {name}"); return };
            let mut f = std::fs::File::create(out).unwrap();
            writeln!(f, "unix_ms,interval_ms").unwrap();
            let mut last: Option<i128> = None;
            for e in evs.iter().filter(|e| e.0 == t && e.2 == 42) {
                if let Some(l) = last { writeln!(f, "{:.3},{:.3}", e.1 as f64 / 1e6, (e.1 - l) as f64 / 1e6).unwrap(); }
                last = Some(e.1);
            }
            println!("wrote {} intervals for pid {t}", per[&t]);
        }
        Some("stats") => {
            let txt = std::fs::read_to_string(&a[2]).unwrap();
            let rows: Vec<(f64, f64)> = txt.lines().skip(1).filter_map(|l| { let (t, i) = l.split_once(',')?; Some((t.parse().ok()?, i.parse().ok()?)) }).collect();
            for w in &a[3..] {
                let (name, r) = w.split_once(':').unwrap();
                let (from, to) = r.split_once('-').unwrap();
                let (from, to): (f64, f64) = (from.parse::<f64>().unwrap() * 1000.0, to.parse::<f64>().unwrap() * 1000.0);
                let mut v: Vec<f64> = rows.iter().filter(|x| x.0 >= from && x.0 < to).map(|x| x.1).collect();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                if v.is_empty() { println!("{name}: no frames"); continue; }
                println!("{name}: {} frames, median {:.2} ms, p99 {:.2} ms, worst {:.2} ms", v.len(), v[v.len() / 2], v[(v.len() * 99 / 100).min(v.len() - 1)], v[v.len() - 1]);
            }
        }
        Some("report") => { // report <csv> <markers> <label>: idle / job / after windows from the sampler's markers
            let txt = std::fs::read_to_string(&a[2]).unwrap();
            let rows: Vec<(f64, f64)> = txt.lines().skip(1).filter_map(|l| { let (t, i) = l.split_once(',')?; Some((t.parse().ok()?, i.parse().ok()?)) }).collect();
            let mk = std::fs::read_to_string(&a[3]).unwrap();
            let at = |key: &str| mk.lines().find(|l| l.contains(key)).and_then(|l| l.split(' ').next()?.parse::<f64>().ok()).map(|t| t * 1000.0);
            let (ready, js, je) = (at("READY"), at("JOB start"), at("JOB end"));
            let (Some(ready), Some(js), Some(je)) = (ready, js, je) else { println!("{}: markers missing", a[4]); return };
            let last = rows.last().map_or(je, |r| r.0);
            for (name, from, to) in [("idle", ready + 3000.0, js), ("job", js, je), ("after", je, last)] {
                let mut v: Vec<f64> = rows.iter().filter(|x| x.0 >= from && x.0 < to).map(|x| x.1).collect();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                if v.is_empty() { println!("{} {name}: no frames", a[4]); continue; }
                println!("{} {name}: {} frames, median {:.2} ms, p99 {:.2} ms, worst {:.2} ms", a[4], v.len(), v[v.len() / 2], v[(v.len() * 99 / 100).min(v.len() - 1)], v[v.len() - 1]);
            }
        }
        _ => eprintln!("usage: frametime capture|stats|report ..."),
    }
}
