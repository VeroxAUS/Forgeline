// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 11 of docs/ux/design-plan.md: the Customise drawer. A panel on the right of the Build
//! screen with saved presets, the two assumption sliders (uptime, weak-spot rate) and the comfort
//! and defence weight sliders. Every slider shows its baseline (weights) or default (assumptions)
//! as a tick under the bar and has a reset.
//!
//! Everything here is sample data. Comfort baselines do not exist until the Phase 4 research round
//! and the assumption defaults are invented, so the drawer says so at the top. Skill names are
//! illustrative; the numbers are not from the game or from research.

use super::*;

/// Drawer width, and its side padding. At the 1000 point design width the `+ Add` link and the
/// Customise button stay clear of it.
const W: f32 = 340.0;
const PAD: f32 = 20.0;
const HEAD_H: f32 = 56.0;
/// Every slider row (name line, bar, baseline tick) is this high, on one grid.
const SLIDER_H: f32 = 48.0;
const PRESET_H: f32 = 28.0;
const FIELD_H: f32 = 28.0;
const SECTION_H: f32 = 30.0;
const RESET_W: f32 = 48.0;
const VALUE_W: f32 = 40.0;
const HANDLE_R: f32 = 5.0;
const BAR_H: f32 = 6.0;
const SAVE_W: f32 = 72.0;
const MAX_PRESETS: usize = 8;

/// One slider: a weight (0 to 10) or an assumption (0 to 100 percent, steps of 5). `base` is the
/// baseline of a weight or the default of an assumption; the tick under the bar and Reset use it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slider {
    pub name: &'static str,
    pub tip: &'static str,
    pub value: u8,
    pub base: u8,
    pub max: u8,
    pub step: u8,
    pub unit: &'static str,
}

impl Slider {
    fn weight(name: &'static str, base: u8) -> Self {
        Slider {
            name,
            tip: "How much the optimizer values this skill.",
            value: base,
            base,
            max: 10,
            step: 1,
            unit: "",
        }
    }

    fn assume(name: &'static str, tip: &'static str, base: u8) -> Self {
        Slider {
            name,
            tip,
            value: base,
            base,
            max: 100,
            step: 5,
            unit: "%",
        }
    }

    pub fn changed(&self) -> bool {
        self.value != self.base
    }
}

/// The value under pointer x on a bar from `left` to `right`, snapped to the slider's step.
pub fn value_at(x: f32, left: f32, right: f32, max: u8, step: u8) -> u8 {
    let t = ((x - left) / (right - left)).clamp(0.0, 1.0);
    let steps = f32::from(max / step);
    (t * steps).round() as u8 * step
}

/// A saved preset: every slider value in drawer order (comfort, defence, assumptions).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preset {
    pub name: String,
    pub values: Vec<u8>,
}

/// The drawer's state.
pub struct Customise {
    pub open: bool,
    /// The preset waiting for a second click to be deleted.
    pub confirm: Option<usize>,
    confirm_btn: Rect,
    pub comfort: Vec<Slider>,
    pub defence: Vec<Slider>,
    pub assume: Vec<Slider>,
    pub presets: Vec<Preset>,
    /// The name typed in the save field.
    pub name: String,
    /// Review only: scrolls the drawer to this offset.
    pub demo_scroll: Option<f32>,
}

impl Customise {
    /// Sample sliders and two sample presets. Skill names are illustrative, all numbers invented.
    pub fn sample() -> Self {
        let mut c = Customise {
            open: false,
            confirm: None,
            confirm_btn: Rect::NOTHING,
            comfort: [
                ("Evade window", 6),
                ("Stamina surge", 5),
                ("Focus", 4),
                ("Wide-range", 4),
                ("Item prolonger", 3),
                ("Quick sheathe", 3),
                ("Speed eating", 2),
            ]
            .map(|(n, b)| Slider::weight(n, b))
            .to_vec(),
            defence: [
                ("Health boost", 5),
                ("Defense boost", 4),
                ("Recovery up", 3),
                ("Divine blessing", 2),
            ]
            .map(|(n, b)| Slider::weight(n, b))
            .to_vec(),
            assume: vec![
                Slider::assume(
                    "Uptime",
                    "How much of a fight a skill that must be triggered is active.",
                    60,
                ),
                Slider::assume(
                    "Weak-spot rate",
                    "How many of your hits land on a weak spot.",
                    50,
                ),
            ],
            presets: Vec::new(),
            name: String::new(),
            demo_scroll: None,
        };
        for (name, edits) in [
            ("Solo comfort", [("Evade window", 9), ("Focus", 7)]),
            ("Safe run", [("Health boost", 9), ("Recovery up", 8)]),
        ] {
            for (n, v) in edits {
                if let Some(s) = c.all_mut().find(|s| s.name == n) {
                    s.value = v;
                }
            }
            c.presets.push(Preset {
                name: name.into(),
                values: c.values(),
            });
            c.reset_all();
        }
        c
    }

    /// The worst case for names: a long preset name and long skill names.
    pub fn sample_long() -> Self {
        let mut c = Self::sample();
        c.comfort[0].name = "Convert Thunder Resistance";
        c.comfort[1].name = "Poison Functionality";
        c.defence[0].name = "Convert Water Resistance and more";
        c.presets[0].name = "Multiplayer support build for the long Arch-Tempered hunts".into();
        c
    }

    fn all(&self) -> impl Iterator<Item = &Slider> {
        self.comfort.iter().chain(&self.defence).chain(&self.assume)
    }

    fn all_mut(&mut self) -> impl Iterator<Item = &mut Slider> {
        self.comfort
            .iter_mut()
            .chain(&mut self.defence)
            .chain(&mut self.assume)
    }

    pub fn values(&self) -> Vec<u8> {
        self.all().map(|s| s.value).collect()
    }

    pub fn is_default(&self) -> bool {
        self.all().all(|s| !s.changed())
    }

    pub fn reset_all(&mut self) {
        self.all_mut().for_each(|s| s.value = s.base);
    }

    /// Why Save does nothing right now, or `None` when it works. A name that exists is replaced.
    pub fn save_blocked(&self) -> Option<&'static str> {
        if self.name.trim().is_empty() {
            Some("Type a name first.")
        } else if self.presets.len() >= MAX_PRESETS && self.replaces().is_none() {
            Some("8 presets are saved. Delete one to save another.")
        } else {
            None
        }
    }

    fn replaces(&self) -> Option<usize> {
        let n = self.name.trim();
        self.presets.iter().position(|p| p.name == n)
    }

    /// Saves the current values under the typed name (replacing a preset of that name) and clears
    /// the field. Returns whether it saved.
    pub fn save(&mut self) -> bool {
        if self.save_blocked().is_some() {
            return false;
        }
        let preset = Preset {
            name: self.name.trim().to_string(),
            values: self.values(),
        };
        match self.replaces() {
            Some(i) => self.presets[i] = preset,
            None => self.presets.push(preset),
        }
        self.name.clear();
        true
    }

    pub fn load(&mut self, i: usize) {
        let Some(p) = self.presets.get(i) else { return };
        let values = p.values.clone();
        for (s, v) in self.all_mut().zip(values) {
            s.value = v;
        }
    }

    /// First click on a preset's cross: asks for a second click before anything is lost.
    pub fn ask_delete(&mut self, i: usize) {
        self.confirm = (i < self.presets.len()).then_some(i);
    }

    /// The second click: deletes the preset that was asked about.
    pub fn confirm_delete(&mut self) {
        if let Some(i) = self.confirm {
            self.delete(i);
        }
    }

    pub fn delete(&mut self, i: usize) {
        self.confirm = None;
        if i < self.presets.len() {
            self.presets.remove(i);
        }
    }

    /// The preset the sliders currently match, if any.
    pub fn loaded(&self) -> Option<usize> {
        let now = self.values();
        self.presets.iter().position(|p| p.values == now)
    }
}

enum Act {
    Load(usize),
    Ask(usize),
    Delete,
    Save,
}

/// The sliders icon: three bars with a knob each, drawn in `r`.
pub fn sliders_glyph(p: &Painter, r: Rect, col: Color32) {
    let u = r.width() / 16.0;
    for (y, kx) in [(-4.5, -3.0), (0.0, 3.5), (4.5, -1.0)] {
        let cy = r.center().y + y * u;
        p.line_segment(
            [Pos2::new(r.left() + u, cy), Pos2::new(r.right() - u, cy)],
            Stroke::new(1.4, col),
        );
        p.circle_filled(Pos2::new(r.center().x + kx * u, cy), 2.2 * u, col);
    }
}

/// Draws the drawer over the right of `content`. Returns true when a slider changed or a preset
/// was loaded, so the caller starts a non-blocking recompute.
pub fn show(ui: &mut Ui, content: Rect, st: &mut Customise) -> bool {
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let panel = Rect::from_min_max(
        Pos2::new(snap(content.max.x - W), content.min.y),
        content.max,
    );
    let mut changed = false;
    let mut act: Option<Act> = None;
    let mut close = false;
    // A press anywhere but the Delete button of the row being asked about cancels the question.
    if st.confirm.is_some()
        && ctx.input(|i| i.pointer.any_pressed())
        && ctx
            .pointer_interact_pos()
            .is_none_or(|p| !st.confirm_btn.contains(p))
    {
        st.confirm = None;
    }
    let confirm = st.confirm;
    let mut confirm_btn = Rect::NOTHING;
    egui::Area::new(ui.scope_id().with("customise-drawer"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(panel.min)
        .show(&ctx, |ui| {
            // The first widget: it catches clicks so nothing behind the drawer reacts.
            let (rect, _) = ui.allocate_exact_size(panel.size(), egui::Sense::click_and_drag());
            let p = ui.painter().clone();
            p.rect_filled(rect, 0.0, CHIP_FILL);
            p.line_segment(
                [rect.left_top(), rect.left_bottom()],
                Stroke::new(1.0, LINE),
            );
            let left = rect.left() + PAD;
            let right = rect.right() - PAD;
            let hover = ctx.pointer_hover_pos();
            let mut body =
                ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(PAD, 0.0))));

            // Header: title, Reset all, close.
            let cy = snap(rect.top() + 30.0);
            p.text(
                Pos2::new(left, cy),
                Align2::LEFT_CENTER,
                "Customise",
                font("inter-medium", 16.0),
                FG,
            );
            let x_rect = Rect::from_center_size(Pos2::new(right - 12.0, cy), Vec2::splat(28.0));
            let x_resp = body.interact(x_rect, body.scope_id().with("close"), egui::Sense::click());
            let x_col = if x_resp.hovered() { FG } else { DIM };
            let c = x_rect.center();
            for (a, b) in [((-4.5, -4.5), (4.5, 4.5)), ((-4.5, 4.5), (4.5, -4.5))] {
                p.line_segment(
                    [c + Vec2::new(a.0, a.1), c + Vec2::new(b.0, b.1)],
                    Stroke::new(1.4, x_col),
                );
            }
            if x_resp.clicked() {
                close = true;
            }
            x_resp
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Close (Escape)");
            let dirty = !st.is_default();
            let reset_rect = Rect::from_center_size(
                Pos2::new(x_rect.left() - 8.0 - 36.0, cy),
                Vec2::new(72.0, 24.0),
            );
            let reset_resp = body.interact(
                reset_rect,
                body.scope_id().with("reset-all"),
                egui::Sense::click(),
            );
            let hot = dirty && reset_resp.hovered();
            p.text(
                Pos2::new(reset_rect.right(), cy),
                Align2::RIGHT_CENTER,
                "RESET ALL",
                mono(10.5),
                if hot {
                    ACCENT
                } else if dirty {
                    DIM
                } else {
                    FAINT
                },
            );
            if dirty {
                if reset_resp.clicked() {
                    st.reset_all();
                    changed = true;
                }
                reset_resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Put every bar back on its baseline or default");
            }
            hair(&p, rect.left(), rect.right(), rect.top() + HEAD_H);

            // The sample label, always on: nothing here is researched yet.
            let note_y = rect.top() + HEAD_H + 12.0;
            p.text(
                Pos2::new(left, note_y + 6.0),
                Align2::LEFT_CENTER,
                "SAMPLE DATA, NOT RESEARCHED",
                mono(10.5),
                WARN,
            );
            let galley = p.layout(
                "Baselines and defaults are placeholders until the Phase 4 research round. \
                 The tick under a bar marks its baseline; Reset puts it back."
                    .to_string(),
                font("inter", 12.0),
                DIM,
                right - left,
            );
            p.galley(Pos2::new(left, note_y + 18.0), galley.clone(), DIM);
            let top = note_y + 18.0 + galley.size().y + 8.0;

            let area = Rect::from_min_max(
                Pos2::new(left, snap(top)),
                Pos2::new(right, rect.bottom() - 12.0),
            );
            let mut scroll_ui = body.new_child(egui::UiBuilder::new().max_rect(area));
            scroll_ui.spacing_mut().item_spacing = Vec2::ZERO;
            let mut scroll = egui::ScrollArea::vertical()
                .id_salt("customise")
                .auto_shrink([false, false])
                .max_height(area.height());
            if let Some(y) = st.demo_scroll {
                scroll = scroll.vertical_scroll_offset(y);
            }
            scroll.show(&mut scroll_ui, |ui| {
                let w = area.width();
                let label = |ui: &mut Ui, text: &str, tail: &str| {
                    let (r, _) =
                        ui.allocate_exact_size(Vec2::new(w, SECTION_H), egui::Sense::hover());
                    let p = ui.painter();
                    section(p, r.left(), r.right(), r.top(), text);
                    p.text(
                        Pos2::new(r.right(), r.top() + 8.0),
                        Align2::RIGHT_CENTER,
                        tail,
                        mono(10.5),
                        FAINT,
                    );
                };

                label(
                    ui,
                    "PRESETS",
                    &format!("{} OF {MAX_PRESETS}", st.presets.len()),
                );
                let loaded = st.loaded();
                for (i, pr) in st.presets.iter().enumerate() {
                    let asked = confirm == Some(i);
                    if let Some(a) = preset_row(
                        ui,
                        w,
                        (i, pr, loaded == Some(i), asked),
                        &mut confirm_btn,
                        hover,
                    ) {
                        act = Some(a);
                    }
                }
                if st.presets.is_empty() {
                    let (r, _) =
                        ui.allocate_exact_size(Vec2::new(w, PRESET_H), egui::Sense::hover());
                    ui.painter().text(
                        Pos2::new(r.left(), r.center().y),
                        Align2::LEFT_CENTER,
                        "No saved presets yet.",
                        font("inter", 12.0),
                        FAINT,
                    );
                }
                ui.add_space(8.0);
                if let Some(a) = save_row(ui, w, st) {
                    act = Some(a);
                }
                ui.add_space(14.0);

                for (title, tail, which) in [
                    ("ASSUMPTIONS", "SAMPLE DEFAULTS", 2),
                    ("COMFORT WEIGHTS", "SAMPLE BASELINES", 0),
                    ("DEFENCE WEIGHTS", "SAMPLE BASELINES", 1),
                ] {
                    label(ui, title, tail);
                    let list = match which {
                        0 => &mut st.comfort,
                        1 => &mut st.defence,
                        _ => &mut st.assume,
                    };
                    let n = list.len();
                    for (i, s) in list.iter_mut().enumerate() {
                        changed |= slider_row(ui, w, s, i + 1 < n);
                    }
                    ui.add_space(14.0);
                }
            });
        });
    match act {
        Some(Act::Load(i)) => {
            st.load(i);
            changed = true;
        }
        Some(Act::Ask(i)) => st.ask_delete(i),
        Some(Act::Delete) => st.confirm_delete(),
        Some(Act::Save) => {
            st.save();
        }
        None => {}
    }
    st.confirm_btn = confirm_btn;
    if close {
        st.open = false;
    }
    changed
}

/// One slider row: name, value and Reset on the first line, the bar with its baseline tick under.
fn slider_row(ui: &mut Ui, w: f32, s: &mut Slider, divider: bool) -> bool {
    let ppp = ui.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, SLIDER_H), egui::Sense::hover());
    let r = Rect::from_min_size(Pos2::new(snap(r.left()), snap(r.top())), r.size());
    let id = ui.scope_id().with(("customise-slider", s.name));
    let mut changed = false;

    // The bar first, so the new value is what gets drawn.
    let bar = Rect::from_min_max(
        Pos2::new(r.left(), r.top() + 24.0),
        Pos2::new(r.right(), r.top() + 42.0),
    );
    let (tl, tr) = (bar.left() + HANDLE_R, bar.right() - HANDLE_R);
    let resp = ui.interact(bar, id.with("bar"), egui::Sense::click_and_drag());
    if resp.dragged() || resp.clicked() {
        if let Some(pos) = resp.interact_pointer_pos() {
            let v = value_at(pos.x, tl, tr, s.max, s.step);
            if v != s.value {
                s.value = v;
                changed = true;
            }
        }
    }
    let tip = format!(
        "{}. {} Baseline {}{} (sample, not researched), now {}{}.",
        s.name, s.tip, s.base, s.unit, s.value, s.unit
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip.clone());

    let reset = Rect::from_min_size(
        Pos2::new(r.right() - RESET_W, r.top()),
        Vec2::new(RESET_W, 24.0),
    );
    let reset_resp = ui.interact(reset, id.with("reset"), egui::Sense::click());
    let name_rect = Rect::from_min_max(
        r.left_top(),
        Pos2::new(reset.left() - VALUE_W - 8.0, r.top() + 24.0),
    );
    ui.interact(name_rect, id.with("name"), egui::Sense::hover())
        .on_hover_text(tip);
    let was_changed = s.changed();
    if was_changed {
        if reset_resp.clicked() {
            s.value = s.base;
            changed = true;
        }
        reset_resp
            .clone()
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(format!("Back to {}{}", s.base, s.unit));
    }

    let p = ui.painter();
    let cy = snap(r.top() + 12.0);
    p.text(
        Pos2::new(r.left(), cy),
        Align2::LEFT_CENTER,
        fit(p, s.name, name_rect.width(), "inter", 12.5),
        font("inter", 12.5),
        FG,
    );
    p.text(
        Pos2::new(reset.left() - 8.0, cy),
        Align2::RIGHT_CENTER,
        format!("{}{}", s.value, s.unit),
        mono(11.5),
        if s.changed() { FG } else { DIM },
    );
    p.text(
        Pos2::new(r.right(), cy),
        Align2::RIGHT_CENTER,
        "RESET",
        mono(10.5),
        if !was_changed {
            FAINT
        } else if reset_resp.hovered() {
            ACCENT
        } else {
            DIM
        },
    );

    let bar_y = snap(bar.top() + 7.0);
    let x_at = |v: u8| tl + (tr - tl) * f32::from(v) / f32::from(s.max);
    let track = Rect::from_min_max(
        Pos2::new(tl, bar_y - BAR_H / 2.0),
        Pos2::new(tr, bar_y + BAR_H / 2.0),
    );
    p.rect_filled(track, BAR_H / 2.0, LINE);
    let x = snap(x_at(s.value));
    if x > tl {
        p.rect_filled(
            Rect::from_min_max(track.min, Pos2::new(x, track.max.y)),
            BAR_H / 2.0,
            ACCENT,
        );
    }
    let bx = snap(x_at(s.base));
    p.line_segment(
        [Pos2::new(bx, bar_y + 6.0), Pos2::new(bx, bar_y + 11.0)],
        Stroke::new(1.5, DIM),
    );
    p.circle_filled(Pos2::new(x, bar_y), HANDLE_R, FG);
    if divider {
        hair(p, r.left(), r.right(), r.bottom() - 0.5);
    }
    changed
}

/// A saved preset: click to load, the cross deletes. The loaded one is marked like the current
/// monster (accent bar and name).
fn preset_row(
    ui: &mut Ui,
    w: f32,
    (i, pr, loaded, asked): (usize, &Preset, bool, bool),
    confirm_btn: &mut Rect,
    hover: Option<Pos2>,
) -> Option<Act> {
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, PRESET_H), egui::Sense::hover());
    let id = ui.scope_id().with(("customise-preset", i));
    // Asked: the cross becomes a `Delete?` button; the rest of the row only reads.
    let del_w = if asked { 64.0 } else { 24.0 };
    let del = Rect::from_min_size(
        Pos2::new(r.right() - del_w, r.top()),
        Vec2::new(del_w, PRESET_H),
    );
    let main = Rect::from_min_max(r.min, Pos2::new(del.left(), r.max.y));
    let main_sense = if asked {
        egui::Sense::hover()
    } else {
        egui::Sense::click()
    };
    let main_resp = ui.interact(main, id.with("load"), main_sense);
    let del_resp = ui.interact(del, id.with("del"), egui::Sense::click());
    let lit = hover.is_some_and(|h| r.contains(h));
    let p = ui.painter();
    if lit {
        p.rect_filled(r, 2.0, LINE);
    }
    if loaded {
        p.rect_filled(
            Rect::from_min_size(r.left_top(), Vec2::new(2.0, r.height())),
            0.0,
            ACCENT,
        );
    }
    p.text(
        Pos2::new(r.left() + 10.0, r.center().y),
        Align2::LEFT_CENTER,
        fit(p, &pr.name, main.width() - 18.0, "inter", 12.5),
        font("inter", 12.5),
        if loaded { ACCENT } else { FG },
    );
    if asked {
        *confirm_btn = del;
        let b = del.shrink2(Vec2::new(0.0, 3.0));
        let hot = del_resp.hovered();
        p.rect_stroke(
            b,
            2.0,
            Stroke::new(1.0, if hot { FG } else { WARN }),
            egui::StrokeKind::Inside,
        );
        p.text(
            b.center(),
            Align2::CENTER_CENTER,
            "Delete?",
            font("inter-medium", 12.0),
            if hot { FG } else { WARN },
        );
    } else {
        let c = del.center();
        let col = if del_resp.hovered() { FG } else { FAINT };
        for (a, b) in [((-3.5, -3.5), (3.5, 3.5)), ((-3.5, 3.5), (3.5, -3.5))] {
            p.line_segment(
                [c + Vec2::new(a.0, a.1), c + Vec2::new(b.0, b.1)],
                Stroke::new(1.2, col),
            );
        }
    }
    let act = if del_resp.clicked() {
        Some(if asked { Act::Delete } else { Act::Ask(i) })
    } else if main_resp.clicked() {
        Some(Act::Load(i))
    } else {
        None
    };
    if asked {
        main_resp.on_hover_text(pr.name.clone());
    } else {
        main_resp
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(format!("{}. Click to load.", pr.name));
    }
    del_resp
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(if asked {
            "Click to delete for good. Click anywhere else or press Escape to keep it."
        } else {
            "Delete this preset"
        });
    act
}

/// The name field with its hairline, and the Save (or Replace) button.
fn save_row(ui: &mut Ui, w: f32, st: &mut Customise) -> Option<Act> {
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, FIELD_H), egui::Sense::hover());
    let field = Rect::from_min_size(r.min, Vec2::new(w - SAVE_W - 12.0, FIELD_H));
    let mut edit_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(field.shrink2(Vec2::new(2.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    let edit = egui::TextEdit::singleline(&mut st.name)
        .frame(egui::Frame::NONE)
        .font(font("inter", 12.5))
        .text_color(FG)
        .hint_text(
            egui::RichText::new("Name this preset")
                .font(font("inter", 12.5))
                .color(FAINT),
        )
        .char_limit(60)
        .desired_width(field.width() - 4.0);
    let resp = edit_ui.add(edit);
    hair(ui.painter(), field.left(), field.right(), field.bottom());
    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

    let btn = Rect::from_center_size(
        Pos2::new(r.right() - SAVE_W / 2.0, r.center().y),
        Vec2::new(SAVE_W, 26.0),
    );
    let b = ui.interact(
        btn,
        ui.scope_id().with("customise-save"),
        egui::Sense::click(),
    );
    let blocked = st.save_blocked();
    let hot = blocked.is_none() && b.hovered();
    let p = ui.painter();
    p.rect_stroke(
        btn,
        2.0,
        Stroke::new(1.0, if hot { ACCENT } else { LINE }),
        egui::StrokeKind::Inside,
    );
    p.text(
        btn.center(),
        Align2::CENTER_CENTER,
        if st.replaces().is_some() {
            "Replace"
        } else {
            "Save"
        },
        font("inter-medium", 12.5),
        if blocked.is_some() {
            FAINT
        } else if hot {
            ACCENT
        } else {
            FG
        },
    );
    let go = blocked.is_none() && (b.clicked() || enter);
    match blocked {
        Some(why) => {
            b.on_hover_cursor(egui::CursorIcon::NotAllowed)
                .on_hover_text(why);
        }
        None => {
            b.on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Save the current bars as a preset");
        }
    }
    go.then_some(Act::Save)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_follows_the_pointer_and_snaps_to_the_step() {
        assert_eq!(value_at(100.0, 100.0, 300.0, 10, 1), 0);
        assert_eq!(value_at(300.0, 100.0, 300.0, 10, 1), 10);
        assert_eq!(value_at(1000.0, 100.0, 300.0, 10, 1), 10, "clamped");
        assert_eq!(value_at(-50.0, 100.0, 300.0, 10, 1), 0, "clamped");
        assert_eq!(value_at(200.0, 100.0, 300.0, 10, 1), 5);
        // Assumptions move in steps of 5 percent.
        assert_eq!(value_at(163.0, 100.0, 300.0, 100, 5), 30);
    }

    #[test]
    fn reset_and_presets_round_trip() {
        let mut c = Customise::sample();
        assert!(c.is_default());
        assert_eq!(c.presets.len(), 2);
        assert_eq!(c.loaded(), None, "defaults match no sample preset");
        c.comfort[0].value = 10;
        c.assume[0].value = 85;
        assert!(!c.is_default());
        // Save needs a name; a new name adds, the same name replaces.
        assert!(c.save_blocked().is_some());
        c.name = "  Mine ".into();
        assert!(c.save());
        assert!(c.name.is_empty());
        assert_eq!(c.presets.len(), 3);
        assert_eq!(c.loaded(), Some(2));
        c.reset_all();
        assert!(c.is_default());
        c.load(2);
        assert_eq!((c.comfort[0].value, c.assume[0].value), (10, 85));
        c.name = "Mine".into();
        c.comfort[0].value = 1;
        assert!(c.save());
        assert_eq!(c.presets.len(), 3, "replaced, not added");
        c.delete(2);
        c.delete(99);
        assert_eq!(c.presets.len(), 2);
    }

    #[test]
    fn deleting_a_preset_needs_a_second_click() {
        let mut c = Customise::sample();
        c.ask_delete(0);
        assert_eq!(
            (c.confirm, c.presets.len()),
            (Some(0), 2),
            "asked, not deleted"
        );
        c.confirm = None; // an outside click or Escape
        c.confirm_delete();
        assert_eq!(c.presets.len(), 2, "nothing asked, nothing deleted");
        c.ask_delete(1);
        c.confirm_delete();
        assert_eq!((c.confirm, c.presets.len()), (None, 1));
        assert_eq!(c.presets[0].name, "Solo comfort");
        c.ask_delete(5);
        assert_eq!(c.confirm, None, "no such preset");
    }

    #[test]
    fn the_preset_list_is_capped_but_a_replace_still_works() {
        let mut c = Customise::sample();
        for i in 0..MAX_PRESETS {
            c.name = format!("p{i}");
            c.save();
        }
        assert_eq!(c.presets.len(), MAX_PRESETS);
        c.name = "one more".into();
        assert!(c.save_blocked().is_some());
        assert!(!c.save());
        c.name = "p0".into();
        assert!(c.save_blocked().is_none());
    }
}
