// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Opens the element 3 icon demo in a real window (the maintainer's PC):
//! `cargo run --release --features window --bin icons_window`.
//! Repaints only on input and when an icon arrives, so it idles at zero.

use std::sync::Arc;
use std::time::Duration;
use ui_sample::icons::{show, Demo, SampleSource};

struct App {
    demo: Demo,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.demo);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Forgeline icons",
        options,
        Box::new(|cc| {
            ui_sample::install_fonts(&cc.egui_ctx);
            let source = SampleSource {
                delay: Duration::from_millis(300),
            };
            Ok(Box::new(App {
                demo: Demo::new(&cc.egui_ctx, Arc::new(source)),
            }))
        }),
    )
}
