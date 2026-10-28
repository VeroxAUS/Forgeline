// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 11 Customise drawer without a window:
//! `customise_shots [out dir] [pixels per point]`. Writes the closed and open drawer at the base,
//! 16:9 and large sizes, edited sliders with a loaded preset, a scrolled drawer, the worst-case
//! long names, an empty preset list, and zoom crops.

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::customise::Customise;
use ui_sample::build_card::BuildCard;
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
    let mut h = ui_sample::build_card::Headless::new();
    let source = SampleSource {
        delay: Duration::from_millis(20),
    };
    let mut app = BuildCard::new(h.ctx(), Arc::new(source));
    h.frame(&mut app, base, ppp);
    assert!(app.cache.wait_idle(Duration::from_secs(5)), "icons arrived");
    let frame =
        |app: &mut BuildCard, h: &mut ui_sample::build_card::Headless, size: Vec2, ppp: f32| {
            h.frame(app, size, ppp);
            h.frame(app, size, ppp);
            h.frame(app, size, ppp)
        };
    let shot = |app: &mut BuildCard,
                h: &mut ui_sample::build_card::Headless,
                name: &str,
                size: Vec2,
                ppp: f32| {
        let path = dir.join(name);
        let r = frame(app, h, size, ppp).write_png(&path);
        println!("{}", path.display());
        r
    };

    // The Build screen with the button, then the drawer open at three sizes.
    shot(&mut app, &mut h, "build-closed.png", base, ppp)?;
    app.custom.open = true;
    shot(&mut app, &mut h, "drawer.png", base, ppp)?;
    shot(&mut app, &mut h, "drawer-16x9.png", wide, 1.0)?;
    shot(&mut app, &mut h, "drawer-large.png", large, 1.0)?;

    // Edited bars, with the preset they match loaded.
    app.custom.load(0);
    shot(&mut app, &mut h, "drawer-preset-loaded.png", base, ppp)?;
    app.custom.comfort[2].value = 9;
    app.custom.assume[1].value = 85;
    shot(&mut app, &mut h, "drawer-edited.png", base, ppp)?;

    // Scrolled to the weights.
    app.custom.demo_scroll = Some(260.0);
    shot(&mut app, &mut h, "drawer-scrolled.png", base, ppp)?;
    app.custom.demo_scroll = Some(0.0);

    // Worst case: long preset and skill names, every preset slot used.
    app.custom = Customise::sample_long();
    app.custom.open = true;
    app.custom.demo_scroll = Some(0.0);
    app.custom.load(0);
    for i in 0..6 {
        app.custom.name = format!("Preset number {i}");
        app.custom.save();
    }
    app.custom.load(0);
    shot(&mut app, &mut h, "drawer-case-long.png", base, ppp)?;
    app.custom.demo_scroll = Some(260.0);
    shot(&mut app, &mut h, "drawer-case-long-scrolled.png", base, ppp)?;
    app.custom.demo_scroll = Some(0.0);

    // No presets saved yet.
    app.custom = Customise::sample();
    app.custom.open = true;
    app.custom.demo_scroll = Some(0.0);
    app.custom.presets.clear();
    shot(&mut app, &mut h, "drawer-empty.png", base, ppp)?;

    // Zoom crops at 3 pixels per point: the drawer's top and a run of slider rows.
    let z = 3.0;
    app.custom = Customise::sample_long();
    app.custom.open = true;
    app.custom.demo_scroll = Some(0.0);
    app.custom.load(0);
    app.custom.comfort[2].value = 9;
    let f = frame(&mut app, &mut h, base, z);
    let (x, w) = (((base.x - 340.0) * z) as usize, (340.0 * z) as usize);
    for (name, y, hh) in [
        ("zoom-top.png", 0.0, 330.0),
        ("zoom-presets.png", 150.0, 250.0),
    ] {
        let path = dir.join(name);
        f.crop(x, (y * z) as usize, w, (hh * z) as usize)
            .write_png(&path)?;
        println!("{}", path.display());
    }
    app.custom.demo_scroll = Some(260.0);
    let f = frame(&mut app, &mut h, base, z);
    let path = dir.join("zoom-sliders.png");
    f.crop(x, (110.0 * z) as usize, w, (330.0 * z) as usize)
        .write_png(&path)?;
    println!("{}", path.display());

    // The delete question in a preset row (the cross turned into `Delete?`).
    app.custom = Customise::sample_long();
    app.custom.open = true;
    app.custom.demo_scroll = Some(0.0);
    app.custom.ask_delete(1);
    shot(&mut app, &mut h, "drawer-confirm-delete.png", base, ppp)?;
    app.custom.ask_delete(0);
    shot(
        &mut app,
        &mut h,
        "drawer-confirm-delete-long.png",
        base,
        ppp,
    )?;

    // The Customise button on the Must have row (the picked placement), at the base size, 16:9,
    // while updating (the crowded case) and with the drawer open.
    app.custom = Customise::sample();
    app.shell.status = Status::Synced;
    shot(&mut app, &mut h, "place-a-1000x700.png", base, ppp)?;
    shot(&mut app, &mut h, "place-a-16x9.png", wide, 1.0)?;
    app.shell.status = Status::Recomputing;
    shot(&mut app, &mut h, "place-a-updating.png", base, ppp)?;
    app.custom.open = true;
    shot(&mut app, &mut h, "place-a-open.png", base, ppp)?;
    app.shell.status = Status::Synced;
    Ok(())
}
