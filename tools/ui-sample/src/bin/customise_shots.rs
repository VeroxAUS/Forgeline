// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 11 Customise drawer without a window:
//! `customise_shots [out dir] [pixels per point]`. Writes the closed button, the drawer at the
//! base, 16:9 and large sizes, an edited state, a long preset list, the naming row and the
//! scrolled weights.

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::BuildCard;
use ui_sample::icons::SampleSource;

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
    let shot = |app: &mut BuildCard,
                h: &mut ui_sample::build_card::Headless,
                name: &str,
                size: Vec2,
                ppp: f32| {
        for _ in 0..3 {
            h.frame(app, size, ppp);
        }
        let path = dir.join(name);
        let r = h.frame(app, size, ppp).write_png(&path);
        println!("{}", path.display());
        r
    };

    shot(&mut app, &mut h, "closed.png", base, ppp)?;
    app.customise.open = true;
    shot(&mut app, &mut h, "drawer.png", base, ppp)?;
    shot(&mut app, &mut h, "drawer-16x9.png", wide, 1.0)?;
    shot(&mut app, &mut h, "drawer-large.png", large, 1.0)?;

    // Some values moved: accent values, reset marks, EDITED, the dot on the button.
    for (i, v) in [(0, 9), (2, 8), (5, 0), (12, 7)] {
        app.customise.weights[i].value = v;
    }
    app.customise.assume[0].value = 85;
    shot(&mut app, &mut h, "drawer-edited.png", base, ppp)?;

    // Under the "Boss rush with a very long preset name" preset, edited.
    app.customise.active = 2;
    shot(&mut app, &mut h, "drawer-preset-edited.png", base, ppp)?;

    // Full preset list (five) and the naming row.
    app.customise.reset_all();
    for n in 3..=5 {
        app.customise.start_naming();
        app.customise.naming = Some(format!("Preset {n}"));
        app.customise.save_demo();
    }
    shot(&mut app, &mut h, "drawer-presets-full.png", base, ppp)?;
    app.customise.reset_all();
    app.customise.presets.truncate(1);
    app.customise.start_naming();
    shot(&mut app, &mut h, "drawer-naming.png", base, ppp)?;
    app.customise.naming = None;

    // Scrolled to the last weights, long name.
    app.customise.demo_scroll = Some(380.0);
    shot(&mut app, &mut h, "drawer-scrolled.png", base, ppp)?;
    Ok(())
}
