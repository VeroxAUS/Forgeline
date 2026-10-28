// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 4 of docs/ux/design-plan.md: the build card, the weapon and armour slots with their
//! icons, drawn in the D3 style (docs/ux/style-guide.md) inside the element 2 shell and fed by the
//! element 3 icon cache.
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

/// The card's data: the weapon, up to five armour pieces and the charm. `None` is an empty slot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BuildData {
    pub weapon: Option<Piece>,
    pub armour: [Option<Piece>; 5],
    pub charm: Option<Piece>,
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
        }
    }
}

/// The build card screen: shell state, the card's data and the icon cache.
pub struct BuildCard {
    pub shell: Shell,
    pub data: BuildData,
    pub cache: IconCache,
    /// Slot circles drawn last frame, for the hover tooltip.
    hits: Vec<(Rect, u8)>,
}

impl BuildCard {
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
    for (i, (rect, level)) in app.hits.iter().enumerate() {
        ui.interact(*rect, ui.scope_id().with(("slot", i)), egui::Sense::hover())
            .on_hover_text(format!("Decoration slot, level {level}"));
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
    hits: &mut Vec<(Rect, u8)>,
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

    let mut y = content.min.y + 120.0;
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
    row(
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
    hits: &mut Vec<(Rect, u8)>,
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

    // Status word on the right, then the slot circles to its left.
    let (txt, col) = if piece.missing == 0 {
        ("Owned".to_string(), OK)
    } else {
        (format!("Missing {}", piece.missing), WARN)
    };
    let status = p.text(
        Pos2::new(right, cy),
        Align2::RIGHT_CENTER,
        txt,
        font("inter-medium", 12.0),
        col,
    );
    let mut x = status.left() - 28.0;
    for slot in piece.slots.iter().rev() {
        let c = Pos2::new(x - SLOT_R, cy);
        let ink = if slot.filled {
            p.circle_filled(c, SLOT_R, DIM);
            BG
        } else {
            p.circle_stroke(c, SLOT_R, Stroke::new(1.0, FAINT));
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
            slot.level,
        ));
        x -= SLOT_R * 2.0 + 6.0;
    }

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
