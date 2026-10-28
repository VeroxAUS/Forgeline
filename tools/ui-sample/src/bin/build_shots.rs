// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 4 and 5 build card and skills strip without a window: `build_shots [out dir] [pixels per point]`.
//! Writes the first frame (icons still loading), the ready frame, the recomputing state and a
//! larger window.

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{BuildCard, Headless};
use ui_sample::icons::SampleSource;
use ui_sample::shell::Status;

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
    let mut app = BuildCard::new(h.ctx(), Arc::new(source));

    let save = |name: &str, f: ui_sample::render::Frame| -> std::io::Result<()> {
        let path = dir.join(name);
        f.write_png(&path)?;
        println!("{}", path.display());
        Ok(())
    };
    h.frame(&mut app, size, ppp);
    save("build-loading.png", h.frame(&mut app, size, ppp))?;
    assert!(app.cache.wait_idle(Duration::from_secs(5)), "icons arrived");
    save("build-ready.png", h.frame(&mut app, size, ppp))?;
    app.shell.status = Status::Recomputing;
    save("build-recomputing.png", h.frame(&mut app, size, ppp))?;
    app.shell.status = Status::Synced;
    // Element 5: the same strip with no skills, for the empty state.
    let skills = std::mem::take(&mut app.data.skills);
    save("skills-empty.png", h.frame(&mut app, size, ppp))?;
    // Endgame: a long list, at the base size and a larger one, with both set and group looks.
    use ui_sample::build_card::{sample_endgame_skills, SetStyle, Skill};
    app.data.skills = sample_endgame_skills();
    save("skills-endgame.png", h.frame(&mut app, size, ppp))?;
    save(
        "skills-endgame-large.png",
        h.frame(&mut app, Vec2::new(1400.0, 900.0), ppp),
    )?;
    app.set_style = SetStyle::Pips;
    save("skills-endgame-pips.png", h.frame(&mut app, size, ppp))?;
    app.set_style = SetStyle::Text;
    // More than fits: four low-weight extras, so the cut and the "+N more on Why" line show.
    for (name, level, max) in [
        ("Wide range", 1, 5),
        ("Free meal", 1, 3),
        ("Botanist", 1, 4),
        ("Speed eating", 2, 3),
    ] {
        app.data.skills.push(Skill {
            weight: Some(3),
            ..Skill::normal(name, level, max)
        });
    }
    save("skills-26.png", h.frame(&mut app, size, ppp))?;
    app.data.skills = sample_endgame_skills();
    app.data.skills.truncate(4 + 10);
    save("skills-14.png", h.frame(&mut app, size, ppp))?;
    // The short default list in both set and group looks.
    app.data.skills = skills.clone();
    app.set_style = SetStyle::Pips;
    save("skills-8-pips.png", h.frame(&mut app, size, ppp))?;
    app.set_style = SetStyle::Text;
    app.data.skills = skills;
    save(
        "build-large.png",
        h.frame(&mut app, Vec2::new(1400.0, 900.0), ppp),
    )?;
    save(
        "build-xl.png",
        h.frame(&mut app, Vec2::new(2000.0, 1300.0), 1.0),
    )?;
    Ok(())
}
