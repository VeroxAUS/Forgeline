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

#[test]
fn build_card_shows_icons_once_loaded_and_keeps_the_build_while_recomputing() {
    use egui::Vec2;
    use std::sync::Arc;
    use std::time::Duration;
    use ui_sample::build_card::{BuildCard, BuildData, Headless};
    use ui_sample::icons::SampleSource;
    use ui_sample::shell::Status;

    let size = Vec2::new(1000.0, 700.0);
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    assert_eq!(app.data.pieces_missing(), 2);
    h.frame(&mut app, size, 1.0);
    assert!(app.cache.wait_idle(Duration::from_secs(5)));
    // Five icons arrive; the sixth (id 6) has none and keeps its glyph.
    assert_eq!((app.cache.stats().ready, app.cache.stats().missing), (5, 1));
    let ready = h.frame(&mut app, size, 1.0);
    assert_eq!((ready.width, ready.height), (1000, 700));

    // Recomputing changes the status line and the summary, not the rows.
    app.shell.status = Status::Recomputing;
    let recomputing = h.frame(&mut app, size, 1.0);
    assert_ne!(ready.rgba, recomputing.rgba);

    // An empty build still draws, and a larger window fills its size.
    app.shell.status = Status::Synced;
    app.data = BuildData {
        weapon: None,
        armour: Default::default(),
        charm: None,
        skills: Vec::new(),
        pinned_style: Default::default(),
        damage: BuildData::sample().damage,
        toughness: BuildData::sample().toughness,
        must_haves: Vec::new(),
        chip_w: ui_sample::build_card::CHIP_W,
        headline_style: Default::default(),
        mode: Default::default(),
        alt: Default::default(),
        unavailable: [None; 6],
        monster: None,
        recent: Vec::new(),
        needs: Vec::new(),
        chains: Vec::new(),
    };
    assert_ne!(ready.rgba, h.frame(&mut app, size, 1.0).rgba);
    let big = h.frame(&mut app, Vec2::new(1400.0, 900.0), 1.0);
    assert_eq!((big.width, big.height), (1400, 900));
}

#[test]
fn window_scale_follows_size_within_limits() {
    use egui::Vec2;
    use ui_sample::shell::{scale_for, MAX_SCALE};
    assert_eq!(scale_for(Vec2::new(1000.0, 700.0)), 1.0);
    assert_eq!(
        scale_for(Vec2::new(800.0, 500.0)),
        1.0,
        "never below the design size"
    );
    assert!((scale_for(Vec2::new(2000.0, 1400.0)) - 2.0).abs() < 1e-6);
    // The tighter axis decides, so a wide window does not overflow vertically.
    assert!((scale_for(Vec2::new(3000.0, 1050.0)) - 1.5).abs() < 1e-6);
    assert_eq!(scale_for(Vec2::new(9000.0, 9000.0)), MAX_SCALE);
}

#[test]
fn larger_window_scales_the_content() {
    use egui::Vec2;
    use std::sync::Arc;
    use std::time::Duration;
    use ui_sample::build_card::{BuildCard, Headless};
    use ui_sample::icons::SampleSource;
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    h.frame(&mut app, Vec2::new(1000.0, 700.0), 1.0);
    assert_eq!(h.ctx().zoom_factor(), 1.0);
    h.frame(&mut app, Vec2::new(2000.0, 1400.0), 1.0);
    assert!((h.ctx().zoom_factor() - 2.0).abs() < 0.01);
}

#[test]
fn skills_strip_draws_in_a_side_strip_or_under_the_card() {
    use egui::Vec2;
    use std::{sync::Arc, time::Duration};
    use ui_sample::build_card::{BuildCard, Headless, Skill};
    use ui_sample::icons::SampleSource;
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    h.frame(&mut app, Vec2::new(1000.0, 700.0), 1.0);
    assert!(app.cache.wait_idle(Duration::from_secs(5)));
    let with = h.frame(&mut app, Vec2::new(1000.0, 700.0), 1.0);

    // Changing one skill's level changes the picture; a build with none shows the empty line.
    app.data.skills[0].level -= 1;
    assert_ne!(
        with.rgba,
        h.frame(&mut app, Vec2::new(1000.0, 700.0), 1.0).rgba
    );
    app.data.skills.clear();
    assert_ne!(
        with.rgba,
        h.frame(&mut app, Vec2::new(1000.0, 700.0), 1.0).rgba
    );

    // A window too narrow for the side strip puts it under the card and still draws.
    app.data.skills =
        vec![Skill::normal("A skill with a name far too long to fit its column", 2, 7); 3];
    let narrow = h.frame(&mut app, Vec2::new(800.0, 900.0), 1.0);
    assert_eq!((narrow.width, narrow.height), (800, 900));
}

#[test]
fn long_builds_collapse_set_and_group_skills_and_use_a_second_column_when_wide() {
    use egui::Vec2;
    use std::{sync::Arc, time::Duration};
    use ui_sample::build_card::{sample_shaped_skills, BuildCard, Headless, PinnedStyle};
    use ui_sample::icons::SampleSource;
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    // 19 normal skills plus 3 set and group skills: the shape of a Gogma endgame build.
    app.data.skills = sample_shaped_skills(19, 2, 1);
    let base = Vec2::new(1000.0, 700.0);
    let wide = Vec2::new(1920.0, 1080.0);

    app.data.pinned_style = PinnedStyle::Tiers;
    h.frame(&mut app, base, 1.0);
    assert!(
        app.skills_shown() < 22,
        "full tiers leave some out at the base size"
    );
    h.frame(&mut app, wide, 1.0);
    assert_eq!(app.skills_shown(), 22, "a wide window has a second column");

    // Auto collapses the set and group rows just enough that everything shows.
    app.data.pinned_style = PinnedStyle::Auto;
    h.frame(&mut app, base, 1.0);
    assert_eq!(app.skills_shown(), 22);

    // A short list keeps its tiers: the Auto frame equals the full-tiers frame.
    app.data.skills = sample_shaped_skills(8, 1, 1);
    let auto = h.frame(&mut app, base, 1.0);
    app.data.pinned_style = PinnedStyle::Tiers;
    assert_eq!(auto.rgba, h.frame(&mut app, base, 1.0).rgba);
}

#[test]
fn headline_options_draw_differently_and_unknown_draws_no_fill() {
    use egui::Vec2;
    use std::{sync::Arc, time::Duration};
    use ui_sample::build_card::{BuildCard, Headless, HeadlineStyle};
    use ui_sample::icons::SampleSource;
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    let size = Vec2::new(1000.0, 700.0);
    h.frame(&mut app, size, 1.0);
    assert!(app.cache.wait_idle(Duration::from_secs(5)));
    let mut seen = Vec::new();
    for style in [
        HeadlineStyle::Word,
        HeadlineStyle::Compare,
        HeadlineStyle::Number,
    ] {
        app.data.headline_style = style;
        seen.push(h.frame(&mut app, size, 1.0).rgba);
    }
    assert!(seen[0] != seen[1] && seen[1] != seen[2] && seen[0] != seen[2]);
    app.data.headline_style = HeadlineStyle::Compare;
    app.data.damage.fill = None;
    assert_ne!(seen[1], h.frame(&mut app, size, 1.0).rgba);
}

#[test]
fn alternatives_fall_back_and_move_the_headline() {
    use ui_sample::build_card::{Alt, BuildData, PlayMode};
    let mut d = BuildData::sample();
    let best = d.damage.fill;
    d.alt = Alt::Gogma; // not unlocked in the sample
    d.sync_alt();
    assert_eq!(
        d.alt,
        Alt::Best,
        "an unavailable alternative falls back to Best"
    );
    d.alt = Alt::Safe;
    d.apply_variant();
    assert!(d.damage.fill < best, "Safe gives up damage");
    assert!(d.toughness.fill > BuildData::sample().toughness.fill);
    let solo = d.toughness.fill;
    d.mode = PlayMode::Multiplayer;
    d.apply_variant();
    assert!(d.toughness.fill > solo);
}

#[test]
fn monster_selector_opens_and_the_pick_names_the_build() {
    use egui::Vec2;
    use std::{sync::Arc, time::Duration};
    use ui_sample::build_card::{Alt, BuildCard, BuildData, Headless, Pick, Popup, ROSTER};
    use ui_sample::icons::SampleSource;
    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    assert!(
        BuildData::sample().unavailable[Alt::Monster as usize].is_none(),
        "the Monster cell is enabled once the selector exists"
    );
    let size = Vec2::new(1000.0, 700.0);
    h.frame(&mut app, size, 1.0);
    assert!(app.cache.wait_idle(Duration::from_secs(5)));
    let closed = h.frame(&mut app, size, 1.0).rgba;
    app.selector.open(None);
    app.popup = Some(Popup::Monster);
    h.frame(&mut app, size, 1.0);
    let open = h.frame(&mut app, size, 1.0).rgba;
    assert_ne!(closed, open, "the panel draws");
    // Searching changes the list; the panel stays inside the window at a small size too.
    app.selector.query = "guardian".into();
    h.frame(&mut app, size, 1.0);
    assert_ne!(open, h.frame(&mut app, size, 1.0).rgba);
    h.frame(&mut app, Vec2::new(1000.0, 700.0), 3.0);
    // A pick shows in the breadcrumb (the frame changes) and the build is for that monster.
    app.popup = None;
    app.data.monster = Some(Pick::new(ROSTER.len() - 1, false));
    app.data.alt = Alt::Monster;
    app.data.apply_variant();
    h.frame(&mut app, size, 1.0);
    assert_ne!(closed, h.frame(&mut app, size, 1.0).rgba);
}

#[test]
fn shopping_views_filter_and_missing_click_through() {
    use egui::Vec2;
    use std::sync::Arc;
    use std::time::Duration;
    use ui_sample::build_card::shopping::sample;
    use ui_sample::build_card::{BuildCard, Headless, ShopView};
    use ui_sample::icons::SampleSource;
    use ui_sample::shell::Section;

    let mut h = Headless::new();
    let mut app = BuildCard::new(
        h.ctx(),
        Arc::new(SampleSource {
            delay: Duration::from_millis(1),
        }),
    );
    app.data
        .set_needs(sample::endgame(), vec![(0, sample::CHAIN)]);
    let size = Vec2::new(1000.0, 700.0);
    h.frame(&mut app, size, 1.0);
    assert!(app.cache.wait_idle(Duration::from_secs(5)));
    let build = h.frame(&mut app, size, 1.0).rgba;
    app.shell.section = Section::Shopping;
    let mut frames = Vec::new();
    for view in ShopView::ALL {
        app.shopping.view = view;
        h.frame(&mut app, size, 1.0);
        frames.push(h.frame(&mut app, size, 1.0).rgba);
    }
    // The three views and the Build screen all look different.
    for i in 0..frames.len() {
        assert_ne!(build, frames[i]);
        for j in i + 1..frames.len() {
            assert_ne!(frames[i], frames[j], "{i} vs {j}");
        }
    }
    // The click on `Missing N` opens Shopping by piece, filtered to that piece.
    app.shopping.show_piece(2);
    app.shell.section = Section::Shopping;
    assert_eq!(app.shopping.view, ShopView::Piece);
    assert_eq!(app.shopping.filter, Some(2));
    h.frame(&mut app, size, 1.0);
    let filtered = h.frame(&mut app, size, 1.0).rgba;
    assert_ne!(filtered, frames[1]);
    // Nothing missing: the list says so, at the largest scale too.
    app.data.set_needs(Vec::new(), Vec::new());
    app.shopping.filter = None;
    h.frame(&mut app, Vec2::new(3000.0, 2100.0), 1.0);
    h.frame(&mut app, size, 1.0);
}
