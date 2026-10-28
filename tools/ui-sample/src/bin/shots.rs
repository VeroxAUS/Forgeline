// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Renders each style variant to a PNG without a window: `shots [out dir] [pixels per point]`.

use ui_sample::{render_frame, State, Variant};

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "shots".into()));
    let ppp: f32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(2.0);
    std::fs::create_dir_all(&dir)?;
    for v in Variant::ALL {
        let frame = render_frame(&mut State::new(v), ppp);
        let path = dir.join(format!("{}.png", v.slug()));
        frame.write_png(&path)?;
        println!("{}", path.display());
    }
    Ok(())
}
