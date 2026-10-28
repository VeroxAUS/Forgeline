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
}
