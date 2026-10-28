// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use ui_sample::{render_frame, State, Variant};

#[test]
fn every_variant_renders_something_distinct() {
    let frames: Vec<_> = Variant::ALL
        .iter()
        .map(|v| render_frame(&mut State::new(*v), 1.0))
        .collect();
    for f in &frames {
        assert_eq!((f.width, f.height), (1000, 700));
        // Text and shapes were drawn: more than one distinct colour on screen.
        let first = f.rgba[0];
        assert!(f.rgba.iter().any(|p| *p != first));
    }
    assert_ne!(frames[0].rgba, frames[1].rgba);
    assert_ne!(frames[1].rgba, frames[2].rgba);
    // Every variant, the D family included, is visibly different from the others.
    for i in 0..frames.len() {
        for j in i + 1..frames.len() {
            assert_ne!(frames[i].rgba, frames[j].rgba, "{i} vs {j}");
        }
    }
}

#[test]
fn shell_renders_each_section_and_resizes() {
    use egui::Vec2;
    use ui_sample::shell::{render_frame, Section, Shell, Status, MIN_SIZE};
    let mut frames = Vec::new();
    for sec in Section::ALL {
        let mut shell = Shell {
            section: sec,
            status: Status::Synced,
        };
        let f = render_frame(&mut shell, MIN_SIZE, 1.0);
        assert_eq!((f.width, f.height), (1000, 700));
        frames.push(f.rgba);
    }
    // Each section looks different (selected rail item, title).
    for i in 0..frames.len() {
        for j in i + 1..frames.len() {
            assert_ne!(frames[i], frames[j], "{i} vs {j}");
        }
    }
    // The status line changes with the status, and a larger window fills its size.
    let mut shell = Shell::new();
    let a = render_frame(&mut shell, MIN_SIZE, 1.0);
    shell.status = Status::Problem("Couldn't read the save.".into());
    let b = render_frame(&mut shell, MIN_SIZE, 1.0);
    assert_ne!(a.rgba, b.rgba);
    let big = render_frame(&mut shell, Vec2::new(1440.0, 900.0), 1.0);
    assert_eq!((big.width, big.height), (1440, 900));
}

#[test]
fn icons_load_in_background_cache_and_fall_back() {
    use std::sync::Arc;
    use std::time::Duration;
    use ui_sample::icons::{Demo, Headless, IconState, SampleSource, MAX_EDGE};
    let size = egui::Vec2::new(1000.0, 700.0);
    let mut h = Headless::new();
    let source = SampleSource {
        delay: Duration::from_millis(10),
    };
    let mut demo = Demo::new(&h.ctx, Arc::new(source));
    let before = h.frame(&mut demo, size, 1.0);
    // Everything visible was asked for once; nothing is ready yet or has failed.
    assert!(demo.cache.stats().pending + demo.cache.stats().ready > 0);
    assert!(demo.cache.wait_idle(Duration::from_secs(5)));
    let after = h.frame(&mut demo, size, 1.0);
    assert_ne!(before.rgba, after.rgba, "icons replaced the glyphs");
    let s = demo.cache.stats();
    assert_eq!((s.ready, s.missing, s.pending), (7, 1, 0));
    assert_eq!(demo.cache.state(6), IconState::Missing);
    // 7 icons, each downscaled from 128 to MAX_EDGE.
    assert_eq!(s.bytes, 7 * MAX_EDGE * MAX_EDGE * 4);
    // A missing icon is remembered: more frames ask for nothing and change nothing.
    let again = h.frame(&mut demo, size, 1.0);
    assert_eq!(after.rgba, again.rgba);
    assert_eq!(demo.cache.stats(), s);
}

#[test]
fn icon_cache_stays_under_its_cap_and_drops_the_least_recently_drawn() {
    use std::sync::Arc;
    use std::time::Duration;
    use ui_sample::icons::{IconCache, IconState, SampleSource, MAX_EDGE};
    let ctx = egui::Context::default();
    let one = MAX_EDGE * MAX_EDGE * 4;
    let source = SampleSource {
        delay: Duration::ZERO,
    };
    let mut cache = IconCache::with_cap(&ctx, Arc::new(source), 3 * one);
    let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::splat(34.0));
    let painter = egui::Painter::new(
        ctx.clone(),
        egui::LayerId::background(),
        egui::Rect::EVERYTHING,
    );
    // Draw icons 0..5 one frame at a time, each frame leaving room for the one before.
    for id in 0..5u32 {
        cache.poll();
        cache.tile(&painter, rect, id, ui_sample::icons::Glyph::Weapon);
        assert!(cache.wait_idle(Duration::from_secs(5)));
        assert!(cache.stats().bytes <= 3 * one + one, "cap holds");
    }
    cache.poll();
    cache.poll();
    assert!(cache.stats().bytes <= 3 * one);
    assert_eq!(cache.state(0), IconState::NotAsked, "oldest was dropped");
    assert_eq!(cache.state(4), IconState::Ready, "newest is kept");
}

#[test]
fn downscale_halves_by_box_average() {
    use ui_sample::icons::{downscale, RawIcon};
    let icon = RawIcon {
        width: 4,
        height: 2,
        rgba: [[0u8, 0, 0, 255], [100, 100, 100, 255]]
            .iter()
            .cycle()
            .take(8)
            .flatten()
            .copied()
            .collect(),
    };
    let small = downscale(icon.clone(), 2);
    assert_eq!((small.width, small.height), (2, 1));
    assert_eq!(small.rgba[0], 50);
    assert_eq!(
        downscale(icon.clone(), 8),
        icon,
        "small icons are untouched"
    );
}
