// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 10 of docs/ux/design-plan.md: the Shopping list. What the build still needs, in three
//! views: by monster, by piece and by upgrade path. One list on a fixed 36 point row grid, driven
//! by the save: counts come from what is owned, so there is nothing to tick off.
//!
//! Also owns the `Missing N` word on the Build equipment rows: hovering it lists the missing items
//! (item, count, monster); clicking it opens Shopping by piece, filtered to that piece.
//!
//! Where an item drops and which monster is best to farm it from are not read from the game files
//! yet (open question 12), so a farm place column exists but shows only when a need carries one.
//! The sample data has none by default, and the sample item and monster pairs are illustrative.

use super::*;

/// Every row of the list (group heads, step labels and items) is this high.
const ROW_H: f32 = 36.0;
const TILE_S: f32 = 28.0;
/// Where the name starts, counted from the list's left edge.
const NAME_X: f32 = TILE_S + 12.0;
/// The `owned/need` column and the context (monster or pieces) and farm columns.
const COUNT_W: f32 = 56.0;
const CTX_W: f32 = 168.0;
const FARM_W: f32 = 136.0;
const COL_GAP: f32 = 16.0;
/// The view switch: three equal halves, 30 high like the other header controls.
const VIEW_W: f32 = 96.0;
const VIEW_H: f32 = 30.0;
const VIEW_TOP: f32 = 78.0;
/// The filter chip beside the switch.
const CHIP_H: f32 = 24.0;
const FILTER_W: f32 = 200.0;
const LIST_TOP: f32 = 126.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum View {
    #[default]
    Monster,
    Piece,
    Path,
}

impl View {
    pub const ALL: [View; 3] = [View::Monster, View::Piece, View::Path];

    pub fn name(self) -> &'static str {
        match self {
            View::Monster => "Monster",
            View::Piece => "Piece",
            View::Path => "Upgrade",
        }
    }

    pub fn tip(self) -> &'static str {
        match self {
            View::Monster => "By monster: what to hunt next, and every item it gives.",
            View::Piece => "By piece: the items each piece of the build still needs.",
            View::Path => "By upgrade path: the items for each step of a piece's upgrade chain.",
        }
    }
}

/// One item a piece of the build still needs. `owned` is what the save holds, so the list changes
/// when the save does. Fully owned items are not listed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Need {
    /// Index of the piece: 0 weapon, 1 to 5 armour, 6 charm.
    pub piece: usize,
    pub item: &'static str,
    pub icon_id: u32,
    pub monster: &'static str,
    pub need: u16,
    pub owned: u16,
    /// Which step of the piece's upgrade chain this belongs to (0 when the piece has no chain).
    pub step: u8,
    /// Best place to farm it. Always `None` until drop data is confirmed (open question 12).
    pub farm: Option<&'static str>,
}

impl Need {
    pub fn missing(&self) -> u16 {
        self.need.saturating_sub(self.owned)
    }
}

/// A piece's upgrade chain: the names of the pieces it passes through, first to last.
pub type Chain = (usize, &'static [&'static str]);

/// What the Shopping screen shows and where it is looking.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct ShoppingState {
    pub view: View,
    /// Only items for this piece (set by clicking `Missing N` on the Build screen).
    pub filter: Option<usize>,
    /// Review only: draws an invented farm place on every row, to show the column filled.
    pub demo_farm: bool,
    /// Review only: scrolls the list to this offset (points) so a screenshot can show the middle.
    pub demo_scroll: Option<f32>,
}

impl ShoppingState {
    /// Opens Shopping by piece, filtered to one piece (the Build screen's `Missing N` click).
    pub fn show_piece(&mut self, piece: usize) {
        self.view = View::Piece;
        self.filter = Some(piece);
    }
}

/// One row of the list.
#[derive(Clone, PartialEq, Debug)]
enum Row {
    /// A group: a monster or a piece. No tile, so it never looks like an item.
    Head {
        title: String,
        tail: String,
        tip: String,
    },
    /// A step of an upgrade chain.
    Step(String),
    Item(ItemRow),
}

#[derive(Clone, PartialEq, Debug)]
struct ItemRow {
    name: &'static str,
    icon_id: u32,
    need: u16,
    owned: u16,
    ctx: String,
    farm: Option<&'static str>,
    tip: String,
    /// Draw a hairline under it (not under the last item of a group).
    line: bool,
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The rows for one view. Only items still missing are listed.
fn rows(data: &BuildData, st: &ShoppingState) -> Vec<Row> {
    let needs: Vec<&Need> = data
        .needs
        .iter()
        .filter(|n| n.missing() > 0 && st.filter.is_none_or(|f| n.piece == f))
        .collect();
    let farm = |n: &Need| n.farm.or(st.demo_farm.then_some("Sample quest, area 3"));
    match st.view {
        View::Monster => {
            // One line per item and monster, adding up what several pieces need.
            struct Line<'a> {
                first: &'a Need,
                need: u16,
                owned: u16,
                pieces: Vec<usize>,
            }
            let mut lines: Vec<Line> = Vec::new();
            for n in &needs {
                match lines
                    .iter_mut()
                    .find(|l| l.first.item == n.item && l.first.monster == n.monster)
                {
                    Some(l) => {
                        l.need += n.need;
                        l.owned += n.owned;
                        if !l.pieces.contains(&n.piece) {
                            l.pieces.push(n.piece);
                        }
                    }
                    None => lines.push(Line {
                        first: n,
                        need: n.need,
                        owned: n.owned,
                        pieces: vec![n.piece],
                    }),
                }
            }
            let missing = |l: &Line| l.need.saturating_sub(l.owned);
            let mut monsters: Vec<&'static str> = Vec::new();
            for l in &lines {
                if !monsters.contains(&l.first.monster) {
                    monsters.push(l.first.monster);
                }
            }
            let total = |m: &str| -> u32 {
                lines
                    .iter()
                    .filter(|l| l.first.monster == m)
                    .map(|l| missing(l) as u32)
                    .sum()
            };
            // The monster with the most missing parts first: that is what to hunt next.
            monsters.sort_by(|a, b| total(b).cmp(&total(a)).then(a.cmp(b)));
            let mut out = Vec::new();
            for m in monsters {
                let mut mine: Vec<&Line> = lines.iter().filter(|l| l.first.monster == m).collect();
                mine.sort_by(|a, b| {
                    missing(b)
                        .cmp(&missing(a))
                        .then(a.first.item.cmp(b.first.item))
                });
                out.push(Row::Head {
                    title: m.to_string(),
                    tail: plural(mine.len(), "ITEM", "ITEMS"),
                    tip: format!("{m}: {}.", plural(mine.len(), "item", "items")),
                });
                for (i, l) in mine.iter().enumerate() {
                    let names: Vec<&str> = l
                        .pieces
                        .iter()
                        .filter_map(|p| data.piece_at(*p).map(|p| p.name))
                        .collect();
                    out.push(Row::Item(ItemRow {
                        name: l.first.item,
                        icon_id: l.first.icon_id,
                        need: l.need,
                        owned: l.owned,
                        ctx: names.join(", "),
                        farm: farm(l.first),
                        tip: format!(
                            "{}. You have {} of {}. From {m}. For {}.",
                            l.first.item,
                            l.owned,
                            l.need,
                            names.join(", ")
                        ),
                        line: i + 1 < mine.len(),
                    }));
                }
            }
            out
        }
        View::Piece | View::Path => {
            let mut out = Vec::new();
            for piece in 0..PIECES {
                let mine: Vec<&&Need> = needs.iter().filter(|n| n.piece == piece).collect();
                let Some(p) = data.piece_at(piece).filter(|_| !mine.is_empty()) else {
                    continue;
                };
                out.push(Row::Head {
                    title: p.name.to_string(),
                    tail: plural(mine.len(), "ITEM", "ITEMS"),
                    tip: format!(
                        "{}: {} missing.",
                        p.name,
                        plural(mine.len(), "item", "items")
                    ),
                });
                let chain = data
                    .chains
                    .iter()
                    .find(|(i, _)| *i == piece)
                    .filter(|_| st.view == View::Path);
                let steps: Vec<u8> = match chain {
                    Some((_, names)) => (0..names.len() as u8).collect(),
                    None => vec![0],
                };
                for s in steps {
                    let mut here: Vec<&&Need> = mine
                        .iter()
                        .copied()
                        .filter(|n| chain.is_none() || n.step == s)
                        .collect();
                    if here.is_empty() {
                        continue;
                    }
                    if let Some((_, names)) = chain {
                        let to = names[s as usize].to_uppercase();
                        out.push(Row::Step(if s == 0 {
                            format!("CRAFT {to}")
                        } else {
                            format!("UPGRADE TO {to}")
                        }));
                    }
                    here.sort_by(|a, b| b.missing().cmp(&a.missing()).then(a.item.cmp(b.item)));
                    for (i, n) in here.iter().enumerate() {
                        out.push(Row::Item(ItemRow {
                            name: n.item,
                            icon_id: n.icon_id,
                            need: n.need,
                            owned: n.owned,
                            ctx: n.monster.to_string(),
                            farm: farm(n),
                            tip: format!(
                                "{}. You have {} of {}. From {}. For {}.",
                                n.item, n.owned, n.need, n.monster, p.name
                            ),
                            line: i + 1 < here.len(),
                        }));
                    }
                }
            }
            out
        }
    }
}

/// How many distinct items and monsters are missing, for the summary line.
fn summary(data: &BuildData, st: &ShoppingState) -> String {
    let mut items: Vec<&str> = Vec::new();
    let mut monsters: Vec<&str> = Vec::new();
    for n in data
        .needs
        .iter()
        .filter(|n| n.missing() > 0 && st.filter.is_none_or(|f| n.piece == f))
    {
        if !items.contains(&n.item) {
            items.push(n.item);
        }
        if !monsters.contains(&n.monster) {
            monsters.push(n.monster);
        }
    }
    if items.is_empty() {
        return String::new();
    }
    format!(
        "{} \u{b7} {}",
        plural(items.len(), "ITEM", "ITEMS"),
        plural(monsters.len(), "MONSTER", "MONSTERS")
    )
}

/// The three-way view switch and, when set, the piece filter chip. Returns their hit rectangles.
pub(super) fn header(
    p: &Painter,
    hits: &mut Vec<(Rect, Hit)>,
    content: Rect,
    data: &BuildData,
    st: &ShoppingState,
) {
    let ppp = p.ctx().pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let hover = p.ctx().pointer_hover_pos();
    let left = content.min.x + MARGIN;
    let right = content.max.x - MARGIN;
    p.text(
        Pos2::new(right, content.min.y + 30.0),
        Align2::RIGHT_CENTER,
        "SAMPLE DATA, NOT YOUR SAVE",
        mono(10.5),
        WARN,
    );
    let top = snap(content.min.y + VIEW_TOP);
    let whole = Rect::from_min_size(Pos2::new(snap(left), top), Vec2::new(3.0 * VIEW_W, VIEW_H));
    p.rect_stroke(whole, 2.0, Stroke::new(1.0, LINE), egui::StrokeKind::Inside);
    for (i, v) in View::ALL.iter().enumerate() {
        let r = Rect::from_min_size(
            Pos2::new(whole.left() + i as f32 * VIEW_W, top),
            Vec2::new(VIEW_W, VIEW_H),
        );
        let on = *v == st.view;
        let hot = hover.is_some_and(|h| r.contains(h));
        if on {
            p.rect_filled(r, 2.0, ACCENT);
        }
        p.text(
            r.center(),
            Align2::CENTER_CENTER,
            v.name(),
            font("inter-medium", 12.5),
            if on {
                ACCENT_INK
            } else if hot {
                FG
            } else {
                DIM
            },
        );
        hits.push((r, Hit::ShopView(*v)));
    }

    // The filter chip: the piece name (plain ellipsis, full name on hover) and a close mark.
    let mut x = whole.right() + 16.0;
    if let Some(piece) = st.filter {
        let name = data.piece_at(piece).map_or("Piece", |p| p.name);
        let r = Rect::from_min_size(
            Pos2::new(snap(x), snap(top + (VIEW_H - CHIP_H) / 2.0)),
            Vec2::new(FILTER_W, CHIP_H),
        );
        let hot = hover.is_some_and(|h| r.contains(h));
        p.rect_filled(r, 2.0, CHIP_FILL);
        p.rect_stroke(
            r,
            2.0,
            Stroke::new(1.0, if hot { DIM } else { FAINT }),
            egui::StrokeKind::Inside,
        );
        p.text(
            Pos2::new(r.left() + 10.0, r.center().y),
            Align2::LEFT_CENTER,
            fit(p, name, FILTER_W - 40.0, "inter", 12.0),
            font("inter", 12.0),
            FG,
        );
        let c = Pos2::new(r.right() - 12.0, r.center().y);
        let st = Stroke::new(1.2, if hot { FG } else { DIM });
        p.line_segment([c + Vec2::new(-3.0, -3.0), c + Vec2::new(3.0, 3.0)], st);
        p.line_segment([c + Vec2::new(-3.0, 3.0), c + Vec2::new(3.0, -3.0)], st);
        hits.push((r, Hit::ShopFilter));
        x = r.right() + 16.0;
    }
    let _ = x;
    let text = summary(data, st);
    p.text(
        Pos2::new(right, top + VIEW_H / 2.0),
        Align2::RIGHT_CENTER,
        text,
        mono(10.5),
        DIM,
    );
}

/// The list. Drawn into `ui` after the shell, in the content area under the header.
pub fn list(
    ui: &mut Ui,
    content: Rect,
    data: &BuildData,
    st: &ShoppingState,
    cache: &mut IconCache,
) {
    let ctx = ui.ctx().clone();
    let ppp = ctx.pixels_per_point();
    let snap = |v: f32| (v * ppp).round() / ppp;
    let left = content.min.x + MARGIN;
    let right = content.max.x - MARGIN;
    let top = snap(content.min.y + LIST_TOP);
    let area = Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, content.max.y - 12.0));
    if area.height() < ROW_H {
        return;
    }
    let all = rows(data, st);
    let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(area));
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    if all.is_empty() {
        let (r, _) = ui.allocate_exact_size(Vec2::new(area.width(), ROW_H), egui::Sense::hover());
        let (text, col) = if st.filter.is_some() {
            ("Nothing missing for this piece.", OK)
        } else {
            ("Nothing missing. You have every part for this build.", OK)
        };
        ui.painter().text(
            Pos2::new(r.left(), r.center().y),
            Align2::LEFT_CENTER,
            text,
            font("inter", 12.5),
            col,
        );
        return;
    }
    // Farm place column: only when at least one row has one.
    let farm_shown = all
        .iter()
        .any(|r| matches!(r, Row::Item(i) if i.farm.is_some()));
    let status_x = |r: Rect| r.right() - STATUS_W;
    let count_right = |r: Rect| status_x(r) - COL_GAP;
    let farm_right = |r: Rect| count_right(r) - COUNT_W - COL_GAP;
    let ctx_right = |r: Rect| {
        if farm_shown {
            farm_right(r) - FARM_W - COL_GAP
        } else {
            farm_right(r)
        }
    };
    let mut scroll = egui::ScrollArea::vertical()
        .id_salt(("shopping", st.view as u8, st.filter))
        .auto_shrink([false, false])
        .max_height(area.height());
    if let Some(y) = st.demo_scroll {
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
                Row::Head { title, tail, tip } => {
                    let x = r.left();
                    let tail_w = p.layout_no_wrap(tail.clone(), mono(10.5), FAINT).size().x;
                    p.text(
                        Pos2::new(r.right(), cy),
                        Align2::RIGHT_CENTER,
                        tail,
                        mono(10.5),
                        FAINT,
                    );
                    p.text(
                        Pos2::new(x, cy),
                        Align2::LEFT_CENTER,
                        fit(
                            p,
                            title,
                            r.right() - x - tail_w - 16.0,
                            "inter-medium",
                            13.5,
                        ),
                        font("inter-medium", 13.5),
                        FG,
                    );
                    hair(p, r.left(), r.right(), r.bottom() - 0.5);
                    resp.on_hover_text(tip);
                }
                Row::Step(label) => {
                    p.text(
                        Pos2::new(r.left() + NAME_X, cy),
                        Align2::LEFT_CENTER,
                        fit(p, label, r.width() - NAME_X, "mono", 10.5),
                        mono(10.5),
                        DIM,
                    );
                }
                Row::Item(it) => {
                    let tile = Rect::from_center_size(
                        Pos2::new(r.left() + TILE_S / 2.0, cy),
                        Vec2::splat(TILE_S),
                    );
                    cache.tile(p, tile, it.icon_id, Icon::Cart);
                    let miss = it.need.saturating_sub(it.owned);
                    p.text(
                        Pos2::new(status_x(r), cy),
                        Align2::LEFT_CENTER,
                        format!("Missing {miss}"),
                        font("inter-medium", 12.0),
                        WARN,
                    );
                    p.text(
                        Pos2::new(count_right(r), cy),
                        Align2::RIGHT_CENTER,
                        format!("{}/{}", it.owned, it.need),
                        mono(11.5),
                        DIM,
                    );
                    if let Some(f) = it.farm.filter(|_| farm_shown) {
                        p.text(
                            Pos2::new(farm_right(r) - FARM_W, cy),
                            Align2::LEFT_CENTER,
                            fit(p, f, FARM_W, "inter", 12.0),
                            font("inter", 12.0),
                            DIM,
                        );
                    }
                    let ctx_left = ctx_right(r) - CTX_W;
                    p.text(
                        Pos2::new(ctx_left, cy),
                        Align2::LEFT_CENTER,
                        fit(p, &it.ctx, CTX_W, "inter", 12.0),
                        font("inter", 12.0),
                        DIM,
                    );
                    let name_x = r.left() + NAME_X;
                    p.text(
                        Pos2::new(name_x, cy),
                        Align2::LEFT_CENTER,
                        fit(p, it.name, ctx_left - COL_GAP - name_x, "inter", 13.0),
                        font("inter", 13.0),
                        FG,
                    );
                    if it.line {
                        hair(p, r.left(), r.right(), r.bottom() - 0.5);
                    }
                    resp.on_hover_text(it.tip.clone());
                }
            }
        }
    });
}

/// The hover for `Missing N` on a Build row: the piece's missing items, at most this many lines.
const HOVER_LINES: usize = 6;

pub fn missing_tip(ui: &mut Ui, data: &BuildData, piece: usize) {
    let needs: Vec<&Need> = data
        .needs
        .iter()
        .filter(|n| n.piece == piece && n.missing() > 0)
        .collect();
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let lines = needs.len().min(HOVER_LINES);
    let extra = needs.len() > HOVER_LINES;
    // One fixed grid: item, count, monster. Long names get a plain ellipsis.
    let size = Vec2::new(
        TIP_ITEM_W + TIP_COUNT_W + TIP_MONSTER_W,
        TIP_ROW * (lines + 1 + extra as usize + 1) as f32 + 4.0,
    );
    let (r, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let p = ui.painter();
    let at = |row: usize| r.top() + TIP_ROW * row as f32 + TIP_ROW / 2.0;
    p.text(
        Pos2::new(r.left(), at(0)),
        Align2::LEFT_CENTER,
        "MISSING",
        mono(10.5),
        DIM,
    );
    for (i, n) in needs.iter().take(HOVER_LINES).enumerate() {
        let y = at(i + 1);
        p.text(
            Pos2::new(r.left(), y),
            Align2::LEFT_CENTER,
            fit(p, n.item, TIP_ITEM_W - 12.0, "inter", 12.0),
            font("inter", 12.0),
            FG,
        );
        p.text(
            Pos2::new(r.left() + TIP_ITEM_W, y),
            Align2::LEFT_CENTER,
            format!("\u{d7}{}", n.missing()),
            mono(11.5),
            WARN,
        );
        p.text(
            Pos2::new(r.left() + TIP_ITEM_W + TIP_COUNT_W, y),
            Align2::LEFT_CENTER,
            fit(p, n.monster, TIP_MONSTER_W, "inter", 12.0),
            font("inter", 12.0),
            DIM,
        );
    }
    let mut row = lines + 1;
    if extra {
        p.text(
            Pos2::new(r.left(), at(row)),
            Align2::LEFT_CENTER,
            format!("+{} more in Shopping", needs.len() - HOVER_LINES),
            font("inter", 11.5),
            DIM,
        );
        row += 1;
    }
    p.text(
        Pos2::new(r.left(), at(row)),
        Align2::LEFT_CENTER,
        "Click to open them in Shopping",
        font("inter", 11.5),
        FAINT,
    );
}

const TIP_ROW: f32 = 20.0;
const TIP_ITEM_W: f32 = 200.0;
const TIP_COUNT_W: f32 = 36.0;
const TIP_MONSTER_W: f32 = 180.0;

/// How many piece slots there are: weapon, five armour, charm.
pub const PIECES: usize = 7;

/// Invented needs for the review screenshots. Item names are real Wilds materials (cited in
/// docs/research.md) except the marked stress-test one; which monster drops what here is
/// illustrative, because drop tables are not confirmed (open question 12).
pub mod sample {
    use super::*;

    const GFA: &str = "Guardian Fulgur Anjanath";

    fn need(
        piece: usize,
        item: &'static str,
        icon_id: u32,
        monster: &'static str,
        n: u16,
        owned: u16,
        step: u8,
    ) -> Need {
        Need {
            piece,
            item,
            icon_id,
            monster,
            need: n,
            owned,
            step,
            farm: None,
        }
    }

    /// The needs that match the default Build sample: the helm misses two items, the greaves one.
    pub fn light() -> Vec<Need> {
        vec![
            need(1, "Thunder Sac", 20, GFA, 3, 1, 0),
            need(1, "Guardian Fulgur Fang +", 21, GFA, 9, 4, 0),
            need(5, "Wyvern Gem", 22, "Rathalos", 1, 0, 0),
        ]
    }

    /// A mid-game list: four pieces, five monsters.
    pub fn mid() -> Vec<Need> {
        vec![
            need(0, "Wyvern Gem", 22, "Rathalos", 1, 0, 1),
            need(0, "Guardian Blood +", 23, "Guardian Rathalos", 2, 0, 1),
            need(1, "Thunder Sac", 20, GFA, 3, 1, 0),
            need(1, "Guardian Fulgur Fang +", 21, GFA, 9, 4, 0),
            need(2, "Guardian Fulgur Scale +", 24, GFA, 14, 6, 0),
            need(2, "Guardian Pelt +", 25, "Guardian Doshaguma", 3, 3, 0),
            need(5, "Wyvern Gem", 22, "Rathalos", 1, 0, 0),
            need(5, "Guardian Fulgur Tail", 26, GFA, 2, 0, 0),
        ]
    }

    /// A full endgame build: six pieces, many items, shared items and the longest names.
    pub fn endgame() -> Vec<Need> {
        let mut v = vec![
            // Weapon, two upgrade steps.
            need(0, "Guardian Fulgur Certificate S", 27, GFA, 2, 0, 1),
            need(0, "Guardian Fulgur Gem", 28, GFA, 1, 0, 1),
            need(0, "Wyvern Gem", 22, "Rathalos", 1, 0, 1),
            need(0, "Guardian Fulgur Nosebone", 29, GFA, 4, 1, 2),
            need(0, "Guardian Blood +", 23, "Guardian Rathalos", 2, 0, 2),
            // Armour: the same materials repeat across pieces.
            need(1, "Guardian Fulgur Certificate S", 27, GFA, 1, 0, 0),
            need(1, "Guardian Fulgur Scale +", 24, GFA, 14, 6, 0),
            need(1, "Guardian Fulgur Fang +", 21, GFA, 9, 4, 0),
            need(1, "Thunder Sac", 20, GFA, 3, 1, 0),
            need(1, "Guardian Fulgur Tail", 26, GFA, 2, 0, 0),
            need(1, "Wyvern Gem", 22, "Rathalos", 1, 0, 0),
            need(1, "Guardian Pelt +", 25, "Guardian Doshaguma", 3, 0, 0),
            need(2, "Guardian Fulgur Pelt +", 30, GFA, 11, 2, 0),
            need(2, "Guardian Fulgur Nosebone", 29, GFA, 8, 2, 0),
            need(2, "Guardian Blood +", 23, "Guardian Rathalos", 2, 0, 0),
            need(2, "Guardian Fulgur Certificate S", 27, GFA, 1, 0, 0),
            need(3, "Guardian Fulgur Tail", 26, GFA, 2, 0, 0),
            need(3, "Guardian Pelt +", 25, "Guardian Doshaguma", 3, 1, 0),
            need(3, "Wyvern Gem", 22, "Rathalos", 1, 0, 0),
            need(4, "Guardian Fulgur Scale +", 24, GFA, 8, 0, 0),
            need(4, "Guardian Fulgur Fang +", 21, GFA, 5, 0, 0),
            need(4, "Thunder Sac", 20, GFA, 2, 0, 0),
            need(5, "Guardian Pelt +", 25, "Guardian Doshaguma", 4, 0, 0),
            need(5, "Guardian Fulgur Pelt +", 30, GFA, 6, 1, 0),
            need(5, "Guardian Fulgur Gem", 28, GFA, 1, 0, 0),
        ];
        // Stress test only: a name longer than any real one, and a long monster name pair.
        v.push(need(
            4,
            "Guardian Fulgur Anjanath Certificate S+ (stress test)",
            31,
            "Guardian Fulgur Anjanath (Arch-Tempered stress test)",
            1,
            0,
            0,
        ));
        v
    }

    /// The weapon's upgrade chain in the sample.
    pub const CHAIN: &[&str] = &["Sample Bowgun I", "Sample Bowgun II", "Sample Bowgun III"];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(needs: Vec<Need>) -> BuildData {
        let mut d = BuildData::sample();
        d.set_needs(needs, vec![(0, sample::CHAIN)]);
        d
    }

    fn items(rows: &[Row]) -> Vec<&'static str> {
        rows.iter()
            .filter_map(|r| match r {
                Row::Item(i) => Some(i.name),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_default_sample_agrees_with_its_build_rows() {
        let d = BuildData::sample();
        for i in 0..PIECES {
            let want = d.piece_at(i).map_or(0, |p| p.missing as usize);
            let got = d
                .needs
                .iter()
                .filter(|n| n.piece == i && n.missing() > 0)
                .count();
            assert_eq!(want, got, "piece {i}");
        }
    }

    #[test]
    fn set_needs_keeps_the_build_rows_in_step() {
        for needs in [sample::light(), sample::mid(), sample::endgame()] {
            let d = data(needs);
            for i in 0..PIECES {
                let want = d
                    .needs
                    .iter()
                    .filter(|n| n.piece == i && n.missing() > 0)
                    .count();
                assert_eq!(d.piece_at(i).map_or(0, |p| p.missing as usize), want);
            }
        }
    }

    #[test]
    fn owned_items_are_not_listed() {
        let d = data(sample::mid());
        let r = rows(&d, &ShoppingState::default());
        // Guardian Pelt + is fully owned in the mid sample.
        assert!(!items(&r).contains(&"Guardian Pelt +"));
    }

    #[test]
    fn by_monster_adds_up_a_shared_item_and_leads_with_the_biggest_gap() {
        let d = data(sample::endgame());
        let r = rows(&d, &ShoppingState::default());
        let Row::Head { title, .. } = &r[0] else {
            panic!("head first")
        };
        assert_eq!(title, "Guardian Fulgur Anjanath");
        // One line for the certificate, not one per piece.
        let certs = items(&r)
            .iter()
            .filter(|n| **n == "Guardian Fulgur Certificate S")
            .count();
        assert_eq!(certs, 1);
        let line = r
            .iter()
            .find_map(|r| match r {
                Row::Item(i) if i.name == "Guardian Fulgur Certificate S" => Some(i),
                _ => None,
            })
            .unwrap();
        // The weapon needs 2, the helm 1 and the mail 1.
        assert_eq!((line.need, line.owned), (4, 0));
    }

    #[test]
    fn the_piece_filter_limits_every_view() {
        let d = data(sample::endgame());
        for view in View::ALL {
            let st = ShoppingState {
                view,
                filter: Some(2),
                ..Default::default()
            };
            let r = rows(&d, &st);
            let n = items(&r).len();
            let want = d
                .needs
                .iter()
                .filter(|n| n.piece == 2 && n.missing() > 0)
                .map(|n| n.item)
                .collect::<std::collections::HashSet<_>>()
                .len();
            assert_eq!(n, want, "{view:?}");
        }
    }

    #[test]
    fn the_upgrade_view_splits_a_chain_into_steps() {
        let d = data(sample::endgame());
        let st = ShoppingState {
            view: View::Path,
            ..Default::default()
        };
        let r = rows(&d, &st);
        let steps: Vec<_> = r
            .iter()
            .filter_map(|r| match r {
                Row::Step(s) => Some(s.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0], "UPGRADE TO SAMPLE BOWGUN II");
        // Armour pieces have no chain: no step rows under them.
        let by_piece = rows(
            &d,
            &ShoppingState {
                view: View::Piece,
                ..Default::default()
            },
        );
        assert!(by_piece.iter().all(|r| !matches!(r, Row::Step(_))));
    }

    #[test]
    fn nothing_missing_gives_no_rows_and_no_summary() {
        let d = data(Vec::new());
        assert!(rows(&d, &ShoppingState::default()).is_empty());
        assert_eq!(summary(&d, &ShoppingState::default()), "");
    }

    #[test]
    fn the_farm_column_has_a_value_only_with_data() {
        let d = data(sample::mid());
        let plain = rows(&d, &ShoppingState::default());
        assert!(plain
            .iter()
            .all(|r| !matches!(r, Row::Item(i) if i.farm.is_some())));
        let demo = rows(
            &d,
            &ShoppingState {
                demo_farm: true,
                ..Default::default()
            },
        );
        assert!(demo
            .iter()
            .any(|r| matches!(r, Row::Item(i) if i.farm.is_some())));
    }
}
