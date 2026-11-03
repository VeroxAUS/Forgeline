// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 12 Why screen without a window: `why_shots [out dir] [pixels per point]`.
//! Writes the default build at the base, 16:9 and large sizes, a full endgame build (scrolled
//! too), a build with stress-test names, the unscored case, the Safe and Monster sentences with
//! many must-haves, edited weights, and the empty build.

use egui::Vec2;
use std::sync::Arc;
use std::time::Duration;
use ui_sample::build_card::{sample_endgame_skills, Alt, BuildCard, MustHave, Pick, Skill, Value};
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

    app.shell.section = Section::Why;
    shot(&mut app, &mut h, "why.png", base, ppp)?;
    shot(&mut app, &mut h, "why-16x9.png", wide, 1.0)?;
    shot(&mut app, &mut h, "why-large.png", large, 1.0)?;

    // A full endgame build: 22 skills, four set and group skills, scrolls.
    let default_skills = app.data.skills.clone();
    app.data.skills = sample_endgame_skills();
    shot(&mut app, &mut h, "why-endgame.png", base, ppp)?;
    app.demo_why_scroll = Some(300.0);
    shot(&mut app, &mut h, "why-endgame-scrolled.png", base, ppp)?;
    app.demo_why_scroll = None;
    shot(&mut app, &mut h, "why-endgame-16x9.png", wide, 1.0)?;

    // Stress test: long names, a long must-have list, a long monster name.
    app.data.skills = default_skills.clone();
    app.data.skills.push(
        Skill {
            must: true,
            weight: Some(4),
            ..Skill::normal("Elemental and status resistance compound", 2, 5)
        }
        .valued(Value::Comfort),
    );
    app.data.must_haves = vec![
        MustHave::new("Weakness exploit", 4, 5),
        MustHave::new("Ammo saver", 2, 3),
        MustHave::new("Elemental and status resistance compound", 2, 5),
        MustHave::new("Critical eye", 3, 5),
        MustHave::new("Sharpshot", 2, 3),
    ];
    shot(&mut app, &mut h, "why-case-long.png", base, ppp)?;

    // The other sentences.
    app.data.must_haves = vec![MustHave::new("Weakness exploit", 4, 5)];
    app.data.alt = Alt::Safe;
    shot(&mut app, &mut h, "why-safe.png", base, ppp)?;
    app.data.alt = Alt::Monster;
    app.data.monster = Some(Pick::new(0, true));
    shot(&mut app, &mut h, "why-monster.png", base, ppp)?;
    app.data.alt = Alt::Best;
    app.data.monster = None;
    app.data.must_haves.clear();
    shot(&mut app, &mut h, "why-no-musts.png", base, ppp)?;

    // Skills the optimizer has not scored yet.
    app.data.skills = default_skills
        .iter()
        .map(|k| Skill { value: None, ..*k })
        .collect();
    shot(&mut app, &mut h, "why-unscored.png", base, ppp)?;

    // Edited weights: the assumptions line says so.
    app.data.skills = default_skills;
    app.customise.weights[0].value = 9;
    app.customise.assume[0].value = 85;
    shot(&mut app, &mut h, "why-edited.png", base, ppp)?;

    // The empty build.
    app.data.skills.clear();
    shot(&mut app, &mut h, "why-empty.png", base, ppp)?;
    Ok(())
}
