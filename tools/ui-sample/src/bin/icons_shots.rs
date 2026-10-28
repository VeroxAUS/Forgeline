// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 3 icon demo without a window: `icons_shots [out dir] [pixels per point]`.
//! Writes the first frame (icons still loading) and the frame after they arrived.

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::icons::{Demo, Headless, SampleSource};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    let ppp: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(2.0);
    std::fs::create_dir_all(&dir)?;
    let size = Vec2::new(1000.0, 700.0);
    let mut h = Headless::new();
    let source = SampleSource {
        delay: Duration::from_millis(20),
    };
    let mut demo = Demo::new(&h.ctx, Arc::new(source));

    let loading = dir.join("icons-loading.png");
    h.frame(&mut demo, size, ppp);
    h.frame(&mut demo, size, ppp).write_png(&loading)?;
    println!("{}", loading.display());

    assert!(
        demo.cache.wait_idle(Duration::from_secs(5)),
        "icons arrived"
    );
    let ready = dir.join("icons-ready.png");
    h.frame(&mut demo, size, ppp).write_png(&ready)?;
    println!("{}", ready.display());
    Ok(())
}
