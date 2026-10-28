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
