// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Opens the element 2 shell in a real window (the maintainer's PC):
//! `cargo run --release --features window --bin shell_window`.
//! Ctrl+1 to Ctrl+6 switch section. Repaints only on input, so it idles at zero.

use ui_sample::shell::{show, Shell, MIN_SIZE};

struct App {
    shell: Shell,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        show(ui, &mut self.shell);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(MIN_SIZE)
            .with_min_inner_size(MIN_SIZE),
        ..Default::default()
    };
    eframe::run_native(
        "Forgeline shell",
        options,
        Box::new(|cc| {
            ui_sample::install_fonts(&cc.egui_ctx);
            Ok(Box::new(App {
                shell: Shell::new(),
            }))
        }),
    )
}
