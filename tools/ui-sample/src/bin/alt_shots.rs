// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 8 switch and alternatives row without a window:
//! `alt_shots [out dir] [pixels per point]`. Writes the base, 16:9 and large sizes, every
//! alternative chosen, Multiplayer, the recomputing state and the worst cases (nothing
//! unavailable, everything but Best unavailable, a long summary line, a must-have out of reach).

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{Alt, BuildCard, BuildData, Headless, MustHave, PlayMode};
use ui_sample::icons::SampleSource;
use ui_sample::shell::Status;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    let ppp: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(2.0);
    std::fs::create_dir_all(&dir)?;
    let base = Vec2::new(1000.0, 700.0);
    let wide = Vec2::new(1600.0, 900.0);
    let large = Vec2::new(2800.0, 1575.0);
    let mut h = Headless::new();
    let source = SampleSource {
        delay: Duration::from_millis(20),
    };
    let mut app = BuildCard::new(h.ctx(), Arc::new(source));
    h.frame(&mut app, base, ppp);
    assert!(app.cache.wait_idle(Duration::from_secs(5)), "icons arrived");
    let shot = |app: &mut BuildCard, h: &mut Headless, name: &str, size: Vec2, ppp: f32| {
        h.frame(app, size, ppp);
        let path = dir.join(name);
        let r = h.frame(app, size, ppp).write_png(&path);
        println!("{}", path.display());
        r
    };
    let sample = BuildData::sample();

    shot(&mut app, &mut h, "alt-base.png", base, ppp)?;
    shot(&mut app, &mut h, "alt-16x9.png", wide, 1.0)?;
    shot(&mut app, &mut h, "alt-large.png", large, 1.0)?;

    // Every alternative chosen, at the base size.
    let mut all_ready = sample.clone();
    all_ready.unavailable = [None; 6];
    for a in Alt::ALL {
        app.data = all_ready.clone();
        app.data.alt = a;
        app.data.apply_variant();
        shot(
            &mut app,
            &mut h,
            &format!("alt-pick-{a:?}.png").to_lowercase(),
            base,
            ppp,
        )?;
    }
    // Multiplayer, and Multiplayer on a wide window.
    app.data = all_ready.clone();
    app.data.mode = PlayMode::Multiplayer;
    app.data.alt = Alt::Safe;
    app.data.apply_variant();
    shot(&mut app, &mut h, "alt-multi.png", base, ppp)?;
    shot(&mut app, &mut h, "alt-multi-16x9.png", wide, 1.0)?;
    // Recomputing after a click.
    app.shell.status = Status::Recomputing;
    shot(&mut app, &mut h, "alt-recomputing.png", base, ppp)?;
    app.shell.status = Status::Synced;

    // Worst cases.
    let mut few = sample.clone();
    few.unavailable = [
        None,
        Some("No safer build exists with your must-haves."),
        Some("Needs more armour unlocked."),
        Some("Pick a monster first."),
        Some("Needs more monsters unlocked."),
        Some("No Gogma gear unlocked yet."),
    ];
    app.data = few;
    shot(&mut app, &mut h, "alt-case-only-best.png", base, ppp)?;
    let mut oor = all_ready.clone();
    oor.must_haves = vec![MustHave::new("Weakness exploit", 5, 5)];
    oor.must_haves[0].reachable = Some(3);
    oor.mode = PlayMode::Multiplayer;
    oor.alt = Alt::Gogma;
    oor.apply_variant();
    app.data = oor;
    shot(&mut app, &mut h, "alt-case-out-of-reach.png", base, ppp)?;
    shot(
        &mut app,
        &mut h,
        "alt-case-out-of-reach-16x9.png",
        wide,
        1.0,
    )?;
    Ok(())
}
