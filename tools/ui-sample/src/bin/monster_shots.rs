// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 9 monster selector without a window:
//! `monster_shots [out dir] [pixels per point]`. Writes the closed and open selector at the base,
//! 16:9 and large sizes, a search, no match, recent picks, the Tempered switch (on and not
//! allowed), and the worst cases (longest name in the breadcrumb, the reserved weak-element column
//! filled with invented values).

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{remember, Alt, BuildCard, Pick, Popup, ROSTER};
use ui_sample::icons::SampleSource;

fn index(name: &str) -> usize {
    ROSTER
        .iter()
        .position(|m| m.name == name)
        .expect("in roster")
}

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
        h.frame(app, size, ppp);
        h.frame(app, size, ppp);
        let path = dir.join(name);
        let r = h.frame(app, size, ppp).write_png(&path);
        println!("{}", path.display());
        r
    };
    let open = |app: &mut BuildCard| {
        let cur = app.data.monster;
        app.selector.open(cur);
        app.selector.fresh = false;
        app.popup = Some(Popup::Monster);
    };

    // Nothing picked yet: the Monster cell is enabled, nothing else changes.
    shot(&mut app, &mut h, "monster-start.png", base, ppp)?;
    open(&mut app);
    shot(&mut app, &mut h, "monster-open.png", base, ppp)?;
    shot(&mut app, &mut h, "monster-open-16x9.png", wide, 1.0)?;
    shot(&mut app, &mut h, "monster-open-large.png", large, 1.0)?;

    // Searching, and a search with no match.
    app.selector.query = "gu".into();
    shot(&mut app, &mut h, "monster-search.png", base, ppp)?;
    app.selector.query = "arch".into();
    shot(&mut app, &mut h, "monster-search-arch.png", base, ppp)?;
    app.selector.query = "zzz".into();
    shot(&mut app, &mut h, "monster-nomatch.png", base, ppp)?;
    app.selector.query.clear();

    // Tempered switch on: only monsters with a Tempered version are listed.
    app.selector.tempered = true;
    shot(&mut app, &mut h, "monster-tempered-filter.png", base, ppp)?;
    // The same search with the switch on and off: Gogmazios has no Tempered version.
    app.selector.query = "go".into();
    shot(&mut app, &mut h, "monster-tempered-go-on.png", base, ppp)?;
    app.selector.tempered = false;
    shot(&mut app, &mut h, "monster-tempered-go-off.png", base, ppp)?;
    app.selector.tempered = true;
    app.selector.query.clear();
    app.selector.tempered = false;

    // Keyboard cursor near the end of the list (the list scrolls to it).
    app.selector.reveal();
    app.selector.cursor = ROSTER
        .iter()
        .take(index("Zoh Shia"))
        .filter(|m| m.group == ui_sample::build_card::Group::Base)
        .count();
    shot(&mut app, &mut h, "monster-scrolled.png", base, ppp)?;
    app.selector.cursor = 0;

    // A pick: the breadcrumb names it, the build is recomputing, recent picks sit on top.
    app.popup = None;
    for name in ["Doshaguma", "Rathalos", "Mizutsune"] {
        let p = Pick::new(index(name), false);
        remember(&mut app.data.recent, p);
        app.data.monster = Some(p);
    }
    app.data.alt = Alt::Monster;
    app.data.apply_variant();
    shot(&mut app, &mut h, "monster-picked.png", base, ppp)?;
    shot(&mut app, &mut h, "monster-picked-16x9.png", wide, 1.0)?;
    open(&mut app);
    shot(&mut app, &mut h, "monster-recent.png", base, ppp)?;
    shot(&mut app, &mut h, "monster-recent-16x9.png", wide, 1.0)?;

    // Tempered on, then the longest name in the breadcrumb.
    app.selector.tempered = true;
    let p = Pick::new(index("Rathalos"), true);
    app.data.monster = Some(p);
    shot(&mut app, &mut h, "monster-tempered.png", base, ppp)?;
    app.popup = None;
    let long = Pick::new(index("Guardian Fulgur Anjanath"), true);
    app.data.monster = Some(long);
    shot(&mut app, &mut h, "monster-case-long.png", base, ppp)?;
    shot(&mut app, &mut h, "monster-case-long-16x9.png", wide, 1.0)?;

    // The reserved weak-element column filled with invented values (review only).
    app.data.monster = Some(p);
    open(&mut app);
    app.selector.demo_weak = true;
    shot(&mut app, &mut h, "monster-case-weak.png", base, ppp)?;
    Ok(())
}
