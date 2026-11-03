// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 12 of docs/ux/design-plan.md: the Why screen, reasoning and skill value. One plain
//! sentence says why this build is the answer, a quiet line under it names the assumptions it
//! rests on (with a way to change them), and a list on a fixed 36 point row grid shows what each
//! skill is worth. Set and group skills come first with every tier spelled out; the rest follow,
//! most valuable first.
//!
//! Every number is modelled, never measured, and says so. The values here are invented sample
//! numbers: the real ones come from the optimizer in Phase 4. A skill the optimizer has not scored
//! says `unknown` and why. Comparing two builds and what changed are elements 13 and 14.

use super::*;

const ROW_H: f32 = 36.0;
const DOT_PITCH_W: f32 = DOT_PITCH;
const GAP: f32 = 16.0;
/// Column widths, counted from the right edge: next level, value, why it is here, level.
const NEXT_W: f32 = 80.0;
const VALUE_W: f32 = 176.0;
const BAR_W: f32 = 84.0;
const WHY_W: f32 = 84.0;
const LEVEL_W: f32 = 104.0;
/// Where the sentence sits, and the list under the column labels.
const REASON_TOP: f32 = 90.0;
const LABELS_Y: f32 = 142.0;
const LIST_TOP: f32 = 154.0;
const BUTTON_W: f32 = 124.0;
const BUTTON_H: f32 = 24.0;
/// The sentence's font size.
const SENTENCE_FACE: f32 = 14.0;

/// What a skill is worth in the model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Value {
    /// Adds this share of one headline axis. `tenths` is percent times ten; `next` is what one
    /// more level would add, `None` when the skill is at its top level.
    Gain {
        axis: Axis,
        tenths: u16,
        next: Option<u16>,
    },
    /// A comfort skill: scored by its weight (set in Customise), not in the damage number.
    Comfort,
}

fn percent(tenths: u16) -> String {
    format!("+{}.{}%", tenths / 10, tenths % 10)
}

fn axis_word(a: Axis) -> &'static str {
    match a {
        Axis::Damage => "damage",
        Axis::Toughness => "toughness",
    }
}

/// The one-sentence reason for the build. Plain words, built from what the user chose.
pub fn verdict(data: &BuildData) -> String {
    let lead = match data.alt {
        Alt::Best => "The highest modelled damage".to_string(),
        Alt::Safe => "The most damage that still keeps you on your feet".to_string(),
        Alt::Defensive => "The most defence and resistance".to_string(),
        Alt::Monster => match data.monster {
            Some(p) => format!("The highest modelled damage against {}", p.label()),
            None => "The highest modelled damage against the monster you pick".to_string(),
        },
        Alt::General => "Steady damage against most monsters".to_string(),
        Alt::Gogma => "The best build around your Gogma gear".to_string(),
    };
    let who = match data.mode {
        PlayMode::Solo => "",
        PlayMode::Multiplayer => " in a group",
    };
    let musts: Vec<String> = data
        .must_haves
        .iter()
        .map(|m| format!("{} {}", m.name, m.level))
        .collect();
    let keeps = match musts.as_slice() {
        [] => String::new(),
        [one] => format!(", keeping {one}"),
        [rest @ .., last] => format!(", keeping {} and {last}", rest.join(", ")),
    };
    format!("{lead}{who}{keeps}.")
}

/// The quiet line under the sentence.
fn assumptions(data: &BuildData, c: &CustomiseState) -> String {
    let lead = if data.skills.iter().any(|k| k.value.is_none()) {
        "SOME SKILLS NOT SCORED YET"
    } else {
        "MODELLED"
    };
    let mut parts = vec![lead.to_string(), "unlocked gear only".to_string()];
    for a in &c.assume {
        parts.push(format!("{}% {}", a.value, a.name.to_lowercase()));
    }
    parts.push(if c.changed() {
        "weights changed".to_string()
    } else {
        "weights at baseline".to_string()
    });
    parts.join(" \u{b7} ")
}

/// The sort order of the normal skills: scored gains first (largest first), then comfort skills
/// (heaviest weight first), then skills not scored yet.
fn rank(k: &Skill) -> (u8, std::cmp::Reverse<u16>) {
    match k.value {
        Some(Value::Gain { tenths, .. }) => (0, std::cmp::Reverse(tenths)),
        Some(Value::Comfort) => (1, std::cmp::Reverse(u16::from(k.weight.unwrap_or(0)))),
        None => (2, std::cmp::Reverse(0)),
    }
}

enum Row<'a> {
    Head(&'static str, usize),
    /// A set or group skill.
    Pinned(&'a Skill),
    /// One tier of the set or group skill above it.
    Tier(&'a Skill, &'a Tier),
    Skill(&'a Skill),
}

fn rows(data: &BuildData) -> Vec<Row<'_>> {
    let pinned: Vec<&Skill> = data.skills.iter().filter(|k| k.pinned()).collect();
    let mut normal: Vec<&Skill> = data.skills.iter().filter(|k| !k.pinned()).collect();
    normal.sort_by_key(|k| rank(k));
    let mut v = Vec::new();
    if !pinned.is_empty() {
        v.push(Row::Head("SET AND GROUP SKILLS", pinned.len()));
        for k in pinned {
            v.push(Row::Pinned(k));
            v.extend(k.tiers.iter().map(|t| Row::Tier(k, t)));
        }
    }
    if !normal.is_empty() {
        v.push(Row::Head("SKILLS", normal.len()));
        v.extend(normal.into_iter().map(Row::Skill));
    }
    v
}

/// The largest scored gain among the shown skills: the full bar.
fn bar_max(data: &BuildData) -> u16 {
    data.skills
        .iter()
        .filter_map(|k| match k.value {
            Some(Value::Gain { tenths, .. }) => Some(tenths),
            _ => None,
        })
        .max()
        .unwrap_or(1)
        .max(1)
}

/// The sentence, the assumptions line and the Change weights button.
pub(super) fn header(
    p: &Painter,
    hits: &mut Vec<(Rect, Hit)>,
    content: Rect,
    data: &BuildData,
    c: &CustomiseState,
) {
    let ppp = p.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let left = content.min.x + MARGIN;
    let right = content.max.x - MARGIN;
    let top = content.min.y;
    let hover = p.ctx().pointer_hover_pos();
    let b = Rect::from_min_size(
        Pos2::new(
            snap(right - BUTTON_W),
            snap(top + REASON_TOP - BUTTON_H / 2.0 + 2.0),
        ),
        Vec2::new(BUTTON_W, BUTTON_H),
    );
    let hot = hover.is_some_and(|h| b.contains(h));
    p.rect_filled(b, 2.0, CHIP_FILL);
    p.rect_stroke(
        b,
        2.0,
        Stroke::new(1.0, if hot { DIM } else { FAINT }),
        egui::StrokeKind::Inside,
    );
    p.text(
        b.center(),
        Align2::CENTER_CENTER,
        "Change weights",
        font("inter", 12.0),
        if hot { FG } else { DIM },
    );
    hits.push((b, Hit::WhyCustomise));
    let text_w = b.left() - GAP - left;
    p.text(
        Pos2::new(left, snap(top + REASON_TOP + 2.0)),
        Align2::LEFT_CENTER,
        fit(p, &verdict(data), text_w, "inter", SENTENCE_FACE),
        font("inter", SENTENCE_FACE),
        FG,
    );
    p.text(
        Pos2::new(left, snap(top + REASON_TOP + 26.0)),
        Align2::LEFT_CENTER,
        fit(p, &assumptions(data, c), right - left, "mono", 10.5),
        mono(10.5),
        DIM,
    );
    // Column labels over the list.
    let y = snap(top + LABELS_Y);
    let area = Rect::from_min_max(
        Pos2::new(left, top + LIST_TOP),
        Pos2::new(right, content.max.y),
    );
    let cols = Cols::new(area);
    for (x, align, label) in [
        (left, Align2::LEFT_CENTER, "SKILL"),
        (cols.level, Align2::LEFT_CENTER, "LEVEL"),
        (cols.why, Align2::LEFT_CENTER, "WHY HERE"),
        (cols.value, Align2::LEFT_CENTER, "VALUE, MODELLED"),
        (right, Align2::RIGHT_CENTER, "NEXT LEVEL"),
    ] {
        p.text(Pos2::new(x, y), align, label, mono(10.0), FAINT);
    }
    hair(p, left, right, y + 8.0);
}

/// Left edges of the columns, from the row's right edge.
struct Cols {
    level: f32,
    why: f32,
    value: f32,
    next_right: f32,
}

impl Cols {
    fn new(r: Rect) -> Self {
        let next_left = r.right() - NEXT_W;
        let value = next_left - GAP - VALUE_W;
        let why = value - GAP - WHY_W;
        let level = why - GAP - LEVEL_W;
        Cols {
            level,
            why,
            value,
            next_right: r.right(),
        }
    }
}

fn tag(p: &Painter, snap: &dyn Fn(f32) -> f32, left: f32, cy: f32, text: &str, col: Color32) {
    let g = p.layout_no_wrap(text.to_string(), mono(9.0), col);
    let r = Rect::from_min_size(
        Pos2::new(snap(left), snap(cy - 7.5)),
        Vec2::new(g.size().x + 10.0, 15.0),
    );
    p.rect_stroke(r, 2.0, Stroke::new(1.0, FAINT), egui::StrokeKind::Inside);
    p.galley(r.center() - g.size() / 2.0, g, col);
}

fn dots(p: &Painter, snap: &dyn Fn(f32) -> f32, x: f32, cy: f32, level: u8, max: u8) {
    for d in 0..max {
        let c = Pos2::new(snap(x + DOT_R + f32::from(d) * DOT_PITCH_W), snap(cy));
        if d < level {
            p.circle_filled(c, DOT_R, ACCENT);
        } else {
            p.circle_stroke(c, DOT_R - 0.5, Stroke::new(1.0, FAINT));
        }
    }
}

fn row_tip(k: &Skill) -> String {
    match (k.kind, k.value) {
        (SkillKind::Normal, Some(Value::Gain { axis, tenths, next })) => {
            let mut t = format!(
                "{}, level {} of {}. Worth about {} {} in the model compared with leaving it out.",
                k.name,
                k.level,
                k.max,
                percent(tenths),
                axis_word(axis)
            );
            match next {
                Some(n) => t += &format!(" One more level adds about {}.", percent(n)),
                None => t += " Already at its top level.",
            }
            t + " Modelled, not measured."
        }
        (SkillKind::Normal, Some(Value::Comfort)) => format!(
            "{}, level {} of {}. A comfort skill: it is scored by its weight{}, not in the damage number. Change the weight in Customise.",
            k.name,
            k.level,
            k.max,
            k.weight.map_or(String::new(), |w| format!(" ({w})"))
        ),
        (SkillKind::Normal, None) => format!(
            "{}, level {} of {}. Not scored yet: the optimizer has not valued this skill.",
            k.name, k.level, k.max
        ),
        (_, v) => {
            let mut t = skill_tip(k);
            if let Some(Value::Gain { axis, tenths, .. }) = v {
                t += &format!(
                    "\nWorth about {} {} in the model. Modelled, not measured.",
                    percent(tenths),
                    axis_word(axis)
                );
            }
            t
        }
    }
}

/// The list. Drawn into `ui` after the shell, under the header.
pub fn list(ui: &mut Ui, content: Rect, data: &BuildData, scroll_to: Option<f32>) {
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let left = content.min.x + MARGIN;
    let right = content.max.x - MARGIN;
    let top = snap(content.min.y + LIST_TOP);
    let area = Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, content.max.y - 12.0));
    // The full sentence on hover, since the line is cut with an ellipsis when it does not fit.
    let sentence = Rect::from_min_size(
        Pos2::new(left, content.min.y + REASON_TOP - 10.0),
        Vec2::new(area.width() - BUTTON_W - GAP, 24.0),
    );
    ui.interact(
        sentence,
        ui.scope_id().with("why-sentence"),
        egui::Sense::hover(),
    )
    .on_hover_text(verdict(data));
    if area.height() < ROW_H {
        return;
    }
    let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(area));
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let all = rows(data);
    if all.is_empty() {
        let (r, _) = ui.allocate_exact_size(Vec2::new(area.width(), ROW_H), egui::Sense::hover());
        ui.painter().text(
            Pos2::new(r.left(), r.center().y),
            Align2::LEFT_CENTER,
            "No skills yet. Pick a weapon and armour, or wait for the first build.",
            font("inter", 12.5),
            DIM,
        );
        return;
    }
    let cols = Cols::new(area);
    let top_value = bar_max(data);
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt("why")
        .auto_shrink([false, false])
        .max_height(area.height());
    if let Some(y) = scroll_to {
        scroll = scroll.vertical_scroll_offset(y);
    }
    scroll.show_rows(&mut ui, ROW_H, all.len(), |ui, range| {
        for row in &all[range] {
            let (r, resp) =
                ui.allocate_exact_size(Vec2::new(area.width(), ROW_H), egui::Sense::hover());
            let r = Rect::from_min_size(Pos2::new(snap(r.left()), snap(r.top())), r.size());
            let p = ui.painter();
            let cy = snap(r.center().y);
            match row {
                Row::Head(title, n) => {
                    p.text(
                        Pos2::new(r.left(), cy),
                        Align2::LEFT_CENTER,
                        *title,
                        font("inter-medium", 13.5),
                        FG,
                    );
                    p.text(
                        Pos2::new(r.right(), cy),
                        Align2::RIGHT_CENTER,
                        n.to_string(),
                        mono(10.5),
                        FAINT,
                    );
                    hair(p, r.left(), r.right(), r.bottom() - 0.5);
                }
                Row::Tier(k, t) => {
                    let active = t.pieces <= k.level;
                    let x = r.left() + 16.0;
                    let c = Pos2::new(snap(x + DOT_R), cy);
                    if active {
                        p.circle_filled(c, DOT_R, ACCENT);
                    } else {
                        p.circle_stroke(c, DOT_R - 0.5, Stroke::new(1.0, FAINT));
                    }
                    p.text(
                        Pos2::new(x + 2.0 * DOT_R + 12.0, cy),
                        Align2::LEFT_CENTER,
                        format!("{} pieces", t.pieces),
                        mono(10.5),
                        if active { FG } else { DIM },
                    );
                    let tx = x + 2.0 * DOT_R + 12.0 + 64.0;
                    p.text(
                        Pos2::new(tx, cy),
                        Align2::LEFT_CENTER,
                        fit(p, t.full, r.right() - tx, "inter", 12.0),
                        font("inter", 12.0),
                        if active { FG } else { DIM },
                    );
                    if std::ptr::eq(*t, k.tiers.last().unwrap_or(*t)) {
                        hair(p, r.left(), r.right(), r.bottom() - 0.5);
                    }
                    resp.on_hover_text(format!(
                        "{} pieces: {}\n{}",
                        t.pieces,
                        t.full,
                        if active {
                            "Active now."
                        } else {
                            "Not active: not enough pieces worn."
                        }
                    ));
                }
                Row::Pinned(k) | Row::Skill(k) => {
                    let pinned = matches!(row, Row::Pinned(_));
                    let face = if pinned { "inter-medium" } else { "inter" };
                    p.text(
                        Pos2::new(r.left(), cy),
                        Align2::LEFT_CENTER,
                        fit(p, k.name, cols.level - GAP - r.left(), face, 13.0),
                        font(face, 13.0),
                        FG,
                    );
                    // Level: dots for a normal skill, pieces worn for a set or group skill.
                    if pinned {
                        p.text(
                            Pos2::new(cols.level, cy),
                            Align2::LEFT_CENTER,
                            format!("{} of {} pieces", k.level, k.max),
                            mono(11.0),
                            DIM,
                        );
                    } else {
                        dots(p, &snap, cols.level, cy, k.level, k.max);
                        p.text(
                            Pos2::new(cols.level + LEVEL_W, cy),
                            Align2::RIGHT_CENTER,
                            k.level.to_string(),
                            mono(11.5),
                            DIM,
                        );
                    }
                    // Why it is here.
                    let word = |text: String, col: Color32| {
                        p.text(
                            Pos2::new(cols.why, cy),
                            Align2::LEFT_CENTER,
                            text,
                            mono(10.5),
                            col,
                        );
                    };
                    match (k.kind, k.must, k.value) {
                        (SkillKind::Set, ..) => tag(p, &snap, cols.why, cy, "SET", DIM),
                        (SkillKind::Group, ..) => tag(p, &snap, cols.why, cy, "GROUP", DIM),
                        (_, true, _) => word("MUST-HAVE".into(), ACCENT),
                        (_, _, Some(Value::Gain { axis, .. })) => {
                            word(axis_word(axis).to_uppercase(), DIM)
                        }
                        (_, _, Some(Value::Comfort)) => word("COMFORT".into(), DIM),
                        _ => {}
                    }
                    // Value and next level.
                    let value_right = cols.value + VALUE_W;
                    match k.value {
                        Some(Value::Gain { tenths, next, .. }) => {
                            let track = Rect::from_min_size(
                                Pos2::new(snap(cols.value), snap(cy - 2.0)),
                                Vec2::new(BAR_W, 4.0),
                            );
                            p.rect_filled(track, 2.0, LINE);
                            let w = (BAR_W * f32::from(tenths) / f32::from(top_value)).max(2.0);
                            p.rect_filled(
                                Rect::from_min_size(track.min, Vec2::new(snap(w), 4.0)),
                                2.0,
                                DIM,
                            );
                            p.text(
                                Pos2::new(value_right, cy),
                                Align2::RIGHT_CENTER,
                                percent(tenths),
                                mono(11.5),
                                FG,
                            );
                            let (t, col) = match next {
                                _ if pinned => ("\u{2013}".to_string(), FAINT),
                                Some(n) => (percent(n), DIM),
                                None => ("Maxed".to_string(), FAINT),
                            };
                            p.text(
                                Pos2::new(cols.next_right, cy),
                                Align2::RIGHT_CENTER,
                                t,
                                mono(11.5),
                                col,
                            );
                        }
                        Some(Value::Comfort) => {
                            p.text(
                                Pos2::new(value_right, cy),
                                Align2::RIGHT_CENTER,
                                k.weight
                                    .map_or("by weight".to_string(), |w| format!("weight {w}")),
                                mono(11.5),
                                DIM,
                            );
                            p.text(
                                Pos2::new(cols.next_right, cy),
                                Align2::RIGHT_CENTER,
                                "\u{2013}",
                                mono(11.5),
                                FAINT,
                            );
                        }
                        None => {
                            p.text(
                                Pos2::new(value_right, cy),
                                Align2::RIGHT_CENTER,
                                "unknown",
                                mono(11.5),
                                FAINT,
                            );
                        }
                    }
                    if !pinned {
                        hair(p, r.left(), r.right(), r.bottom() - 0.5);
                    }
                    resp.on_hover_text(row_tip(k));
                }
            }
        }
    });
}
