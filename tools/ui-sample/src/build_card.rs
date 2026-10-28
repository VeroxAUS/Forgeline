// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Elements 4 and 5 of docs/ux/design-plan.md: the build card, the weapon and armour slots with
//! their icons, and the skills strip beside it, drawn in the D3 style (docs/ux/style-guide.md)
//! inside the element 2 shell and fed by the element 3 icon cache.
//!
//! Everything shown is sample data with invented names, labelled as such. The real build arrives
//! with the optimizer (Phase 4). Rows lay out from the width they get, so the card resizes. Nothing
//! animates and a repaint is requested only when an icon arrives.

use super::icons::{IconCache, IconSource};
use super::shell::{self, Shell, Status};
use super::{font, render, Icon};
use egui::{Align2, Color32, Context, Painter, Pos2, Rect, Stroke, Ui, Vec2};
use std::sync::Arc;

const MARGIN: f32 = 28.0;
const ROW_H: f32 = 48.0;
const ROW_STEP: f32 = 54.0;
const TILE: f32 = 34.0;
/// Every decoration slot circle is this size; its level is the number inside.
const SLOT_R: f32 = 7.0;
/// Gap between neighbouring slot circles, and the slot positions per row (three is what the sample uses; more are not drawn).
const SLOT_GAP: f32 = 6.0;
const SLOT_PITCH: f32 = SLOT_R * 2.0 + SLOT_GAP;
const SLOT_COLS: usize = 3;
/// Width of the status column (`Missing 2` fits).
const STATUS_W: f32 = 72.0;
/// Skills strip: every dot has this outer radius (filled and outlined alike) and sits on a fixed
/// pitch, so dots of different skills line up in columns.
const DOT_R: f32 = 4.0;
const DOT_PITCH: f32 = 13.0;
const SKILL_ROW: f32 = 38.0;
/// Compact rows (used when the list is long): one line, dots in a fixed column at this offset.
const COMPACT_ROW: f32 = 22.0;
const COMPACT_DOTS_X: f32 = 120.0;
/// Width of the side strip, and the gap between it and the equipment rows.
const SKILLS_W: f32 = 232.0;
const SKILLS_GAP: f32 = 32.0;
/// The equipment rows keep at least this width; below it the strip drops under the card.
const EQUIP_MIN_W: f32 = 480.0;
/// Each strip column is at least this wide (name, dots and level number).
const SKILL_COL_MIN_W: f32 = 200.0;
const ACCENT: Color32 = Color32::from_rgb(255, 138, 61);
const ACCENT_INK: Color32 = Color32::from_rgb(26, 12, 2);
const BG: Color32 = Color32::from_rgb(10, 10, 11);

const FG: Color32 = Color32::from_rgb(236, 236, 238);
const DIM: Color32 = Color32::from_rgb(140, 140, 148);
const FAINT: Color32 = Color32::from_rgb(82, 82, 90);
const LINE: Color32 = Color32::from_rgb(34, 34, 38);
const OK: Color32 = Color32::from_rgb(76, 195, 138);
const WARN: Color32 = Color32::from_rgb(229, 176, 74);

/// One decoration slot on a piece: its size (1 to 3) and whether a decoration sits in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DecoSlot {
    pub level: u8,
    pub filled: bool,
}

/// A piece in the build. `icon_id` asks the icon cache for the game's icon; `glyph` shows until
/// it arrives and when there is none.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Piece {
    pub icon_id: u32,
    pub glyph: Icon,
    pub name: &'static str,
    /// One line, plain: the skills this piece brings.
    pub detail: &'static str,
    pub slots: Vec<DecoSlot>,
    /// Parts still missing; 0 means owned.
    pub missing: u8,
}

/// What kind of skill this is. Set and group skills matter far more than normal ones, so the
/// strip pins them at the top and draws them differently.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SkillKind {
    Normal,
    Set,
    Group,
}

/// How set and group skills show their progress (two variants for the maintainer to compare).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SetStyle {
    /// A numbered circle per threshold (`2` and `4` pieces), filled once reached.
    Pips,
    /// Plain words: `3 of 4 pieces`.
    Text,
}

/// One skill in the build. For a normal skill `level` is its total level and `max` the most it
/// can reach. For a set or group skill `level` is the pieces worn, `max` the highest threshold and
/// `steps` the piece counts where a bonus starts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Skill {
    pub name: &'static str,
    pub level: u8,
    pub max: u8,
    pub kind: SkillKind,
    pub steps: &'static [u8],
    /// A must-have the user set. Never cut from the strip.
    pub must: bool,
    /// The optimizer's weight for this skill, when there is one (Phase 4). Without it the strip
    /// falls back to level, lowest first, when it has to cut.
    pub weight: Option<u8>,
}

impl Skill {
    pub fn normal(name: &'static str, level: u8, max: u8) -> Self {
        Skill {
            name,
            level,
            max,
            kind: SkillKind::Normal,
            steps: &[],
            must: false,
            weight: None,
        }
    }

    /// A set or group skill: `pieces` worn, bonuses at each of `steps`.
    pub fn bonus(kind: SkillKind, name: &'static str, pieces: u8, steps: &'static [u8]) -> Self {
        Skill {
            name,
            level: pieces,
            max: steps.last().copied().unwrap_or(0),
            kind,
            steps,
            must: false,
            weight: None,
        }
    }

    fn pinned(&self) -> bool {
        self.kind != SkillKind::Normal
    }

    /// Higher is kept longer when the strip has to cut. Must-haves, set and group skills never go.
    fn keep_rank(&self) -> (u8, u8, u8) {
        if self.must || self.pinned() {
            (2, 0, 0)
        } else {
            (
                u8::from(self.weight.is_some()),
                self.weight.unwrap_or(0),
                self.level,
            )
        }
    }
}

/// The card's data: the weapon, up to five armour pieces and the charm. `None` is an empty slot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BuildData {
    pub weapon: Option<Piece>,
    pub armour: [Option<Piece>; 5],
    pub charm: Option<Piece>,
    /// Skill totals for the whole build, in the order to show them.
    pub skills: Vec<Skill>,
}

impl BuildData {
    /// How many pieces still need parts.
    pub fn pieces_missing(&self) -> usize {
        self.all().filter(|p| p.missing > 0).count()
    }

    fn all(&self) -> impl Iterator<Item = &Piece> {
        self.weapon
            .iter()
            .chain(self.armour.iter().flatten())
            .chain(self.charm.iter())
    }

    /// Invented pieces for the review screenshots. Never the user's real build.
    pub fn sample() -> Self {
        let s = |level, filled| DecoSlot { level, filled };
        BuildData {
            weapon: Some(Piece {
                icon_id: 0,
                glyph: Icon::Weapon,
                name: "Sample Bowgun III",
                detail: "Sharpshot 2 \u{b7} Rapid fire",
                slots: vec![s(2, true), s(1, false)],
                missing: 0,
            }),
            armour: [
                Some(Piece {
                    icon_id: 1,
                    glyph: Icon::Head,
                    name: "Sample Helm \u{3b1}",
                    detail: "Weakness exploit 2",
                    slots: vec![s(1, true)],
                    missing: 2,
                }),
                Some(Piece {
                    icon_id: 2,
                    glyph: Icon::Chest,
                    name: "Sample Mail \u{3b2}",
                    detail: "Critical eye 3 \u{b7} Ammo saver 1",
                    slots: vec![s(3, true), s(1, false)],
                    missing: 0,
                }),
                Some(Piece {
                    icon_id: 3,
                    glyph: Icon::Arms,
                    name: "Sample Vambraces \u{3b1}",
                    detail: "Marksman 2",
                    slots: vec![s(2, false), s(1, false), s(1, false)],
                    missing: 0,
                }),
                Some(Piece {
                    icon_id: 4,
                    glyph: Icon::Waist,
                    name: "Sample Coil \u{3b3}",
                    detail: "Evade window 1",
                    slots: vec![],
                    missing: 0,
                }),
                // Id 6 has no icon in the sample source, to show the plain glyph in a real row.
                Some(Piece {
                    icon_id: 6,
                    glyph: Icon::Legs,
                    name: "Sample Greaves \u{3b2}",
                    detail: "Weakness exploit 1 \u{b7} Stamina surge 2",
                    slots: vec![s(2, true)],
                    missing: 1,
                }),
            ],
            charm: None,
            skills: vec![
                Skill::bonus(SkillKind::Set, "Set bonus Alpha", 3, &[2, 4]),
                Skill::bonus(SkillKind::Group, "Group skill Gamma", 3, &[3]),
                Skill {
                    must: true,
                    ..Skill::normal("Weakness exploit", 4, 5)
                },
                Skill::normal("Critical eye", 3, 5),
                Skill::normal("Sharpshot", 2, 3),
                Skill::normal("Marksman", 2, 3),
                Skill::normal("Ammo saver", 3, 3),
                Skill::normal("Stamina surge", 2, 3),
            ],
        }
    }
}

/// A realistic full endgame build's skills (invented names): 22 skills, with set and group
/// skills first and the rest most important first, each with a sample weight.
pub fn sample_endgame_skills() -> Vec<Skill> {
    let mut v = vec![
        Skill::bonus(SkillKind::Set, "Set bonus Alpha", 4, &[2, 4]),
        Skill::bonus(SkillKind::Set, "Set bonus Beta", 2, &[2, 4]),
        Skill::bonus(SkillKind::Group, "Group skill Gamma", 3, &[3]),
        Skill::bonus(SkillKind::Group, "Group skill Delta", 2, &[3]),
    ];
    let normal = [
        ("Weakness exploit", 5, 5, true),
        ("Critical eye", 5, 5, true),
        ("Sharpshot", 3, 3, false),
        ("Critical boost", 5, 5, false),
        ("Marksman", 3, 3, false),
        ("Attack boost", 4, 5, false),
        ("Ammo saver", 3, 3, false),
        ("Evade extender", 3, 3, false),
        ("Stamina surge", 3, 3, false),
        ("Recovery speed", 3, 3, false),
        ("Constitution", 5, 5, false),
        ("Rapid fire", 1, 1, false),
        ("Quick sheathe", 3, 3, false),
        ("Item prolonger", 3, 3, false),
        ("Partbreaker", 3, 3, false),
        ("Mind's eye", 3, 3, false),
        ("Fortify", 1, 1, false),
        ("Evade window", 5, 5, false),
    ];
    for (i, (name, level, max, must)) in normal.into_iter().enumerate() {
        v.push(Skill {
            must,
            weight: Some(100 - 5 * i as u8),
            ..Skill::normal(name, level, max)
        });
    }
    v
}

/// What a hover rectangle explains.
#[derive(Clone, Copy)]
enum Hit {
    Slot(u8),
    Skill(Skill),
    /// The "+N more on Why" line.
    Why,
}

/// The build card screen: shell state, the card's data and the icon cache.
pub struct BuildCard {
    pub shell: Shell,
    pub data: BuildData,
    pub cache: IconCache,
    /// Slot circles drawn last frame, for the hover tooltip.
    hits: Vec<(Rect, Hit)>,
    /// Which of the two set and group skill looks to draw.
    pub set_style: SetStyle,
}

impl BuildCard {
    pub fn new(ctx: &Context, source: Arc<dyn IconSource>) -> Self {
        BuildCard {
            shell: Shell::new(),
            data: BuildData::sample(),
            cache: IconCache::new(ctx, source),
            hits: Vec::new(),
            set_style: SetStyle::Text,
        }
    }
}

/// Draws the shell with the build card in its content area (Build section only).
pub fn show(ui: &mut Ui, app: &mut BuildCard) {
    app.cache.poll();
    let data = &app.data;
    let cache = &mut app.cache;
    let hits = &mut app.hits;
    let set_style = app.set_style;
    hits.clear();
    shell::show_with(ui, &mut app.shell, &mut |p, content, shell| {
        shell::show_content(p, content, shell);
        if shell.section == shell::Section::Build {
            card(p, content, data, cache, hits, &shell.status, set_style);
        } else {
            placeholder(p, content);
        }
    });
    let hits = std::mem::take(&mut app.hits);
    for (i, (rect, hit)) in hits.iter().enumerate() {
        let sense = match hit {
            Hit::Why => egui::Sense::click(),
            _ => egui::Sense::hover(),
        };
        let tip = ui.interact(*rect, ui.scope_id().with(("hit", i)), sense);
        match hit {
            Hit::Slot(level) => {
                tip.on_hover_text(format!("Decoration slot, level {level}"));
            }
            Hit::Skill(k) => {
                tip.on_hover_text(skill_tip(k));
            }
            Hit::Why => {
                if tip.clicked() {
                    app.shell.section = shell::Section::Why;
                }
                tip.on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text("Open Why for the full list");
            }
        }
    }
    app.hits = hits;
}

fn skill_tip(k: &Skill) -> String {
    match k.kind {
        SkillKind::Normal => format!("{}, level {} of {}", k.name, k.level, k.max),
        SkillKind::Set | SkillKind::Group => {
            let steps: Vec<String> = k.steps.iter().map(u8::to_string).collect();
            format!(
                "{}, {} pieces worn, bonus at {} pieces",
                k.name,
                k.level,
                steps.join(" and ")
            )
        }
    }
}

fn mono(size: f32) -> egui::FontId {
    font("mono", size)
}

fn placeholder(p: &Painter, content: Rect) {
    p.text(
        Pos2::new(content.min.x + MARGIN, content.min.y + 114.0),
        Align2::LEFT_CENTER,
        "Not part of this element.",
        font("inter", 12.5),
        DIM,
    );
}

fn card(
    p: &Painter,
    content: Rect,
    data: &BuildData,
    cache: &mut IconCache,
    hits: &mut Vec<(Rect, Hit)>,
    status: &Status,
    set_style: SetStyle,
) {
    let left = content.min.x + MARGIN;
    let right = content.max.x - MARGIN;
    p.text(
        Pos2::new(right, content.min.y + 30.0),
        Align2::RIGHT_CENTER,
        "SAMPLE DATA, NOT YOUR SAVE",
        mono(10.5),
        WARN,
    );

    // One quiet line under the title: what is missing, in words. While the build recomputes the
    // last build stays on screen, so this line is the only thing that notes it.
    let summary = match (status, data.pieces_missing()) {
        (Status::Recomputing, _) => "Showing the last build while it updates".to_string(),
        (_, 0) => "Everything here is owned".to_string(),
        (_, 1) => "1 piece needs parts".to_string(),
        (_, n) => format!("{n} pieces need parts"),
    };
    p.text(
        Pos2::new(left, content.min.y + 86.0),
        Align2::LEFT_CENTER,
        summary,
        font("inter", 12.5),
        DIM,
    );

    // Wide enough: the skills sit in a side strip level with the weapon label. Otherwise they go
    // under the charm, full width.
    let side = right - left - SKILLS_W - SKILLS_GAP >= EQUIP_MIN_W;
    let top = content.min.y + 120.0;
    let eq_right = if side {
        right - SKILLS_W - SKILLS_GAP
    } else {
        right
    };
    if side {
        skills(
            p,
            hits,
            right - SKILLS_W,
            right,
            top,
            content.max.y - 16.0,
            &data.skills,
            set_style,
        );
    }
    let right = eq_right;
    let mut y = top;
    y = section(p, left, right, y, "WEAPON");
    y = row(
        p,
        cache,
        hits,
        left,
        right,
        y,
        1,
        data.weapon.as_ref(),
        "No weapon",
    );
    y += 14.0;
    y = section(p, left, right, y, "ARMOUR");
    for (i, piece) in data.armour.iter().enumerate() {
        y = row(
            p,
            cache,
            hits,
            left,
            right,
            y,
            i + 2,
            piece.as_ref(),
            "No piece",
        );
        if i + 1 < data.armour.len() {
            hair(p, left, right, y - 3.0);
        }
    }
    y += 14.0;
    y = section(p, left, right, y, "CHARM");
    y = row(
        p,
        cache,
        hits,
        left,
        right,
        y,
        7,
        data.charm.as_ref(),
        "No charm",
    );
    if !side {
        skills(
            p,
            hits,
            left,
            right,
            y + 14.0,
            content.max.y - 16.0,
            &data.skills,
            set_style,
        );
    }
}

/// The skills strip: a section label with its hairline, then the set and group skills pinned at
/// the top (two-line blocks with a small kind tag), then the normal skills. Each normal skill is a
/// name, a level number at the right and one round dot per possible level. Columns follow the
/// width; dots and numbers sit on the same x in every block and are snapped to whole pixels.
///
/// A long list changes density instead of scrolling. Normal skills are two-line blocks while they
/// fit above `bottom`, then one-line rows with the dots in a fixed column. If they still do not
/// fit, the least important are left out and a "+N more on Why" line says where the rest are.
/// Must-haves and set and group skills are never left out; the rest go by the optimizer's weight
/// when there is one, otherwise lowest level first.
#[allow(clippy::too_many_arguments)]
fn skills(
    p: &Painter,
    hits: &mut Vec<(Rect, Hit)>,
    left: f32,
    right: f32,
    y: f32,
    bottom: f32,
    list: &[Skill],
    set_style: SetStyle,
) {
    let mut y = section(p, left, right, y, "SKILLS");
    if list.is_empty() {
        p.text(
            Pos2::new(left, y + 12.0),
            Align2::LEFT_CENTER,
            "No skills yet",
            font("inter", 13.0),
            FAINT,
        );
        return;
    }
    let ppp = p.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let cols = (((right - left) / SKILL_COL_MIN_W).floor() as usize).max(1);
    let col_w = (right - left - (cols - 1) as f32 * 24.0) / cols as f32;
    let col_x = |i: usize| left + (i % cols) as f32 * (col_w + 24.0);
    y += 6.0;

    // Set and group skills first, in two-line blocks.
    let pinned: Vec<&Skill> = list.iter().filter(|k| k.pinned()).collect();
    for (i, k) in pinned.iter().enumerate() {
        let x = col_x(i);
        let ry = y + (i / cols) as f32 * SKILL_ROW;
        p.text(
            Pos2::new(x, ry + 9.0),
            Align2::LEFT_CENTER,
            fit(p, k.name, col_w - 56.0, "inter-medium"),
            font("inter-medium", 12.5),
            FG,
        );
        // The kind tag: a small outlined mono label at the right, so the block reads as a
        // different kind of thing at a glance without using the accent.
        let tag = if k.kind == SkillKind::Set {
            "SET"
        } else {
            "GROUP"
        };
        let g = p.layout_no_wrap(tag.to_string(), mono(9.0), DIM);
        let tag_rect = Rect::from_center_size(
            Pos2::new(snap(x + col_w - (g.size().x + 10.0) / 2.0), snap(ry + 9.0)),
            Vec2::new(g.size().x + 10.0, 15.0),
        );
        p.rect_stroke(
            tag_rect,
            2.0,
            Stroke::new(1.0, FAINT),
            egui::StrokeKind::Inside,
        );
        p.galley(tag_rect.center() - g.size() / 2.0, g, DIM);

        match set_style {
            SetStyle::Pips => {
                // One numbered circle per threshold, the slot-circle size and look, filled once
                // reached. Pieces worn at the right.
                for (j, step) in k.steps.iter().enumerate() {
                    let c = Pos2::new(snap(x + SLOT_R + j as f32 * SLOT_PITCH), snap(ry + 26.0));
                    let ink = if k.level >= *step {
                        p.circle_filled(c, SLOT_R, ACCENT);
                        ACCENT_INK
                    } else {
                        p.circle_stroke(c, SLOT_R - 1.0, Stroke::new(1.0, FAINT));
                        DIM
                    };
                    p.text(c, Align2::CENTER_CENTER, step.to_string(), mono(9.0), ink);
                }
                p.text(
                    Pos2::new(x + col_w, ry + 26.0),
                    Align2::RIGHT_CENTER,
                    format!("{} pieces", k.level),
                    mono(11.0),
                    DIM,
                );
            }
            SetStyle::Text => {
                let done = k.steps.iter().any(|s| k.level >= *s);
                p.text(
                    Pos2::new(x, ry + 26.0),
                    Align2::LEFT_CENTER,
                    format!("{} of {} pieces", k.level, k.max),
                    font("inter", 11.5),
                    if done { OK } else { DIM },
                );
            }
        }
        hits.push((
            Rect::from_min_size(Pos2::new(x, ry), Vec2::new(col_w, SKILL_ROW - 4.0)),
            Hit::Skill(**k),
        ));
    }
    if !pinned.is_empty() {
        y += pinned.len().div_ceil(cols) as f32 * SKILL_ROW;
    }

    let normal: Vec<&Skill> = list.iter().filter(|k| !k.pinned()).collect();
    if normal.is_empty() {
        return;
    }
    if !pinned.is_empty() {
        hair(p, left, right, y + 2.0);
        y += 12.0;
    }

    // Density follows the count.
    let avail = (bottom - y).max(0.0);
    let per_col = normal.len().div_ceil(cols);
    let comfortable = per_col as f32 * SKILL_ROW <= avail;
    let pitch = if comfortable { SKILL_ROW } else { COMPACT_ROW };
    let fit_rows = ((avail / pitch).floor() as usize).max(1);
    let n_cut = if per_col <= fit_rows {
        0
    } else {
        normal.len() - (fit_rows.saturating_sub(1) * cols).max(1).min(normal.len())
    };
    let hidden = hide_least_important(&normal, n_cut);
    let cut = hidden.iter().filter(|h| **h).count();

    let visible = normal
        .iter()
        .zip(&hidden)
        .filter(|(_, h)| !**h)
        .map(|(k, _)| k);
    for (i, k) in visible.enumerate() {
        let x = col_x(i);
        let ry = y + (i / cols) as f32 * pitch;
        // Name left, level number at the right edge; the name gives way with an ellipsis.
        let (name_y, dots_x, dots_y, name_w) = if comfortable {
            (ry + 9.0, x, ry + 26.0, col_w - 24.0)
        } else {
            let mid = ry + COMPACT_ROW / 2.0 - 2.0;
            (mid, x + COMPACT_DOTS_X, mid, COMPACT_DOTS_X - 8.0)
        };
        p.text(
            Pos2::new(x + col_w, name_y),
            Align2::RIGHT_CENTER,
            k.level.to_string(),
            mono(11.5),
            DIM,
        );
        p.text(
            Pos2::new(x, name_y),
            Align2::LEFT_CENTER,
            fit(p, k.name, name_w, "inter"),
            font("inter", 12.5),
            FG,
        );
        for d in 0..k.max {
            let c = Pos2::new(
                snap(dots_x + DOT_R + f32::from(d) * DOT_PITCH),
                snap(dots_y),
            );
            if d < k.level {
                p.circle_filled(c, DOT_R, ACCENT);
            } else {
                // Same outer diameter as the filled dot: the outline sits inside the radius.
                p.circle_stroke(c, DOT_R - 0.5, Stroke::new(1.0, FAINT));
            }
        }
        hits.push((
            Rect::from_min_size(Pos2::new(x, ry), Vec2::new(col_w, pitch - 2.0)),
            Hit::Skill(**k),
        ));
    }
    if cut > 0 {
        let shown = normal.len() - cut;
        let ry = y + shown.div_ceil(cols) as f32 * pitch;
        let g = p.layout_no_wrap(format!("+{cut} more on Why"), mono(11.0), DIM);
        let rect = Rect::from_min_size(Pos2::new(left, ry + 2.0), Vec2::new(g.size().x, 18.0));
        p.galley(Pos2::new(left, ry + 11.0 - g.size().y / 2.0), g, DIM);
        hits.push((rect, Hit::Why));
    }
}

/// Marks `n_cut` of the normal skills to leave out: the least important first (by weight when
/// there is one, else lowest level). Must-haves are never marked, so fewer may be.
fn hide_least_important(normal: &[&Skill], n_cut: usize) -> Vec<bool> {
    let mut by_rank: Vec<usize> = (0..normal.len()).collect();
    by_rank.sort_by_key(|&i| normal[i].keep_rank());
    let mut hidden = vec![false; normal.len()];
    for &i in by_rank.iter().take(n_cut) {
        if normal[i].keep_rank().0 < 2 {
            hidden[i] = true;
        }
    }
    hidden
}

/// `text` shortened with an ellipsis until it fits `width` (12.5 pt, in the named face).
fn fit(p: &Painter, text: &str, width: f32, face: &str) -> String {
    let fits = |t: &str| {
        p.layout_no_wrap(t.to_string(), font(face, 12.5), FG)
            .size()
            .x
            <= width
    };
    if fits(text) {
        return text.to_string();
    }
    let mut t: String = text.to_string();
    while !t.is_empty() {
        t.pop();
        let cut = format!("{}\u{2026}", t.trim_end());
        if fits(&cut) {
            return cut;
        }
    }
    String::new()
}

fn hair(p: &Painter, left: f32, right: f32, y: f32) {
    p.line_segment(
        [Pos2::new(left, y), Pos2::new(right, y)],
        Stroke::new(1.0, LINE),
    );
}

/// A mono caps section label with its hairline; returns the y of the first row under it.
fn section(p: &Painter, left: f32, right: f32, y: f32, label: &str) -> f32 {
    p.text(
        Pos2::new(left, y + 8.0),
        Align2::LEFT_CENTER,
        label,
        mono(10.5),
        DIM,
    );
    hair(p, left, right, y + 22.0);
    y + 30.0
}

/// One equipment row. Returns the y for the next row.
#[allow(clippy::too_many_arguments)]
fn row(
    p: &Painter,
    cache: &mut IconCache,
    hits: &mut Vec<(Rect, Hit)>,
    left: f32,
    right: f32,
    y: f32,
    n: usize,
    piece: Option<&Piece>,
    empty: &str,
) -> f32 {
    let cy = y + ROW_H / 2.0;
    p.text(
        Pos2::new(left, cy),
        Align2::LEFT_CENTER,
        format!("{n:02}"),
        mono(11.0),
        FAINT,
    );
    let tile = Rect::from_center_size(Pos2::new(left + 46.0 + TILE / 2.0, cy), Vec2::splat(TILE));
    let text_x = tile.right() + 14.0;
    let Some(piece) = piece else {
        // Empty slot: the tile outline and a faint word, so the card keeps its shape.
        p.rect_stroke(tile, 2.0, Stroke::new(1.0, LINE), egui::StrokeKind::Inside);
        p.text(
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            empty,
            font("inter", 13.0),
            FAINT,
        );
        return y + ROW_STEP;
    };
    cache.tile(p, tile, piece.icon_id, piece.glyph);

    // Two fixed columns at the same x on every row: the status word (left-aligned) at the right
    // edge, and three slot positions to its left, filled from the left. Empty positions stay blank.
    let (txt, col) = if piece.missing == 0 {
        ("Owned".to_string(), OK)
    } else {
        (format!("Missing {}", piece.missing), WARN)
    };
    p.text(
        Pos2::new(right - STATUS_W, cy),
        Align2::LEFT_CENTER,
        txt,
        font("inter-medium", 12.0),
        col,
    );
    let ppp = p.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let slots_left = right - STATUS_W - 28.0 - SLOT_COLS as f32 * SLOT_PITCH + SLOT_GAP;
    for (i, slot) in piece.slots.iter().take(SLOT_COLS).enumerate() {
        let c = Pos2::new(snap(slots_left + SLOT_R + i as f32 * SLOT_PITCH), snap(cy));
        // Filled and empty share one outer diameter: the outline is drawn inside the radius.
        let ink = if slot.filled {
            p.circle_filled(c, SLOT_R, DIM);
            BG
        } else {
            p.circle_stroke(c, SLOT_R - 1.0, Stroke::new(1.0, FAINT));
            DIM
        };
        p.text(
            c,
            Align2::CENTER_CENTER,
            slot.level.to_string(),
            mono(9.0),
            ink,
        );
        hits.push((
            Rect::from_center_size(c, Vec2::splat(SLOT_R * 2.0)),
            Hit::Slot(slot.level),
        ));
    }
    let x = slots_left;

    p.text(
        Pos2::new(text_x, cy - 9.0),
        Align2::LEFT_CENTER,
        piece.name,
        font("inter-medium", 13.5),
        FG,
    );
    // The detail line gives way to the slots instead of running under them.
    let detail = p.layout_no_wrap(piece.detail.to_string(), font("inter", 11.5), DIM);
    let avail = (x - 16.0 - text_x).max(0.0);
    if detail.size().x <= avail {
        p.galley(
            Pos2::new(text_x, cy + 8.0 - detail.size().y / 2.0),
            detail,
            DIM,
        );
    }
    y + ROW_STEP
}

/// Renders the card with no window or GPU. Keeps its context and rasteriser between calls,
/// because the icon textures live in them.
pub struct Headless {
    inner: super::icons::Headless,
}

impl Headless {
    pub fn new() -> Self {
        Headless {
            inner: super::icons::Headless::new(),
        }
    }

    pub fn ctx(&self) -> &Context {
        &self.inner.ctx
    }

    pub fn frame(&mut self, app: &mut BuildCard, size: Vec2, ppp: f32) -> render::Frame {
        self.inner.run(size, ppp, |ui| show(ui, app))
    }
}

impl Default for Headless {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[Skill], hidden: &[bool]) -> Vec<&'static str> {
        list.iter()
            .zip(hidden)
            .filter(|(_, h)| **h)
            .map(|(k, _)| k.name)
            .collect()
    }

    #[test]
    fn cuts_lowest_weight_first_and_never_must_haves() {
        let w = |name, level, weight| Skill {
            weight: Some(weight),
            ..Skill::normal(name, level, 5)
        };
        let list = [
            Skill {
                must: true,
                ..w("Must", 1, 1)
            },
            w("High", 5, 90),
            w("Low", 5, 10),
            w("Mid", 1, 50),
        ];
        let refs: Vec<&Skill> = list.iter().collect();
        // The must-have has the lowest weight and level but stays.
        assert_eq!(names(&list, &hide_least_important(&refs, 1)), ["Low"]);
        assert_eq!(
            names(&list, &hide_least_important(&refs, 2)),
            ["Low", "Mid"]
        );
        assert_eq!(
            names(&list, &hide_least_important(&refs, 4)),
            ["High", "Low", "Mid"]
        );
    }

    #[test]
    fn without_weights_cuts_lowest_level_first() {
        let list = [
            Skill::normal("Three", 3, 5),
            Skill::normal("One", 1, 5),
            Skill::normal("Two", 2, 5),
        ];
        let refs: Vec<&Skill> = list.iter().collect();
        assert_eq!(
            names(&list, &hide_least_important(&refs, 2)),
            ["One", "Two"]
        );
        // A skill with a weight outranks one without.
        let mixed = [
            Skill::normal("Unweighted", 5, 5),
            Skill {
                weight: Some(1),
                ..Skill::normal("Weighted", 1, 5)
            },
        ];
        let refs: Vec<&Skill> = mixed.iter().collect();
        assert_eq!(
            names(&mixed, &hide_least_important(&refs, 1)),
            ["Unweighted"]
        );
    }
}
