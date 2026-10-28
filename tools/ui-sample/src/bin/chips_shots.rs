// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 7 must-have chips without a window: `chips_shots [out dir] [pixels per point]`.
//! Writes the base, 16:9 and large sizes, the recomputing state, the three popups and the worst
//! cases (none, full row, overflow, a long name, out of reach).

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{BuildCard, BuildData, Headless, MustHave, Popup};
use ui_sample::icons::SampleSource;
use ui_sample::shell::Status;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    let ppp: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(2.0);
    std::fs::create_dir_all(&dir)?;
    let base = Vec2::new(1000.0, 700.0);
    let mut h = Headless::new();
    let source = SampleSource {
        delay: Duration::from_millis(20),
    };
    let mut app = BuildCard::new(h.ctx(), Arc::new(source));
    h.frame(&mut app, base, ppp);
    assert!(app.cache.wait_idle(Duration::from_secs(5)), "icons arrived");
    let save = |name: &str, f: ui_sample::render::Frame| -> std::io::Result<()> {
        let path = dir.join(name);
        f.write_png(&path)?;
        println!("{}", path.display());
        Ok(())
    };
    let shot = |app: &mut BuildCard, h: &mut Headless, name: &str, size: Vec2, ppp: f32| {
        h.frame(app, size, ppp);
        save(name, h.frame(app, size, ppp))
    };
    shot(&mut app, &mut h, "chips-base.png", base, ppp)?;
    shot(
        &mut app,
        &mut h,
        "chips-16x9.png",
        Vec2::new(1600.0, 900.0),
        1.0,
    )?;
    shot(
        &mut app,
        &mut h,
        "chips-large.png",
        Vec2::new(2800.0, 1575.0),
        1.0,
    )?;
    app.shell.status = Status::Recomputing;
    shot(&mut app, &mut h, "chips-recomputing.png", base, ppp)?;
    app.shell.status = Status::Synced;

    for (name, popup) in [("edit", Popup::Edit(0)), ("add", Popup::Add)] {
        app.popup = Some(popup);
        shot(&mut app, &mut h, &format!("popup-{name}.png"), base, ppp)?;
    }
    app.popup = None;

    let m = |n, l, x| MustHave::new(n, l, x);
    let names = [
        ("Weakness exploit", 5),
        ("Critical eye", 5),
        ("Sharpshot", 3),
        ("Marksman", 3),
        ("Ammo saver", 3),
        ("Stamina surge", 3),
        ("Evade window", 5),
    ];
    let with =
        |n: usize| -> Vec<MustHave> { names.iter().take(n).map(|(a, b)| m(a, *b, *b)).collect() };
    let sample = BuildData::sample();
    let mut cases: Vec<(&str, Vec<MustHave>)> = vec![
        ("none", vec![]),
        ("three", with(3)),
        ("overflow-7", with(7)),
        (
            "long-names",
            vec![
                m("Convert Thunder Resistance", 1, 1),
                m("Convert Water Resistance", 1, 1),
                m("Weakness exploit", 5, 5),
            ],
        ),
    ];
    let mut oor = with(2);
    oor[1].reachable = Some(3);
    oor[1].level = 5;
    cases.push(("out-of-reach", oor.clone()));
    for (name, list) in cases {
        app.data = sample.clone();
        app.data.must_haves = list;
        app.data.sync_must();
        shot(&mut app, &mut h, &format!("case-{name}.png"), base, ppp)?;
    }
    // Overflow popup and the out-of-reach chip hidden in the overflow.
    app.data = sample.clone();
    app.data.must_haves = with(7);
    app.data.must_haves[6].level = 5;
    app.data.must_haves[6].reachable = Some(2);
    app.data.sync_must();
    h.frame(&mut app, base, ppp);
    app.popup = Some(Popup::More);
    shot(&mut app, &mut h, "popup-more.png", base, ppp)?;
    Ok(())
}
