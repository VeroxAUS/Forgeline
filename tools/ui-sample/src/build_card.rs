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
/// One tier line under a set or group skill's name.
const TIER_ROW: f32 = 16.0;
const COMPACT_DOTS_X: f32 = 120.0;
/// Width of the side strip, and the gap between it and the equipment rows.
const SKILLS_W: f32 = 232.0;
const SKILLS_GAP: f32 = 32.0;
/// Gap between strip columns.
const SKILL_GAP: f32 = 24.0;
/// The equipment rows keep at least this width; below it the strip drops under the card.
const EQUIP_MIN_W: f32 = 480.0;
/// Each strip column is at least this wide (name, dots and level number).
const SKILL_COL_MIN_W: f32 = 200.0;
/// Headline block: its width is one skills column, so the toughness block sits over the skills
/// strip and the damage block ends where the equipment rows end. Its top is under the content
/// area's top edge.
const HEAD_W: f32 = SKILLS_W;
const HEAD_TOP: f32 = 40.0;
const HEAD_BAR_H: f32 = 6.0;
const ACCENT: Color32 = Color32::from_rgb(255, 138, 61);
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

/// How tall set and group skills are drawn in the strip (the options compared for the long-list
/// question).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PinnedStyle {
    /// The tallest of the three below that still lets every skill show; the one-row style if
    /// none does. Short lists get the full tiers, long ones collapse.
    #[default]
    Auto,
    /// Name, then one line per tier.
    Tiers,
    /// Name, then one line: the highest active tier and what is next.
    Summary,
    /// One row like a normal skill: name, tag and pieces worn. Tiers on hover and on Why.
    OneLine,
}

/// One tier of a set or group skill: the effect that starts at `pieces` worn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tier {
    pub pieces: u8,
    /// Short, for the strip.
    pub effect: &'static str,
    /// The full effect text, for the tooltip and the Why screen.
    pub full: &'static str,
}

/// One skill in the build. For a normal skill `level` is its total level and `max` the most it
/// can reach. For a set or group skill `level` is the pieces worn, `max` the highest tier and
/// `tiers` the effects, lowest piece count first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Skill {
    pub name: &'static str,
    pub level: u8,
    pub max: u8,
    pub kind: SkillKind,
    pub tiers: &'static [Tier],
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
            tiers: &[],
            must: false,
            weight: None,
        }
    }

    /// A set or group skill: `pieces` worn, an effect at each of `tiers`.
    pub fn bonus(kind: SkillKind, name: &'static str, pieces: u8, tiers: &'static [Tier]) -> Self {
        Skill {
            name,
            level: pieces,
            max: tiers.last().map_or(0, |t| t.pieces),
            kind,
            tiers,
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

/// Which of the two headline axes a block shows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    Damage,
    Toughness,
}

/// How the headline blocks are drawn (the options compared for open item A in the design plan).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum HeadlineStyle {
    /// A plain word and a bar. Nothing else.
    Word,
    /// The word, the bar with a tick for the current gear, and the change against it.
    #[default]
    Compare,
    /// The word, the bar and the key number underneath.
    Number,
}

/// One headline block. `fill` is the build's place on the bar (0 to 1, as a share of the best the
/// unlocked pool allows); `None` means unknown. `current` is where the user's current gear sits on
/// the same bar, `delta` the change against it in percent.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Headline {
    pub axis: Axis,
    pub fill: Option<f32>,
    pub current: Option<f32>,
    pub delta: Option<i32>,
    /// The key number with its unit, for raw detail. Sample text here.
    pub number: &'static str,
    /// Why the value is unknown, when `fill` is `None`.
    pub unknown_why: &'static str,
}

impl Headline {
    /// The headline word. Against current gear when there is a comparison, otherwise by place on
    /// the bar. The thresholds are provisional.
    pub fn word(&self) -> &'static str {
        let Some(fill) = self.fill else {
            return "UNKNOWN";
        };
        match (self.axis, self.delta) {
            (Axis::Damage, Some(d)) => match d {
                20.. => "MUCH HIGHER",
                5..=19 => "HIGHER",
                -4..=4 => "SIMILAR",
                _ => "LOWER",
            },
            (Axis::Toughness, Some(d)) => match d {
                20.. => "MUCH TANKIER",
                5..=19 => "TANKIER",
                -4..=4 => "SOLID",
                _ => "SQUISHIER",
            },
            (Axis::Damage, None) => match fill {
                f if f >= 0.85 => "VERY HIGH",
                f if f >= 0.6 => "HIGH",
                f if f >= 0.35 => "MODERATE",
                _ => "LOW",
            },
            (Axis::Toughness, None) => match fill {
                f if f >= 0.85 => "VERY TOUGH",
                f if f >= 0.6 => "TOUGH",
                f if f >= 0.35 => "MODERATE",
                _ => "FRAGILE",
            },
        }
    }

    fn label(&self) -> &'static str {
        match self.axis {
            Axis::Damage => "DAMAGE",
            Axis::Toughness => "TOUGHNESS",
        }
    }

    fn tip(&self) -> &'static str {
        match self.axis {
            Axis::Damage => "Damage is modelled from assumed uptime and weak-spot rate. Open Why for the numbers.",
            Axis::Toughness => "Toughness is modelled from defence, resistances and health skills. Open Why for the numbers.",
        }
    }
}

/// The card's data: the weapon, up to five armour pieces and the charm. `None` is an empty slot.
#[derive(Clone, PartialEq, Debug)]
pub struct BuildData {
    pub weapon: Option<Piece>,
    pub armour: [Option<Piece>; 5],
    pub charm: Option<Piece>,
    /// Skill totals for the whole build, in the order to show them.
    pub skills: Vec<Skill>,
    /// How the set and group skills are drawn.
    pub pinned_style: PinnedStyle,
    pub damage: Headline,
    pub toughness: Headline,
    pub headline_style: HeadlineStyle,
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
                Skill::bonus(SkillKind::Set, "Set bonus Alpha", 3, TWO_FOUR),
                Skill::bonus(SkillKind::Group, "Group skill Gamma", 3, THREE),
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
            pinned_style: PinnedStyle::default(),
            damage: Headline {
                axis: Axis::Damage,
                fill: Some(0.72),
                current: Some(0.58),
                delta: Some(24),
                number: "4,820 dmg/min",
                unknown_why: "Needs a weapon",
            },
            toughness: Headline {
                axis: Axis::Toughness,
                fill: Some(0.64),
                current: Some(0.61),
                delta: Some(4),
                number: "1,180 effective HP",
                unknown_why: "Needs armour",
            },
            headline_style: HeadlineStyle::default(),
        }
    }
}

/// Invented tier text. Whether the game files hold set and group skill tier effects is unverified,
/// so the strip is shown with sample text.
const TWO_FOUR: &[Tier] = &[
    Tier {
        pieces: 2,
        effect: "Sample effect I",
        full: "Sample text: the 2-piece effect of this set, in full.",
    },
    Tier {
        pieces: 4,
        effect: "Sample effect II",
        full: "Sample text: the 4-piece effect of this set, in full.",
    },
];
const THREE: &[Tier] = &[Tier {
    pieces: 3,
    effect: "Sample group effect",
    full: "Sample text: the 3-piece effect of this group skill, in full.",
}];

/// A realistic full endgame build's skills (invented names): 22 skills, with set and group
/// skills first and the rest most important first, each with a sample weight.
pub fn sample_endgame_skills() -> Vec<Skill> {
    let mut v = vec![
        Skill::bonus(SkillKind::Set, "Set bonus Alpha", 4, TWO_FOUR),
        Skill::bonus(SkillKind::Set, "Set bonus Beta", 2, TWO_FOUR),
        Skill::bonus(SkillKind::Group, "Group skill Gamma", 3, THREE),
        Skill::bonus(SkillKind::Group, "Group skill Delta", 2, THREE),
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

/// A build shaped like the real endgame builds found for the research (counts only, invented
/// names): `normal` normal skills, then `sets` set skills and `groups` group skills, each with
/// tiers, some active.
pub fn sample_shaped_skills(normal: usize, sets: usize, groups: usize) -> Vec<Skill> {
    const NAMES: [&str; 24] = [
        "Skill A", "Skill B", "Skill C", "Skill D", "Skill E", "Skill F", "Skill G", "Skill H",
        "Skill I", "Skill J", "Skill K", "Skill L", "Skill M", "Skill N", "Skill O", "Skill P",
        "Skill Q", "Skill R", "Skill S", "Skill T", "Skill U", "Skill V", "Skill W", "Skill X",
    ];
    const SETS: [&str; 3] = ["Set bonus Alpha", "Set bonus Beta", "Set bonus Gamma"];
    const GROUPS: [&str; 3] = [
        "Group skill Delta",
        "Group skill Epsilon",
        "Group skill Zeta",
    ];
    let mut v = Vec::new();
    for (i, name) in SETS.iter().take(sets).enumerate() {
        v.push(Skill::bonus(SkillKind::Set, name, [4, 2, 2][i], TWO_FOUR));
    }
    for (i, name) in GROUPS.iter().take(groups).enumerate() {
        v.push(Skill::bonus(SkillKind::Group, name, [3, 2, 3][i], THREE));
    }
    for (i, name) in NAMES.iter().take(normal).enumerate() {
        v.push(Skill {
            weight: Some(100 - 4 * i as u8),
            ..Skill::normal(name, [5, 5, 3, 3, 3, 2, 2, 3][i % 8], 5)
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
    /// A headline block's modelled-value note.
    Tip(&'static str),
}

/// The build card screen: shell state, the card's data and the icon cache.
pub struct BuildCard {
    pub shell: Shell,
    pub data: BuildData,
    pub cache: IconCache,
    /// Slot circles drawn last frame, for the hover tooltip.
    hits: Vec<(Rect, Hit)>,
}

impl BuildCard {
    /// How many skills the strip drew last frame (the rest are behind "+N more on Why").
    pub fn skills_shown(&self) -> usize {
        self.hits
            .iter()
            .filter(|(_, h)| matches!(h, Hit::Skill(_)))
            .count()
    }

    pub fn new(ctx: &Context, source: Arc<dyn IconSource>) -> Self {
        BuildCard {
            shell: Shell::new(),
            data: BuildData::sample(),
            cache: IconCache::new(ctx, source),
            hits: Vec::new(),
        }
    }
}

/// Draws the shell with the build card in its content area (Build section only).
pub fn show(ui: &mut Ui, app: &mut BuildCard) {
    app.cache.poll();
    let data = &app.data;
    let cache = &mut app.cache;
    let hits = &mut app.hits;
    hits.clear();
    shell::show_with(ui, &mut app.shell, &mut |p, content, shell| {
        shell::show_content(p, content, shell);
        if shell.section == shell::Section::Build {
            card(p, content, data, cache, hits, &shell.status);
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
            Hit::Tip(text) => {
                tip.on_hover_text(*text);
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
            let mut tip = format!("{}, {} pieces worn", k.name, k.level);
            for t in k.tiers {
                tip += &format!("\n{} pieces: {}", t.pieces, t.full);
            }
            tip
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
    // In a wider window (a 16:9 screen is about 1245 points wide at the 700 point design height)
    // the strip gets a second column and the equipment rows keep their width.
    let strip_w = if right - left - 2.0 * SKILLS_W - SKILL_GAP - SKILLS_GAP >= EQUIP_MIN_W {
        2.0 * SKILLS_W + SKILL_GAP
    } else {
        SKILLS_W
    };
    let side = right - left - strip_w - SKILLS_GAP >= EQUIP_MIN_W;
    // The two headline blocks sit on the same columns as the rows and the skills strip: toughness
    // over the last skills column, damage over the one before it (or ending at the equipment rows).
    let gap = if side && strip_w > SKILLS_W {
        SKILL_GAP
    } else {
        SKILLS_GAP
    };
    let by = content.min.y + HEAD_TOP;
    let tx = right - HEAD_W;
    headline(
        p,
        hits,
        tx - gap - HEAD_W,
        by,
        &data.damage,
        data.headline_style,
    );
    headline(p, hits, tx, by, &data.toughness, data.headline_style);
    let top = content.min.y + 120.0;
    let eq_right = if side {
        right - strip_w - SKILLS_GAP
    } else {
        right
    };
    if side {
        skills(
            p,
            hits,
            right - strip_w,
            right,
            top,
            content.max.y - 16.0,
            &data.skills,
            data.pinned_style,
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
            data.pinned_style,
        );
    }
}

/// One headline block, `HEAD_W` wide, four fixed rows so both blocks line up: label with the
/// `MODELLED` tag, the word, the bar, and a detail line (change or number, depending on style).
/// The bar fill and the tick snap to whole pixels.
fn headline(
    p: &Painter,
    hits: &mut Vec<(Rect, Hit)>,
    x: f32,
    y: f32,
    h: &Headline,
    style: HeadlineStyle,
) {
    let ppp = p.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let right = x + HEAD_W;
    p.text(
        Pos2::new(x, y + 6.0),
        Align2::LEFT_CENTER,
        h.label(),
        mono(10.5),
        DIM,
    );
    if h.fill.is_some() {
        p.text(
            Pos2::new(right, y + 6.0),
            Align2::RIGHT_CENTER,
            "MODELLED",
            mono(10.5),
            WARN,
        );
    }
    let known = h.fill.is_some();
    let word = fit(p, h.word(), HEAD_W, "mono", 20.0);
    p.text(
        Pos2::new(x, y + 27.0),
        Align2::LEFT_CENTER,
        word,
        mono(20.0),
        if known { FG } else { FAINT },
    );
    let bar = Rect::from_min_size(Pos2::new(x, snap(y + 44.0)), Vec2::new(HEAD_W, HEAD_BAR_H));
    p.rect_filled(bar, HEAD_BAR_H / 2.0, LINE);
    if let Some(fill) = h.fill {
        let w = snap((HEAD_W * fill.clamp(0.0, 1.0)).max(HEAD_BAR_H));
        p.rect_filled(
            Rect::from_min_size(bar.min, Vec2::new(w, HEAD_BAR_H)),
            HEAD_BAR_H / 2.0,
            ACCENT,
        );
    }
    let detail_y = y + 62.0;
    let detail = |text: String, color: Color32| {
        p.text(
            Pos2::new(x, detail_y),
            Align2::LEFT_CENTER,
            fit(p, &text, HEAD_W, "mono", 11.0),
            mono(11.0),
            color,
        );
    };
    if !known {
        detail(h.unknown_why.to_string(), DIM);
    } else {
        match style {
            HeadlineStyle::Word => {}
            HeadlineStyle::Compare => {
                if let Some(c) = h.current {
                    let tx = snap(x + HEAD_W * c.clamp(0.0, 1.0));
                    p.rect_filled(
                        Rect::from_min_max(
                            Pos2::new(tx - 0.75, bar.top() - 3.0),
                            Pos2::new(tx + 0.75, bar.bottom() + 3.0),
                        ),
                        0.0,
                        FG,
                    );
                }
                match h.delta {
                    Some(0) => detail("Same as your gear".into(), DIM),
                    Some(d) if d > 0 => detail(format!("+{d}% vs your gear"), OK),
                    Some(d) => detail(format!("{d}% vs your gear"), WARN),
                    None => detail("Nothing worn to compare".into(), FAINT),
                }
            }
            HeadlineStyle::Number => detail(h.number.to_string(), DIM),
        }
    }
    hits.push((
        Rect::from_min_max(Pos2::new(x, y), Pos2::new(right, detail_y + 8.0)),
        Hit::Tip(h.tip()),
    ));
}

/// The skills strip: a section label with its hairline, then the set and group skills pinned at
/// the top (a name with a small kind tag, then one line per tier), then the normal skills. Each normal skill is a
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
    pinned_style: PinnedStyle,
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
    let col_w = (right - left - (cols - 1) as f32 * SKILL_GAP) / cols as f32;
    let col_x = |i: usize| left + (i % cols) as f32 * (col_w + SKILL_GAP);
    y += 6.0;

    // Set and group skills first: the name and kind tag, then one line per tier. The active tiers
    // are green, the next one is muted with how many pieces it still needs, later ones fainter.
    // Blocks are as tall as their tiers; columns follow the width like the normal skills.
    let pinned: Vec<&Skill> = list.iter().filter(|k| k.pinned()).collect();
    let pinned_style = if pinned_style == PinnedStyle::Auto {
        let n_normal = list.len() - pinned.len();
        [PinnedStyle::Tiers, PinnedStyle::Summary]
            .into_iter()
            .find(|st| everything_fits(&pinned, n_normal, *st, cols, y, bottom))
            .unwrap_or(PinnedStyle::OneLine)
    } else {
        pinned_style
    };
    let mut col_y = vec![y; cols];
    let one_line = pinned_style == PinnedStyle::OneLine;
    for (i, k) in pinned.iter().enumerate() {
        let x = col_x(i);
        let ry = col_y[i % cols];
        // Name, with the kind tag: an outlined mono label, so the block reads as a different kind
        // of thing at a glance without using the accent. In a one-line row the tag sits in the
        // same column as the dots of a normal compact row.
        let cy = if one_line {
            ry + COMPACT_ROW / 2.0 - 2.0
        } else {
            ry + 9.0
        };
        let name_w = if one_line {
            COMPACT_DOTS_X - 8.0
        } else {
            col_w - 56.0
        };
        p.text(
            Pos2::new(x, cy),
            Align2::LEFT_CENTER,
            fit(p, k.name, name_w, "inter-medium", 12.5),
            font("inter-medium", 12.5),
            FG,
        );
        let tag = if k.kind == SkillKind::Set {
            "SET"
        } else {
            "GROUP"
        };
        let g = p.layout_no_wrap(tag.to_string(), mono(9.0), DIM);
        let tag_w = g.size().x + 10.0;
        let tag_cx = if one_line {
            x + COMPACT_DOTS_X + tag_w / 2.0
        } else {
            x + col_w - tag_w / 2.0
        };
        let tag_rect =
            Rect::from_center_size(Pos2::new(snap(tag_cx), snap(cy)), Vec2::new(tag_w, 15.0));
        p.rect_stroke(
            tag_rect,
            2.0,
            Stroke::new(1.0, FAINT),
            egui::StrokeKind::Inside,
        );
        p.galley(tag_rect.center() - g.size() / 2.0, g, DIM);

        let mut next_seen = false;
        let lines: Vec<(String, Color32)> = k
            .tiers
            .iter()
            .map(|t| tier_text(k, t, &mut next_seen))
            .collect();
        let h = match pinned_style {
            PinnedStyle::Tiers | PinnedStyle::Auto => {
                for (j, (text, color)) in lines.iter().enumerate() {
                    tier_line(p, x, ry + 26.0 + j as f32 * TIER_ROW, col_w, text, *color);
                }
                20.0 + lines.len().max(1) as f32 * TIER_ROW + 6.0
            }
            PinnedStyle::Summary => {
                // The highest tier reached, then the next one after it.
                let reached = k.tiers.iter().filter(|t| k.level >= t.pieces).count();
                let text = match (reached.checked_sub(1), k.tiers.get(reached)) {
                    (Some(a), Some(n)) => format!("{} \u{b7} next {} pcs", lines[a].0, n.pieces),
                    (Some(a), None) => lines[a].0.clone(),
                    (None, _) => lines.first().map(|l| l.0.clone()).unwrap_or_default(),
                };
                let color = if reached > 0 { OK } else { DIM };
                tier_line(p, x, ry + 26.0, col_w, &text, color);
                42.0
            }
            PinnedStyle::OneLine => {
                let active = k.tiers.iter().any(|t| k.level >= t.pieces);
                p.text(
                    Pos2::new(x + col_w, cy),
                    Align2::RIGHT_CENTER,
                    format!("{} pcs", k.level),
                    mono(11.5),
                    if active { OK } else { DIM },
                );
                COMPACT_ROW
            }
        };
        hits.push((
            Rect::from_min_size(Pos2::new(x, ry), Vec2::new(col_w, h - 2.0)),
            Hit::Skill(**k),
        ));
        col_y[i % cols] += h;
    }
    if !pinned.is_empty() {
        y = col_y.iter().cloned().fold(y, f32::max);
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
            fit(p, k.name, name_w, "inter", 12.5),
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

/// Height of one set or group skill block in the given style.
fn pinned_height(k: &Skill, style: PinnedStyle) -> f32 {
    match style {
        PinnedStyle::Tiers | PinnedStyle::Auto => {
            20.0 + k.tiers.len().max(1) as f32 * TIER_ROW + 6.0
        }
        PinnedStyle::Summary => 42.0,
        PinnedStyle::OneLine => COMPACT_ROW,
    }
}

/// Whether the pinned blocks in `style` and `n_normal` compact normal rows fit between `y` and
/// `bottom`, so nothing has to be left out.
fn everything_fits(
    pinned: &[&Skill],
    n_normal: usize,
    style: PinnedStyle,
    cols: usize,
    y: f32,
    bottom: f32,
) -> bool {
    let mut col_h = vec![0.0f32; cols];
    for (i, k) in pinned.iter().enumerate() {
        col_h[i % cols] += pinned_height(k, style);
    }
    let pinned_h = col_h.into_iter().fold(0.0, f32::max);
    let gap = if pinned.is_empty() || n_normal == 0 {
        0.0
    } else {
        12.0
    };
    y + pinned_h + gap + n_normal.div_ceil(cols) as f32 * COMPACT_ROW <= bottom
}

/// The line for one tier and its colour: reached tiers green, the first one not reached muted with
/// how many pieces it still needs, later ones fainter. `next_seen` carries across a skill's tiers.
fn tier_text(k: &Skill, t: &Tier, next_seen: &mut bool) -> (String, Color32) {
    if k.level >= t.pieces {
        (format!("{} pcs: {}", t.pieces, t.effect), OK)
    } else if !*next_seen {
        *next_seen = true;
        let n = t.pieces - k.level;
        let more = if n == 1 {
            "1 more piece".to_string()
        } else {
            format!("{n} more pieces")
        };
        (format!("{} pcs: {}, {more}", t.pieces, t.effect), DIM)
    } else {
        (format!("{} pcs: {}", t.pieces, t.effect), FAINT)
    }
}

fn tier_line(p: &Painter, x: f32, cy: f32, width: f32, text: &str, color: Color32) {
    p.text(
        Pos2::new(x, cy),
        Align2::LEFT_CENTER,
        fit(p, text, width, "inter", 11.5),
        font("inter", 11.5),
        color,
    );
}

/// `text` shortened with an ellipsis until it fits `width` (in the named face and size).
fn fit(p: &Painter, text: &str, width: f32, face: &str, size: f32) -> String {
    let fits = |t: &str| {
        p.layout_no_wrap(t.to_string(), font(face, size), FG)
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

    fn line(axis: Axis, fill: Option<f32>, delta: Option<i32>) -> Headline {
        Headline {
            axis,
            fill,
            current: None,
            delta,
            number: "",
            unknown_why: "",
        }
    }

    #[test]
    fn headline_words_follow_the_change_against_gear_or_the_bar() {
        let t = |d| line(Axis::Toughness, Some(0.5), d).word();
        assert_eq!(t(Some(40)), "MUCH TANKIER");
        assert_eq!(t(Some(5)), "TANKIER");
        assert_eq!(t(Some(4)), "SOLID");
        assert_eq!(t(Some(-4)), "SOLID");
        assert_eq!(t(Some(-5)), "SQUISHIER");
        assert_eq!(
            line(Axis::Damage, Some(0.5), Some(20)).word(),
            "MUCH HIGHER"
        );
        // No gear to compare: the place on the bar decides.
        assert_eq!(line(Axis::Damage, Some(0.9), None).word(), "VERY HIGH");
        assert_eq!(line(Axis::Toughness, Some(0.1), None).word(), "FRAGILE");
        // Unknown never gets a word that sounds like a result.
        assert_eq!(line(Axis::Damage, None, Some(30)).word(), "UNKNOWN");
    }
}
