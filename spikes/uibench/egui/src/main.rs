//! Throwaway spike (Phase 1 step 6): minimal pure-Rust UI candidate. Dark table of 300 rows and a button
//! that starts a background job at lowest thread priority. Prints "READY" once after the first frame.
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct App {
    rows: Vec<[String; 5]>,
    running: Arc<AtomicBool>,
    ready: bool,
    start: std::time::Instant,
    auto_job_at: Option<f64>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let auto = self.auto_job_at.map_or(false, |t| self.start.elapsed().as_secs_f64() > t);
                if auto { self.auto_job_at = None; }
                if (ui.button("Run background job").clicked() || auto) && !self.running.swap(true, Ordering::SeqCst) {
                    let (flag, ctx) = (self.running.clone(), ui.ctx().clone());
                    std::thread::spawn(move || {
                        println!("JOB start");
                        low_priority();
                        let mut h = 0u64;
                        let t = std::time::Instant::now();
                        while t.elapsed().as_secs() < 5 {
                            for i in 0..1_000_000u64 { h = h.wrapping_mul(31).wrapping_add(i); }
                        }
                        std::hint::black_box(h);
                        println!("JOB end");
                        flag.store(false, Ordering::SeqCst);
                        ctx.request_repaint();
                    });
                }
                ui.label(if self.running.load(Ordering::SeqCst) { "job running" } else { "idle" });
            });
        });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("g").striped(true).show(ui, |ui| {
                    for r in &self.rows {
                        for c in r { ui.label(c); }
                        ui.end_row();
                    }
                });
            });
        });
        if let Some(t) = self.auto_job_at { ui.ctx().request_repaint_after(std::time::Duration::from_secs_f64((t - self.start.elapsed().as_secs_f64()).max(0.05))); }
        if !self.ready {
            self.ready = true;
            println!("READY");
        }
    }
}

#[cfg(windows)]
fn low_priority() {
    use windows_sys::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_LOWEST};
    unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_LOWEST); }
}
#[cfg(not(windows))]
fn low_priority() {}

fn main() -> eframe::Result {
    let rows = (0..300).map(|i| [format!("Item {i}"), format!("{}", i * 7 % 100), format!("{}", i * 13 % 250), "Rare 8".into(), format!("Slot {}", i % 4)]).collect();
    let opts = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]), ..Default::default() };
    eframe::run_native("uibench", opts, Box::new(|cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Ok(Box::new(App { rows, running: Arc::new(AtomicBool::new(false)), ready: false, start: std::time::Instant::now(), auto_job_at: std::env::var("UIBENCH_JOB_AT").ok().and_then(|v| v.parse().ok()) }))
    }))
}
