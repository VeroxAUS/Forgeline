// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 6 headline blocks without a window: `headline_shots [out dir] [pixels per point]`.
//! Writes the three display options at the base size, a 16:9 size and a large size, and the
//! worst cases (unknown, no gear to compare, worse than gear, longest word, empty and full bars).

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{BuildCard, BuildData, Headless, HeadlineStyle};
use ui_sample::icons::SampleSource;

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
    for (name, style) in [
        ("word", HeadlineStyle::Word),
        ("compare", HeadlineStyle::Compare),
        ("number", HeadlineStyle::Number),
    ] {
        app.data.headline_style = style;
        save(&format!("{name}-base.png"), h.frame(&mut app, base, ppp))?;
        // 16:9, not maximised: 1600 by 900 pixels at scale 1 (the UI scales 1.29 times).
        save(
            &format!("{name}-16x9.png"),
            h.frame(&mut app, Vec2::new(1600.0, 900.0), 1.0),
        )?;
        save(
            &format!("{name}-large.png"),
            h.frame(&mut app, Vec2::new(2800.0, 1575.0), 1.0),
        )?;
    }

    // Worst cases, in the Compare option.
    app.data.headline_style = HeadlineStyle::Compare;
    let sample = BuildData::sample();
    let cases: [(&str, [ui_sample::build_card::Headline; 2]); 5] = [
        (
            "longest-word-big-delta",
            [
                ui_sample::build_card::Headline {
                    fill: Some(1.0),
                    current: Some(0.2),
                    delta: Some(412),
                    ..sample.damage
                },
                ui_sample::build_card::Headline {
                    fill: Some(0.98),
                    current: Some(0.3),
                    delta: Some(228),
                    ..sample.toughness
                },
            ],
        ),
        (
            "unknown",
            [
                ui_sample::build_card::Headline {
                    fill: None,
                    current: None,
                    delta: None,
                    ..sample.damage
                },
                ui_sample::build_card::Headline {
                    fill: None,
                    current: None,
                    delta: None,
                    ..sample.toughness
                },
            ],
        ),
        (
            "no-gear-to-compare",
            [
                ui_sample::build_card::Headline {
                    current: None,
                    delta: None,
                    ..sample.damage
                },
                ui_sample::build_card::Headline {
                    fill: Some(0.9),
                    current: None,
                    delta: None,
                    ..sample.toughness
                },
            ],
        ),
        (
            "worse-than-gear",
            [
                ui_sample::build_card::Headline {
                    fill: Some(0.4),
                    current: Some(0.46),
                    delta: Some(-13),
                    ..sample.damage
                },
                ui_sample::build_card::Headline {
                    fill: Some(0.3),
                    current: Some(0.31),
                    delta: Some(-3),
                    ..sample.toughness
                },
            ],
        ),
        (
            "empty-bar-and-tick-at-edge",
            [
                ui_sample::build_card::Headline {
                    fill: Some(0.0),
                    current: Some(0.0),
                    delta: Some(0),
                    ..sample.damage
                },
                ui_sample::build_card::Headline {
                    fill: Some(1.0),
                    current: Some(1.0),
                    delta: Some(0),
                    ..sample.toughness
                },
            ],
        ),
    ];
    for (name, [d, t]) in cases {
        app.data.damage = d;
        app.data.toughness = t;
        save(&format!("case-{name}.png"), h.frame(&mut app, base, ppp))?;
    }
    Ok(())
}
