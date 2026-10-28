// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 11 of docs/ux/design-plan.md: the Customise drawer. A panel that opens over the right
//! side of the Build screen, under the headline, so the damage and toughness words stay in view
//! while a slider moves. It holds saved presets, the two assumptions the optimizer states
//! (time attacking and weak-spot hits) and a weight per comfort or defence skill.
//!
//! Every slider shows its baseline as a tick and a reset mark appears only on a changed row (its
//! space is always kept, so the grid never shifts). A recompute starts when a slider is let go,
//! never while it moves, and never blocks the screen. Damage is calculated, so it has no slider.
//!
//! Sample data only. The researched baseline for every skill is the Phase 4 research round; the
//! values here are invented and the drawer says so. Presets live in memory in this sample; the
//! real app writes them to its config file.

use super::*;

pub const DRAWER_W: f32 = 320.0;
/// The drawer starts under the headline block (damage and toughness stay visible).
pub const TOP: f32 = 114.0;
const PAD: f32 = 16.0;
const INNER_W: f32 = DRAWER_W - 2.0 * PAD;
const HEAD_H: f32 = 28.0;
const FOOT_H: f32 = 40.0;
const LABEL_H: f32 = 30.0;
/// Weight rows: name, slider, value and a reset mark, all on one 34 point grid.
const ROW_H: f32 = 34.0;
const NAME_W: f32 = 112.0;
const TRACK_W: f32 = 116.0;
const VALUE_W: f32 = 22.0;
const RESET_W: f32 = 18.0;
const GAP: f32 = 8.0;
const TRACK_H: f32 = 24.0;
const KNOB_R: f32 = 6.0;
/// Assumption rows: a name line over a full width slider.
const ASSUME_H: f32 = 56.0;
/// Presets: two equal columns of identical chips.
const CHIP_W: f32 = (INNER_W - GAP) / 2.0;
const CHIP_H: f32 = 24.0;
const MAX_USER_PRESETS: usize = 5;
const NAME_LIMIT: usize = 28;

pub struct Weight {
    pub name: &'static str,
    pub baseline: u8,
    pub value: u8,
}

pub struct Assumption {
    pub name: &'static str,
    pub tip: &'static str,
    pub min: u8,
    pub max: u8,
    pub step: u8,
    pub baseline: u8,
    pub value: u8,
    /// The range the optimizer also checks the best build across (shaded on the track).
    pub band: (u8, u8),
}

/// A saved set of weights and assumptions.
#[derive(Clone)]
pub struct Preset {
    pub name: String,
    pub weights: Vec<u8>,
    pub assume: Vec<u8>,
}

/// The drawer's state. `active` is 0 for the baseline and `n` for user preset `n - 1`.
pub struct State {
    pub open: bool,
    pub weights: Vec<Weight>,
    pub assume: Vec<Assumption>,
    pub presets: Vec<Preset>,
    pub active: usize,
    /// The name being typed while saving a preset.
    pub naming: Option<String>,
    naming_fresh: bool,
    /// Review only: scrolls the body to this offset.
    pub demo_scroll: Option<f32>,
}

const SAMPLE_WEIGHTS: &[(&str, u8)] = &[
    ("Evade window", 6),
    ("Evade extender", 4),
    ("Stamina surge", 5),
    ("Recovery speed", 4),
    ("Divine blessing", 5),
    ("Free meal", 3),
    ("Speed eating", 2),
    ("Item prolonger", 3),
    ("Constitution", 4),
    ("Flinch free", 6),
    ("Wind resist", 3),
    ("Stun resistance", 5),
    ("Elemental and status resistance compound", 2),
];

impl State {
    pub fn new() -> Self {
        let weights = SAMPLE_WEIGHTS
            .iter()
            .map(|(name, b)| Weight {
                name,
                baseline: *b,
                value: *b,
            })
            .collect();
        let assume = vec![
            Assumption {
                name: "Time attacking",
                tip: "How much of a hunt you spend hitting, not moving or healing.",
                min: 30,
                max: 100,
                step: 5,
                baseline: 70,
                value: 70,
                band: (55, 85),
            },
            Assumption {
                name: "Weak-spot hits",
                tip: "The share of your hits that land on a weak spot.",
                min: 20,
                max: 100,
                step: 5,
                baseline: 55,
                value: 55,
                band: (40, 70),
            },
        ];
        let mut st = State {
            open: false,
            weights,
            assume,
            presets: Vec::new(),
            active: 0,
            naming: None,
            naming_fresh: false,
            demo_scroll: None,
        };
        let mut comfort = st.snapshot("Comfort first");
        for (i, w) in comfort.weights.iter_mut().enumerate() {
            if matches!(i, 0 | 1 | 2 | 9) {
                *w = (*w + 3).min(10);
            }
        }
        let mut boss = st.snapshot("Boss rush with a very long preset name");
        boss.assume = vec![85, 70];
        st.presets = vec![comfort, boss];
        st
    }

    fn snapshot(&self, name: &str) -> Preset {
        Preset {
            name: name.to_string(),
            weights: self.weights.iter().map(|w| w.value).collect(),
            assume: self.assume.iter().map(|a| a.value).collect(),
        }
    }

    fn load(&mut self, weights: &[u8], assume: &[u8]) {
        for (w, v) in self.weights.iter_mut().zip(weights) {
            w.value = *v;
        }
        for (a, v) in self.assume.iter_mut().zip(assume) {
            a.value = *v;
        }
    }

    fn baseline_values(&self) -> (Vec<u8>, Vec<u8>) {
        (
            self.weights.iter().map(|w| w.baseline).collect(),
            self.assume.iter().map(|a| a.baseline).collect(),
        )
    }

    fn differs_from(&self, weights: &[u8], assume: &[u8]) -> bool {
        self.weights
            .iter()
            .map(|w| w.value)
            .ne(weights.iter().copied())
            || self
                .assume
                .iter()
                .map(|a| a.value)
                .ne(assume.iter().copied())
    }

    /// Whether the values differ from the baseline.
    pub fn changed(&self) -> bool {
        let (w, a) = self.baseline_values();
        self.differs_from(&w, &a)
    }

    /// Whether the values differ from the active preset (the baseline when `active` is 0).
    pub fn edited(&self) -> bool {
        match self.active.checked_sub(1).and_then(|i| self.presets.get(i)) {
            Some(p) => self.differs_from(&p.weights, &p.assume),
            None => self.changed(),
        }
    }

    pub fn reset_all(&mut self) {
        let (w, a) = self.baseline_values();
        self.load(&w, &a);
        self.active = 0;
    }

    fn save_as(&mut self, name: &str) {
        let name: String = name.trim().chars().take(NAME_LIMIT).collect();
        if name.is_empty() || self.presets.len() >= MAX_USER_PRESETS {
            return;
        }
        let p = self.snapshot(&name);
        self.presets.push(p);
        self.active = self.presets.len();
    }

    fn delete(&mut self, i: usize) {
        if i < self.presets.len() {
            self.presets.remove(i);
            // The chips after it move up one place.
            if self.active == i + 1 {
                self.active = 0;
            } else if self.active > i + 1 {
                self.active -= 1;
            }
        }
    }

    fn update_active(&mut self) {
        if let Some(i) = self.active.checked_sub(1) {
            let name = self.presets.get(i).map(|p| p.name.clone());
            if let Some(name) = name {
                self.presets[i] = self.snapshot(&name);
            }
        }
    }

    /// The name suggested when saving: `Preset N`, the first one not taken.
    fn suggest_name(&self) -> String {
        (1..)
            .map(|n| format!("Preset {n}"))
            .find(|s| !self.presets.iter().any(|p| &p.name == s))
            .unwrap_or_default()
    }

    /// Review only: saves the name typed so far, as pressing Save would.
    pub fn save_demo(&mut self) {
        if let Some(n) = self.naming.take() {
            self.save_as(&n);
        }
    }

    pub fn start_naming(&mut self) {
        self.naming = Some(self.suggest_name());
        self.naming_fresh = true;
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// The value of a slider at a pointer x on its track.
fn value_at(x: f32, left: f32, right: f32, min: u8, max: u8, step: u8) -> u8 {
    let t = ((x - left) / (right - left)).clamp(0.0, 1.0);
    let raw = min as f32 + t * (max - min) as f32;
    let snapped = min as f32 + ((raw - min as f32) / step as f32).round() * step as f32;
    snapped.clamp(min as f32, max as f32) as u8
}

fn x_of(v: u8, left: f32, right: f32, min: u8, max: u8) -> f32 {
    left + (v - min) as f32 / (max - min) as f32 * (right - left)
}

/// One slider: a track with the value filled in accent, a tick at the baseline, an optional shaded
/// band and a knob. Drag or click to set; arrows move it by a step while it has focus. Returns
/// true when a change is finished (pointer let go, or a key press).
#[allow(clippy::too_many_arguments)]
fn slider(
    ui: &mut Ui,
    id: egui::Id,
    rect: Rect,
    value: &mut u8,
    baseline: u8,
    (min, max, step): (u8, u8, u8),
    band: Option<(u8, u8)>,
    tip: &str,
) -> bool {
    let resp = ui.interact(rect, id, egui::Sense::click_and_drag());
    let (left, right) = (rect.left() + KNOB_R + 1.0, rect.right() - KNOB_R - 1.0);
    let start_id = id.with("start");
    if resp.drag_started() || resp.is_pointer_button_down_on() && !resp.dragged() {
        ui.data_mut(|d| d.insert_temp(start_id, *value));
    }
    if let Some(pos) = resp.interact_pointer_pos() {
        if resp.dragged() || resp.is_pointer_button_down_on() || resp.clicked() {
            *value = value_at(pos.x, left, right, min, max, step);
        }
    }
    if resp.clicked() {
        resp.request_focus();
    }
    let mut done = false;
    if resp.drag_stopped() || resp.clicked() {
        let start: Option<u8> = ui.data(|d| d.get_temp(start_id));
        done = start != Some(*value);
    }
    if resp.has_focus() {
        let (dec, inc) = ui.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
            )
        });
        if dec && *value > min {
            *value = (*value).saturating_sub(step).max(min);
            done = true;
        }
        if inc && *value < max {
            *value = (*value + step).min(max);
            done = true;
        }
    }
    let p = ui.painter();
    let cy = rect.center().y;
    if let Some((lo, hi)) = band {
        let band_r = Rect::from_min_max(
            Pos2::new(x_of(lo, left, right, min, max), cy - 6.0),
            Pos2::new(x_of(hi, left, right, min, max), cy + 6.0),
        );
        p.rect_filled(band_r, 2.0, Color32::from_rgb(42, 42, 48));
    }
    let track = Rect::from_min_max(Pos2::new(left, cy - 2.0), Pos2::new(right, cy + 2.0));
    p.rect_filled(track, 2.0, LINE);
    let vx = x_of(*value, left, right, min, max);
    let fill = Rect::from_min_max(track.min, Pos2::new(vx, track.max.y));
    p.rect_filled(fill, 2.0, ACCENT);
    let bx = x_of(baseline, left, right, min, max);
    p.line_segment(
        [Pos2::new(bx, cy - 7.0), Pos2::new(bx, cy + 7.0)],
        Stroke::new(1.5, DIM),
    );
    let hot = resp.hovered() || resp.dragged();
    p.circle_filled(Pos2::new(vx, cy), KNOB_R, if hot { ACCENT } else { FG });
    if resp.has_focus() {
        p.circle_stroke(Pos2::new(vx, cy), KNOB_R + 3.0, Stroke::new(1.5, ACCENT));
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tip);
    done
}

/// The reset mark: a small open arc with an arrow head. Drawn, since the fonts have no such glyph.
fn reset_mark(p: &Painter, c: Pos2, col: Color32) {
    let r = 5.0;
    let pts: Vec<Pos2> = (0..=12)
        .map(|i| {
            let a = (40.0 + i as f32 * 25.0).to_radians();
            Pos2::new(c.x + r * a.cos(), c.y - r * a.sin())
        })
        .collect();
    let tip = *pts.first().unwrap_or(&c);
    p.add(egui::Shape::line(pts, Stroke::new(1.4, col)));
    p.add(egui::Shape::convex_polygon(
        vec![
            Pos2::new(tip.x + 3.5, tip.y - 1.5),
            Pos2::new(tip.x - 1.5, tip.y - 3.5),
            Pos2::new(tip.x - 1.5, tip.y + 2.5),
        ],
        col,
        Stroke::NONE,
    ));
}

fn reset_button(ui: &mut Ui, id: egui::Id, r: Rect, name: &str, baseline: String) -> bool {
    let resp = ui.interact(r, id, egui::Sense::click());
    reset_mark(
        ui.painter(),
        r.center(),
        if resp.hovered() { ACCENT } else { DIM },
    );
    let clicked = resp.clicked();
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(format!("Reset {name} to its baseline, {baseline}"));
    clicked
}

fn label(ui: &mut Ui, text: &str, right: Option<(&str, Color32)>) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(INNER_W, LABEL_H), egui::Sense::hover());
    let p = ui.painter();
    p.text(
        Pos2::new(r.left(), r.bottom() - 14.0),
        Align2::LEFT_CENTER,
        text,
        mono(10.5),
        DIM,
    );
    if let Some((t, c)) = right {
        p.text(
            Pos2::new(r.right(), r.bottom() - 14.0),
            Align2::RIGHT_CENTER,
            t,
            mono(10.0),
            c,
        );
    }
    hair(p, r.left(), r.right(), r.bottom() - 2.0);
}

/// A small outlined text button, 24 high, used for the footer and the naming row.
fn button(ui: &mut Ui, id: egui::Id, r: Rect, text: &str, on: bool, tip: &str) -> bool {
    let resp = ui.interact(r, id, egui::Sense::click());
    let hot = on && resp.hovered();
    let p = ui.painter();
    p.rect_stroke(
        r,
        2.0,
        Stroke::new(1.0, if hot { FAINT } else { LINE }),
        egui::StrokeKind::Inside,
    );
    p.text(
        r.center(),
        Align2::CENTER_CENTER,
        fit(p, text, r.width() - 12.0, "inter-medium", 11.5),
        font("inter-medium", 11.5),
        if !on {
            FAINT
        } else if hot {
            ACCENT
        } else {
            FG
        },
    );
    let clicked = on && resp.clicked();
    let resp = if on {
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    } else {
        resp
    };
    resp.on_hover_text(tip);
    clicked
}

/// Draws the drawer inside `content` and applies what the user does. Returns true when a
/// recompute should start (a slider was let go, a preset was chosen, or a reset was pressed).
pub fn drawer(ui: &mut Ui, content: Rect, st: &mut State) -> bool {
    if !st.open {
        return false;
    }
    let ctx = ui.ctx().clone();
    let mut recompute = false;
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        if st.naming.is_some() {
            st.naming = None;
        } else {
            st.open = false;
            return false;
        }
    }
    let top = content.min.y + TOP;
    let height = (content.max.y - top - 12.0).max(200.0);
    let id = ui.scope_id().with("customise-drawer");
    egui::Area::new(id)
        .order(egui::Order::Foreground)
        .fade_in(false)
        .fixed_pos(Pos2::new(content.max.x - MARGIN - DRAWER_W - 1.0, top))
        .show(&ctx, |ui| {
            egui::Frame::new()
                .fill(CHIP_FILL)
                .stroke(Stroke::new(1.0, FAINT))
                .corner_radius(6.0)
                .inner_margin(PAD)
                .show(ui, |ui| {
                    ui.set_width(INNER_W);
                    ui.set_min_height(height - 2.0 * PAD);
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    recompute |= contents(ui, st, height - 2.0 * PAD);
                });
        });
    recompute
}

fn contents(ui: &mut Ui, st: &mut State, height: f32) -> bool {
    let mut recompute = false;
    let id = ui.scope_id();

    // Title row.
    let (r, _) = ui.allocate_exact_size(Vec2::new(INNER_W, HEAD_H), egui::Sense::hover());
    ui.painter().text(
        Pos2::new(r.left(), r.center().y),
        Align2::LEFT_CENTER,
        "Customise",
        font("inter-medium", 15.0),
        FG,
    );
    let close =
        Rect::from_center_size(Pos2::new(r.right() - 10.0, r.center().y), Vec2::splat(24.0));
    let resp = ui.interact(close, id.with("close"), egui::Sense::click());
    ui.painter().text(
        close.center(),
        Align2::CENTER_CENTER,
        "\u{d7}",
        font("inter", 18.0),
        if resp.hovered() { FG } else { DIM },
    );
    if resp.clicked() {
        st.open = false;
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text("Close (Esc)");
    ui.painter().text(
        Pos2::new(r.right() - 30.0, r.center().y),
        Align2::RIGHT_CENTER,
        "SAMPLE VALUES",
        mono(10.0),
        WARN,
    );

    ui.add_space(6.0);
    let body_h = (height - HEAD_H - FOOT_H - 6.0).max(80.0);
    let mut area = egui::ScrollArea::vertical()
        .id_salt("customise-body")
        .max_height(body_h)
        .min_scrolled_height(body_h)
        .auto_shrink([false, false]);
    if let Some(y) = st.demo_scroll {
        area = area.vertical_scroll_offset(y);
    }
    area.show(ui, |ui| {
        ui.set_width(INNER_W);
        recompute |= presets(ui, st);
        recompute |= assumptions(ui, st);
        recompute |= weights(ui, st);
    });

    // Footer: reset everything, and update the active preset when it was edited.
    ui.add_space(8.0);
    let (r, _) = ui.allocate_exact_size(Vec2::new(INNER_W, FOOT_H - 8.0), egui::Sense::hover());
    hair(ui.painter(), r.left(), r.right(), r.top() - 4.0);
    let y = r.center().y - CHIP_H / 2.0 + 2.0;
    let reset = Rect::from_min_size(Pos2::new(r.left(), y), Vec2::new(CHIP_W, CHIP_H));
    if button(
        ui,
        id.with("reset-all"),
        reset,
        "Reset all to baseline",
        st.changed(),
        "Put every weight and assumption back to its baseline.",
    ) {
        st.reset_all();
        recompute = true;
    }
    if st.edited() && st.active > 0 {
        let upd = Rect::from_min_size(
            Pos2::new(r.left() + CHIP_W + GAP, y),
            Vec2::new(CHIP_W, CHIP_H),
        );
        let name = st
            .presets
            .get(st.active - 1)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        if button(
            ui,
            id.with("update"),
            upd,
            "Update preset",
            true,
            &format!("Save these values into the preset \u{201c}{name}\u{201d}."),
        ) {
            st.update_active();
        }
    }
    recompute
}

fn presets(ui: &mut Ui, st: &mut State) -> bool {
    let id = ui.scope_id().with("presets");
    let mut recompute = false;
    let edited = st.edited();
    label(ui, "PRESETS", edited.then_some(("EDITED", ACCENT)));
    ui.add_space(8.0);
    // Cells: Baseline, the user's presets, then the Save cell. Two columns, fixed size.
    let user = st.presets.len();
    let cells = 1 + user + usize::from(st.naming.is_none());
    let rows = cells.div_ceil(2);
    let (grid, _) = ui.allocate_exact_size(
        Vec2::new(INNER_W, rows as f32 * CHIP_H + (rows as f32 - 1.0) * GAP),
        egui::Sense::hover(),
    );
    let mut act: Option<(usize, bool)> = None; // (cell, delete)
    for cell in 0..cells {
        let (c, rw) = (cell % 2, cell / 2);
        let r = Rect::from_min_size(
            Pos2::new(
                grid.left() + c as f32 * (CHIP_W + GAP),
                grid.top() + rw as f32 * (CHIP_H + GAP),
            ),
            Vec2::new(CHIP_W, CHIP_H),
        );
        if cell > user {
            // The Save cell.
            let full = user >= MAX_USER_PRESETS;
            if button(
                ui,
                id.with("save"),
                r,
                "+ Save current",
                !full,
                if full {
                    "Five presets is the most. Delete one first."
                } else {
                    "Save these weights and assumptions as a preset."
                },
            ) {
                act = Some((cell, false));
            }
            continue;
        }
        let on = st.active == cell;
        let resp = ui.interact(r, id.with(("chip", cell)), egui::Sense::click());
        let p = ui.painter();
        p.rect_filled(
            r,
            3.0,
            if on {
                Color32::from_rgb(34, 22, 14)
            } else {
                CHIP_FILL
            },
        );
        p.rect_stroke(
            r,
            3.0,
            Stroke::new(
                1.0,
                if on {
                    ACCENT
                } else if resp.hovered() {
                    FAINT
                } else {
                    LINE
                },
            ),
            egui::StrokeKind::Inside,
        );
        let name = if cell == 0 {
            "Baseline"
        } else {
            st.presets[cell - 1].name.as_str()
        };
        let text_w = CHIP_W - 14.0 - if cell == 0 { 0.0 } else { 20.0 };
        p.text(
            Pos2::new(r.left() + 8.0, r.center().y),
            Align2::LEFT_CENTER,
            fit(p, name, text_w, "inter", 12.0),
            font("inter", 12.0),
            if on { ACCENT } else { FG },
        );
        let mut over_delete = false;
        if cell > 0 {
            let x = Rect::from_center_size(
                Pos2::new(r.right() - 12.0, r.center().y),
                Vec2::new(20.0, CHIP_H),
            );
            let dx = ui.interact(x, id.with(("del", cell)), egui::Sense::click());
            over_delete = dx.hovered();
            ui.painter().text(
                x.center(),
                Align2::CENTER_CENTER,
                "\u{d7}",
                font("inter", 14.0),
                if dx.hovered() { WARN } else { DIM },
            );
            if dx.clicked() {
                act = Some((cell, true));
            }
            dx.on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text("Delete this preset");
        }
        if resp.clicked() && !over_delete {
            act = Some((cell, false));
        }
        let tip = if cell == 0 {
            "Baseline: the researched starting weights and assumptions.".to_string()
        } else {
            format!("{}. Click to use it.", st.presets[cell - 1].name)
        };
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
            .on_hover_text(tip);
    }
    if let Some((cell, delete)) = act {
        if delete {
            st.delete(cell - 1);
        } else if cell > user {
            st.start_naming();
        } else if st.active != cell || st.edited() {
            st.active = cell;
            if cell == 0 {
                let (w, a) = st.baseline_values();
                st.load(&w, &a);
            } else {
                let p = st.presets[cell - 1].clone();
                st.load(&p.weights, &p.assume);
            }
            recompute = true;
        }
    }

    // The naming row: a field with a hairline, Save and Cancel.
    if let Some(mut name) = st.naming.take() {
        ui.add_space(GAP);
        let (row, _) = ui.allocate_exact_size(Vec2::new(INNER_W, CHIP_H), egui::Sense::hover());
        let btn_w = 52.0;
        let field = Rect::from_min_size(
            row.min,
            Vec2::new(INNER_W - 2.0 * btn_w - 2.0 * GAP, CHIP_H),
        );
        let mut edit_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(field.shrink2(Vec2::new(2.0, 0.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        let resp = edit_ui.add(
            egui::TextEdit::singleline(&mut name)
                .frame(egui::Frame::NONE)
                .font(font("inter", 12.5))
                .text_color(FG)
                .char_limit(NAME_LIMIT)
                .hint_text(
                    egui::RichText::new("Preset name")
                        .font(font("inter", 12.5))
                        .color(FAINT),
                )
                .desired_width(field.width() - 4.0),
        );
        if st.naming_fresh {
            resp.request_focus();
            st.naming_fresh = false;
        }
        hair(ui.painter(), field.left(), field.right(), field.bottom());
        let save_r = Rect::from_min_size(
            Pos2::new(field.right() + GAP, row.top()),
            Vec2::new(btn_w, CHIP_H),
        );
        let cancel_r = Rect::from_min_size(
            Pos2::new(save_r.right() + GAP, row.top()),
            Vec2::new(btn_w, CHIP_H),
        );
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let ok = !name.trim().is_empty();
        if button(
            ui,
            id.with("name-ok"),
            save_r,
            "Save",
            ok,
            "Save the preset.",
        ) || (enter && ok)
        {
            st.save_as(&name);
            recompute = false;
        } else if button(
            ui,
            id.with("name-no"),
            cancel_r,
            "Cancel",
            true,
            "Don't save a preset.",
        ) {
            // dropped
        } else {
            st.naming = Some(name);
        }
    }
    ui.add_space(4.0);
    recompute
}

fn assumptions(ui: &mut Ui, st: &mut State) -> bool {
    let id = ui.scope_id().with("assume");
    let mut recompute = false;
    label(ui, "ASSUMPTIONS", Some(("SHADED = ALSO CHECKED", FAINT)));
    for (i, a) in st.assume.iter_mut().enumerate() {
        let (r, _) = ui.allocate_exact_size(Vec2::new(INNER_W, ASSUME_H), egui::Sense::hover());
        let changed = a.value != a.baseline;
        let line = Rect::from_min_size(r.min + Vec2::new(0.0, 4.0), Vec2::new(INNER_W, 22.0));
        let name_w = INNER_W - VALUE_W - RESET_W - 2.0 * GAP - 14.0;
        let p = ui.painter();
        p.text(
            Pos2::new(line.left(), line.center().y),
            Align2::LEFT_CENTER,
            fit(p, a.name, name_w, "inter", 12.5),
            font("inter", 12.5),
            FG,
        );
        let name_hit = Rect::from_min_size(line.min, Vec2::new(name_w, 22.0));
        ui.interact(name_hit, id.with(("n", i)), egui::Sense::hover())
            .on_hover_text(format!(
                "{} Baseline {}%, plausible {}% to {}%.",
                a.tip, a.baseline, a.band.0, a.band.1
            ));
        ui.painter().text(
            Pos2::new(line.right() - RESET_W - GAP, line.center().y),
            Align2::RIGHT_CENTER,
            format!("{}%", a.value),
            mono(12.0),
            if changed { ACCENT } else { DIM },
        );
        if changed {
            let rr = Rect::from_center_size(
                Pos2::new(line.right() - RESET_W / 2.0, line.center().y),
                Vec2::new(RESET_W, 22.0),
            );
            if reset_button(
                ui,
                id.with(("r", i)),
                rr,
                a.name,
                format!("{}%", a.baseline),
            ) {
                a.value = a.baseline;
                recompute = true;
            }
        }
        let track = Rect::from_min_size(
            Pos2::new(r.left(), r.top() + 26.0),
            Vec2::new(INNER_W, TRACK_H),
        );
        let tip = format!("{}. Drag, click, or use the arrow keys.", a.name);
        let (min, max, step, baseline, band) = (a.min, a.max, a.step, a.baseline, a.band);
        if slider(
            ui,
            id.with(("s", i)),
            track,
            &mut a.value,
            baseline,
            (min, max, step),
            Some(band),
            &tip,
        ) {
            recompute = true;
        }
    }
    ui.add_space(6.0);
    recompute
}

fn weights(ui: &mut Ui, st: &mut State) -> bool {
    let id = ui.scope_id().with("weights");
    let mut recompute = false;
    label(ui, "WEIGHTS", Some(("0 TO 10, TICK = BASELINE", FAINT)));
    for (i, w) in st.weights.iter_mut().enumerate() {
        let (r, _) = ui.allocate_exact_size(Vec2::new(INNER_W, ROW_H), egui::Sense::hover());
        let changed = w.value != w.baseline;
        let name_r = Rect::from_min_size(r.min, Vec2::new(NAME_W, ROW_H));
        let p = ui.painter();
        p.text(
            Pos2::new(name_r.left(), r.center().y),
            Align2::LEFT_CENTER,
            fit(p, w.name, NAME_W - 4.0, "inter", 12.5),
            font("inter", 12.5),
            FG,
        );
        let track = Rect::from_min_size(
            Pos2::new(r.left() + NAME_W + GAP, r.center().y - TRACK_H / 2.0),
            Vec2::new(TRACK_W, TRACK_H),
        );
        let tip = format!(
            "{}. Baseline {}. Drag, click, or use the arrow keys.",
            w.name, w.baseline
        );
        let baseline = w.baseline;
        if slider(
            ui,
            id.with(("s", i)),
            track,
            &mut w.value,
            baseline,
            (0, 10, 1),
            None,
            &tip,
        ) {
            recompute = true;
        }
        let vx = track.right() + GAP + VALUE_W;
        ui.painter().text(
            Pos2::new(vx, r.center().y),
            Align2::RIGHT_CENTER,
            format!("{}", w.value),
            mono(12.0),
            if changed { ACCENT } else { DIM },
        );
        // Whole-row hover for the full name (a long one is cut with "…" above).
        ui.interact(name_r, id.with(("n", i)), egui::Sense::hover())
            .on_hover_text(format!(
                "{}. Baseline {}, now {}.",
                w.name, w.baseline, w.value
            ));
        if changed {
            let rr = Rect::from_center_size(
                Pos2::new(r.right() - RESET_W / 2.0, r.center().y),
                Vec2::new(RESET_W, ROW_H - 8.0),
            );
            if reset_button(ui, id.with(("r", i)), rr, w.name, format!("{}", w.baseline)) {
                w.value = w.baseline;
                recompute = true;
            }
        }
    }
    ui.add_space(4.0);
    recompute
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_value_snaps_and_clamps() {
        assert_eq!(value_at(0.0, 10.0, 110.0, 0, 10, 1), 0);
        assert_eq!(value_at(500.0, 10.0, 110.0, 0, 10, 1), 10);
        assert_eq!(value_at(60.0, 10.0, 110.0, 0, 10, 1), 5);
        // Steps of 5 on 30 to 100: the middle of the track snaps onto the grid.
        let v = value_at(60.0, 10.0, 110.0, 30, 100, 5);
        assert_eq!((v - 30) % 5, 0);
        assert!((30..=100).contains(&v));
    }

    #[test]
    fn presets_save_load_delete_and_track_edits() {
        let mut st = State::new();
        assert!(!st.changed() && !st.edited());
        st.weights[0].value = 9;
        assert!(st.changed() && st.edited());
        st.save_as("  Mine  ");
        assert_eq!(st.presets.last().map(|p| p.name.as_str()), Some("Mine"));
        assert_eq!(st.active, st.presets.len());
        assert!(!st.edited(), "a fresh preset matches its own values");
        st.weights[1].value = 1;
        assert!(st.edited());
        st.update_active();
        assert!(!st.edited());
        let n = st.presets.len();
        st.delete(n - 1);
        assert_eq!(
            st.active, 0,
            "deleting the active preset falls back to baseline"
        );
        st.reset_all();
        assert!(!st.changed());
    }

    #[test]
    fn at_most_five_user_presets_and_names_are_cut() {
        let mut st = State::new();
        for n in 0..10 {
            st.save_as(&format!("p{n}"));
        }
        assert_eq!(st.presets.len(), MAX_USER_PRESETS);
        st.presets.clear();
        st.save_as(&"x".repeat(60));
        assert_eq!(st.presets[0].name.chars().count(), NAME_LIMIT);
        st.save_as("   ");
        assert_eq!(st.presets.len(), 1, "a blank name saves nothing");
        assert_eq!(st.suggest_name(), "Preset 1");
    }
}
