// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 10 Shopping list without a window:
//! `shopping_shots [out dir] [pixels per point]`. Writes the three views at the base, 16:9 and
//! large sizes, a piece filter, the farm column filled (review only), a scrolled long list, the
//! empty list, and the `Missing N` hover on the Build screen (a short list and a long one).

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::shopping::sample;
use ui_sample::build_card::{BuildCard, ShopView};
use ui_sample::icons::SampleSource;
use ui_sample::shell::Section;

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

    // The Build screen with the default sample: `Missing 2` and `Missing 1` hovers.
    app.demo_missing_tip = Some(1);
    shot(&mut app, &mut h, "missing-tip.png", base, ppp)?;
    app.demo_missing_tip = None;

    // A full endgame list: every view, then the Build hover with the longest list.
    let chains = vec![(0, sample::CHAIN)];
    app.data.set_needs(sample::endgame(), chains.clone());
    app.demo_missing_tip = Some(1);
    shot(&mut app, &mut h, "missing-tip-long.png", base, ppp)?;
    app.demo_missing_tip = Some(0);
    shot(&mut app, &mut h, "missing-tip-weapon.png", base, ppp)?;
    app.demo_missing_tip = None;

    app.shell.section = Section::Shopping;
    for (view, slug) in [
        (ShopView::Monster, "monster"),
        (ShopView::Piece, "piece"),
        (ShopView::Path, "upgrade"),
    ] {
        app.shopping.view = view;
        shot(&mut app, &mut h, &format!("shop-{slug}.png"), base, ppp)?;
        shot(
            &mut app,
            &mut h,
            &format!("shop-{slug}-16x9.png"),
            wide,
            1.0,
        )?;
        shot(
            &mut app,
            &mut h,
            &format!("shop-{slug}-large.png"),
            large,
            1.0,
        )?;
    }

    // Scrolled into the middle of the long list.
    app.shopping.view = ShopView::Monster;
    app.shopping.demo_scroll = Some(180.0);
    shot(&mut app, &mut h, "shop-monster-scrolled.png", base, ppp)?;
    app.shopping.demo_scroll = None;

    // Filtered to one piece (what clicking `Missing N` does).
    app.shopping.show_piece(1);
    shot(&mut app, &mut h, "shop-filtered.png", base, ppp)?;
    shot(&mut app, &mut h, "shop-filtered-16x9.png", wide, 1.0)?;
    // The piece with the stress-test names.
    app.shopping.show_piece(4);
    shot(&mut app, &mut h, "shop-case-long.png", base, ppp)?;
    app.shopping.filter = None;

    // The farm column, filled with an invented place (review only: drop data is not confirmed).
    app.shopping.view = ShopView::Piece;
    app.shopping.demo_farm = true;
    shot(&mut app, &mut h, "shop-case-farm.png", base, ppp)?;
    app.shopping.demo_farm = false;

    // A mid-game list and the empty list.
    app.data.set_needs(sample::mid(), chains);
    app.shopping.view = ShopView::Monster;
    shot(&mut app, &mut h, "shop-mid.png", base, ppp)?;
    app.data.set_needs(Vec::new(), Vec::new());
    shot(&mut app, &mut h, "shop-empty.png", base, ppp)?;
    app.shopping.filter = Some(2);
    shot(&mut app, &mut h, "shop-empty-piece.png", base, ppp)?;
    Ok(())
}
