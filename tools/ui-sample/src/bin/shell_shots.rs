// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders the element 2 shell without a window: `shell_shots [out dir] [pixels per point]`.

use egui::Vec2;
use ui_sample::shell::{render_frame, Section, Shell, Status, MIN_SIZE};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    let ppp: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(2.0);
    std::fs::create_dir_all(&dir)?;
    let jobs: [(&str, Section, Status, Vec2); 5] = [
        ("shell-build", Section::Build, Status::Synced, MIN_SIZE),
        (
            "shell-settings",
            Section::Settings,
            Status::Synced,
            MIN_SIZE,
        ),
        (
            "shell-recomputing",
            Section::Shopping,
            Status::Recomputing,
            MIN_SIZE,
        ),
        (
            "shell-problem",
            Section::Data,
            Status::Problem("Couldn't read the save (file in use). Showing the last build.".into()),
            MIN_SIZE,
        ),
        (
            "shell-large",
            Section::Why,
            Status::Synced,
            Vec2::new(1440.0, 900.0),
        ),
    ];
    for (name, section, status, size) in jobs {
        let mut shell = Shell { section, status };
        let path = dir.join(format!("{name}.png"));
        render_frame(&mut shell, size, ppp).write_png(&path)?;
        println!("{}", path.display());
    }
    Ok(())
}
