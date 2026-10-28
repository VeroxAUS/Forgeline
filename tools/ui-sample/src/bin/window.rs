// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Opens the sample in a real window: `cargo run --release --features window --bin window [a|b|c]`.
//! Press 1, 2 or 3 to switch style. Repaints only on input, so it idles at zero.

use ui_sample::{install_fonts, show, State, Variant, SCREEN};

struct App {
    state: State,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.input(|i| {
            for (key, v) in [
                (egui::Key::Num1, Variant::ModernDark),
                (egui::Key::Num2, Variant::Slate),
                (egui::Key::Num3, Variant::Ember),
            ] {
                if i.key_pressed(key) {
                    self.state.variant = v;
                }
            }
        });
        show(ui, &mut self.state);
    }
}

fn main() -> eframe::Result {
    let variant = match std::env::args().nth(1).as_deref() {
        Some("b") => Variant::Slate,
        Some("c") => Variant::Ember,
        _ => Variant::ModernDark,
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(SCREEN),
        ..Default::default()
    };
    eframe::run_native(
        "Forgeline style sample",
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(App {
                state: State::new(variant),
            }))
        }),
    )
}
