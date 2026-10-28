// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 1 of docs/ux/design-plan.md: one Build-like screen in three style variants, drawn in
//! real egui so the maintainer can pick a style before any style rules are written.
//!
//! Everything shown is sample data with invented names. Icons are placeholder shapes, not game
//! art. Nothing animates and nothing blurs: flat fills, hairlines and at most one static gradient.

pub mod render;

use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{
    Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2, Rect, Sense,
    Shape, Stroke, StrokeKind, Ui, Vec2,
};
use std::sync::Arc;

/// Logical size of the sample screen (the plan's minimum readable window).
pub const SCREEN: Vec2 = Vec2::new(1000.0, 700.0);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Variant {
    /// Near-monochrome, hairlines, mono labels and numbers, numbered rows.
    ModernDark,
    /// Soft slate surfaces, sentence-case sans labels, one blue accent.
    Slate,
    /// Warm black, icon tiles, one static header gradient, one orange accent.
    Ember,
    /// The maintainer's pick: A as the base with C's orange accent, round skill dots and icon tiles.
    ModernEmber,
    /// D plus C's header gradient.
    ModernEmberGradient,
    /// D plus C's tinted rail item.
    ModernEmberRail,
    /// D plus both.
    ModernEmberBoth,
}

impl Variant {
    pub const ALL: [Variant; 7] = [
        Variant::ModernDark,
        Variant::Slate,
        Variant::Ember,
        Variant::ModernEmber,
        Variant::ModernEmberGradient,
        Variant::ModernEmberRail,
        Variant::ModernEmberBoth,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Variant::ModernDark => "a-modern-dark",
            Variant::Slate => "b-slate",
            Variant::Ember => "c-ember",
            Variant::ModernEmber => "d-modern-ember",
            Variant::ModernEmberGradient => "d1-modern-ember-gradient",
            Variant::ModernEmberRail => "d2-modern-ember-rail",
            Variant::ModernEmberBoth => "d3-modern-ember-both",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Variant::ModernDark => "A. Modern Dark",
            Variant::Slate => "B. Slate",
            Variant::Ember => "C. Ember",
            Variant::ModernEmber => "D. Modern Ember",
            Variant::ModernEmberGradient => "D1. Modern Ember + header gradient",
            Variant::ModernEmberRail => "D2. Modern Ember + tinted rail item",
            Variant::ModernEmberBoth => "D3. Modern Ember + both",
        }
    }

    fn style(self) -> Style {
        let c = |r, g, b| Color32::from_rgb(r, g, b);
        let base = Style {
            bg: c(10, 10, 11),
            fg: c(236, 236, 238),
            dim: c(140, 140, 148),
            faint: c(82, 82, 90),
            line: c(34, 34, 38),
            fill: Color32::TRANSPARENT,
            accent: c(190, 242, 100),
            accent_ink: c(12, 14, 6),
            ok: c(76, 195, 138),
            warn: c(229, 176, 74),
            radius: 2.0,
            upper_labels: true,
            icon_tiles: false,
            gradient: None,
            title_size: 20.0,
            numbered: true,
            skills: SkillStyle::Squares,
            headline: Headline::MonoCaps,
            tinted_rail: false,
        };
        match self {
            Variant::ModernDark => base,
            Variant::Slate => Style {
                bg: c(15, 18, 23),
                fg: c(230, 234, 240),
                dim: c(139, 149, 163),
                faint: c(85, 94, 108),
                line: c(36, 43, 52),
                fill: c(21, 26, 33),
                accent: c(90, 169, 255),
                accent_ink: c(6, 14, 26),
                radius: 8.0,
                upper_labels: false,
                icon_tiles: false,
                title_size: 22.0,
                numbered: false,
                skills: SkillStyle::ThinBars,
                headline: Headline::Plain,
                ..base
            },
            Variant::ModernEmber
            | Variant::ModernEmberGradient
            | Variant::ModernEmberRail
            | Variant::ModernEmberBoth => {
                let d = Style {
                    accent: c(255, 138, 61),
                    accent_ink: c(26, 12, 2),
                    icon_tiles: true,
                    skills: SkillStyle::Dots,
                    ..base
                };
                let gradient = Some((c(38, 24, 16), d.bg));
                match self {
                    Variant::ModernEmberGradient => Style { gradient, ..d },
                    Variant::ModernEmberRail => Style {
                        tinted_rail: true,
                        ..d
                    },
                    Variant::ModernEmberBoth => Style {
                        gradient,
                        tinted_rail: true,
                        ..d
                    },
                    _ => d,
                }
            }
            Variant::Ember => Style {
                bg: c(12, 10, 9),
                fg: c(242, 236, 230),
                dim: c(160, 148, 138),
                faint: c(98, 88, 80),
                line: c(44, 37, 33),
                fill: c(22, 18, 16),
                accent: c(255, 138, 61),
                accent_ink: c(26, 12, 2),
                radius: 6.0,
                upper_labels: false,
                icon_tiles: true,
                gradient: Some((c(38, 24, 16), c(12, 10, 9))),
                title_size: 24.0,
                numbered: false,
                skills: SkillStyle::Dots,
                headline: Headline::Big,
                tinted_rail: true,
                ..base
            },
        }
    }
}

struct Style {
    bg: Color32,
    fg: Color32,
    dim: Color32,
    faint: Color32,
    line: Color32,
    /// Row and control fill; transparent means hairlines only.
    fill: Color32,
    accent: Color32,
    accent_ink: Color32,
    ok: Color32,
    warn: Color32,
    radius: f32,
    upper_labels: bool,
    icon_tiles: bool,
    gradient: Option<(Color32, Color32)>,
    title_size: f32,
    numbered: bool,
    /// How skill levels are drawn.
    skills: SkillStyle,
    /// Headline word style: mono caps (A), regular (B) or large and tinted (C).
    headline: Headline,
    /// Selected rail item gets a tinted fill (C) instead of a plain one.
    tinted_rail: bool,
}

#[derive(Clone, Copy)]
enum SkillStyle {
    Squares,
    ThinBars,
    Dots,
}

#[derive(Clone, Copy)]
enum Headline {
    MonoCaps,
    Plain,
    Big,
}

#[derive(Clone, PartialEq, Eq)]
pub struct State {
    pub variant: Variant,
    pub section: usize,
    pub multiplayer: bool,
    pub alt: usize,
}

impl State {
    pub fn new(variant: Variant) -> Self {
        State {
            variant,
            section: 0,
            multiplayer: false,
            alt: 0,
        }
    }
}

/// Installs the fonts. Inter for text, JetBrains Mono for labels and numbers (both OFL, see
/// THIRD-PARTY-NOTICES.md).
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::empty();
    let faces: [(&str, &[u8]); 4] = [
        ("inter", include_bytes!("../fonts/Inter-Regular.otf")),
        ("inter-medium", include_bytes!("../fonts/Inter-Medium.otf")),
        ("mono", include_bytes!("../fonts/JetBrainsMono-Regular.ttf")),
        (
            "mono-medium",
            include_bytes!("../fonts/JetBrainsMono-Medium.ttf"),
        ),
    ];
    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_static(bytes)));
        fonts
            .families
            .insert(FontFamily::Name(name.into()), vec![name.into()]);
    }
    fonts
        .families
        .insert(FontFamily::Proportional, vec!["inter".into()]);
    fonts
        .families
        .insert(FontFamily::Monospace, vec!["mono".into()]);
    ctx.set_fonts(fonts);
}

fn font(name: &str, size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(name.into()))
}

struct Ctx<'a> {
    p: &'a Painter,
    s: &'a Style,
}

impl Ctx<'_> {
    fn text(&self, pos: Pos2, a: Align2, t: &str, f: FontId, col: Color32) -> Rect {
        self.p.text(pos, a, t, f, col)
    }

    /// A small section label: mono caps in A, plain sans elsewhere.
    fn label(&self, pos: Pos2, a: Align2, t: &str, col: Color32) -> Rect {
        if self.s.upper_labels {
            self.text(pos, a, &t.to_uppercase(), font("mono", 10.5), col)
        } else {
            self.text(pos, a, t, font("inter-medium", 12.0), col)
        }
    }

    /// Numbers and short codes: always mono.
    fn num(&self, pos: Pos2, a: Align2, t: &str, size: f32, col: Color32) -> Rect {
        self.text(pos, a, t, font("mono", size), col)
    }

    fn hairline(&self, a: Pos2, b: Pos2) {
        self.p.line_segment([a, b], Stroke::new(1.0, self.s.line));
    }

    fn frame(&self, r: Rect, fill: Color32, stroke: Option<Color32>) {
        if fill != Color32::TRANSPARENT {
            self.p.rect_filled(r, self.s.radius, fill);
        }
        if let Some(c) = stroke {
            self.p
                .rect_stroke(r, self.s.radius, Stroke::new(1.0, c), StrokeKind::Inside);
        }
    }

    fn gradient(&self, r: Rect, top: Color32, bottom: Color32) {
        let mut m = Mesh::default();
        for (pos, color) in [
            (r.left_top(), top),
            (r.right_top(), top),
            (r.right_bottom(), bottom),
            (r.left_bottom(), bottom),
        ] {
            m.vertices.push(Vertex {
                pos,
                uv: WHITE_UV,
                color,
            });
        }
        m.indices.extend([0, 1, 2, 0, 2, 3]);
        self.p.add(Shape::mesh(m));
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Build,
    Cart,
    Why,
    Gear,
    Data,
    About,
    Weapon,
    Head,
    Chest,
    Arms,
    Waist,
    Legs,
    Charm,
}

/// Placeholder glyphs made of plain shapes. Real icons come from the user's install later
/// (element 3); these only stand in for them.
fn icon(p: &Painter, r: Rect, ic: Icon, col: Color32) {
    let c = r.center();
    let u = r.width() / 16.0;
    let st = Stroke::new(1.4, col);
    let pt = |x: f32, y: f32| c + Vec2::new(x, y) * u;
    match ic {
        Icon::Build => {
            p.rect_filled(
                Rect::from_center_size(pt(0.0, 4.0), Vec2::new(12.0, 3.0) * u),
                0.5,
                col,
            );
            p.add(Shape::convex_polygon(
                vec![pt(-6.0, 2.0), pt(6.0, 2.0), pt(3.0, -3.0), pt(-3.0, -3.0)],
                col,
                Stroke::NONE,
            ));
        }
        Icon::Cart => {
            p.add(Shape::line(
                vec![
                    pt(-7.0, -6.0),
                    pt(-5.0, -6.0),
                    pt(-3.0, 3.0),
                    pt(5.0, 3.0),
                    pt(6.0, -3.0),
                    pt(-4.5, -3.0),
                ],
                st,
            ));
            p.circle_filled(pt(-2.0, 6.5), 1.3 * u, col);
            p.circle_filled(pt(4.0, 6.5), 1.3 * u, col);
        }
        Icon::Why => {
            p.circle_stroke(pt(0.0, 0.0), 6.5 * u, st);
            p.circle_filled(pt(0.0, -3.0), 1.0 * u, col);
            p.line_segment([pt(0.0, -0.5), pt(0.0, 3.5)], st);
        }
        Icon::Gear => {
            p.circle_stroke(pt(0.0, 0.0), 3.0 * u, st);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let d = Vec2::angled(a);
                p.line_segment([c + d * 5.2 * u, c + d * 7.0 * u], Stroke::new(1.8, col));
            }
        }
        Icon::Data => {
            for y in [-4.5, 0.0, 4.5] {
                p.rect_stroke(
                    Rect::from_center_size(pt(0.0, y), Vec2::new(12.0, 3.4) * u),
                    1.0,
                    st,
                    StrokeKind::Inside,
                );
            }
        }
        Icon::About => {
            p.circle_stroke(pt(0.0, 0.0), 6.5 * u, st);
            p.line_segment([pt(0.0, -0.5), pt(0.0, 3.5)], st);
            p.circle_filled(pt(0.0, -3.0), 1.0 * u, col);
        }
        Icon::Weapon => {
            p.line_segment([pt(-6.0, 6.0), pt(6.0, -6.0)], Stroke::new(2.4, col));
            p.line_segment([pt(-6.0, -1.0), pt(1.0, 6.0)], Stroke::new(1.4, col));
            p.circle_filled(pt(6.0, -6.0), 1.6 * u, col);
        }
        Icon::Head => {
            p.add(Shape::convex_polygon(
                vec![
                    pt(-6.0, 5.0),
                    pt(-6.0, -1.0),
                    pt(-3.0, -6.0),
                    pt(3.0, -6.0),
                    pt(6.0, -1.0),
                    pt(6.0, 5.0),
                    pt(2.5, 5.0),
                    pt(2.5, 1.0),
                    pt(-2.5, 1.0),
                    pt(-2.5, 5.0),
                ],
                col,
                Stroke::NONE,
            ));
        }
        Icon::Chest => {
            p.add(Shape::convex_polygon(
                vec![
                    pt(-7.0, -5.0),
                    pt(-3.0, -6.0),
                    pt(3.0, -6.0),
                    pt(7.0, -5.0),
                    pt(5.0, 6.0),
                    pt(-5.0, 6.0),
                ],
                col,
                Stroke::NONE,
            ));
        }
        Icon::Arms => {
            p.rect_filled(
                Rect::from_center_size(pt(-4.0, 0.0), Vec2::new(5.0, 13.0) * u),
                1.5,
                col,
            );
            p.rect_filled(
                Rect::from_center_size(pt(4.0, 0.0), Vec2::new(5.0, 13.0) * u),
                1.5,
                col,
            );
        }
        Icon::Waist => {
            p.rect_filled(
                Rect::from_center_size(pt(0.0, -1.0), Vec2::new(14.0, 5.0) * u),
                1.5,
                col,
            );
            p.rect_filled(
                Rect::from_center_size(pt(0.0, 4.5), Vec2::new(5.0, 5.0) * u),
                1.0,
                col,
            );
        }
        Icon::Legs => {
            p.rect_filled(
                Rect::from_center_size(pt(-3.2, 0.0), Vec2::new(5.0, 14.0) * u),
                1.2,
                col,
            );
            p.rect_filled(
                Rect::from_center_size(pt(3.2, 0.0), Vec2::new(5.0, 14.0) * u),
                1.2,
                col,
            );
        }
        Icon::Charm => {
            p.add(Shape::convex_polygon(
                vec![pt(0.0, -7.0), pt(5.0, 0.0), pt(0.0, 7.0), pt(-5.0, 0.0)],
                col,
                Stroke::NONE,
            ));
        }
    }
}

struct Piece {
    name: &'static str,
    icon: Icon,
    slots: u8,
    detail: &'static str,
    /// Parts still missing; 0 means owned.
    missing: u8,
}

const PIECES: [Piece; 7] = [
    Piece {
        name: "Sample Bowgun IV",
        icon: Icon::Weapon,
        slots: 2,
        detail: "Sharpshot 2 \u{b7} slot 2, 1",
        missing: 0,
    },
    Piece {
        name: "Sample Helm \u{3b1}",
        icon: Icon::Head,
        slots: 1,
        detail: "Edge Focus 2, Steady 1",
        missing: 2,
    },
    Piece {
        name: "Sample Mail \u{3b2}",
        icon: Icon::Chest,
        slots: 2,
        detail: "Sharpshot 1, Guard Up 2",
        missing: 0,
    },
    Piece {
        name: "Sample Vambraces \u{3b2}",
        icon: Icon::Arms,
        slots: 1,
        detail: "Edge Focus 1",
        missing: 0,
    },
    Piece {
        name: "Sample Coil \u{3b3}",
        icon: Icon::Waist,
        slots: 3,
        detail: "Steady 2, Quick Hands 1",
        missing: 1,
    },
    Piece {
        name: "Sample Greaves \u{3b2}",
        icon: Icon::Legs,
        slots: 0,
        detail: "Guard Up 1, Sharpshot 1",
        missing: 0,
    },
    Piece {
        name: "Sample Charm II",
        icon: Icon::Charm,
        slots: 1,
        detail: "Edge Focus 1",
        missing: 0,
    },
];

const SKILLS: [(&str, u8, u8); 6] = [
    ("Sharpshot", 4, 5),
    ("Edge Focus", 3, 5),
    ("Steady", 3, 3),
    ("Guard Up", 3, 5),
    ("Quick Hands", 1, 3),
    ("Tough Nerves", 2, 3),
];

const SECTIONS: [(&str, Icon); 6] = [
    ("Build", Icon::Build),
    ("Shopping", Icon::Cart),
    ("Why", Icon::Why),
    ("Settings", Icon::Gear),
    ("Data", Icon::Data),
    ("About", Icon::About),
];

const ALTS: [&str; 6] = ["Best", "Safe", "Defensive", "Monster", "General", "Gogma"];

/// Draws the whole screen into `ui` and applies clicks to `state`.
pub fn show(ui: &mut Ui, state: &mut State) {
    let s = state.variant.style();
    let origin = ui.max_rect().min;
    let screen = Rect::from_min_size(origin, SCREEN);
    let painter = ui.painter_at(screen);
    let cx = Ctx { p: &painter, s: &s };
    let at = |x: f32, y: f32| origin + Vec2::new(x, y);

    painter.rect_filled(screen, 0.0, s.bg);
    if let Some((top, bottom)) = s.gradient {
        cx.gradient(
            Rect::from_min_size(at(176.0, 0.0), Vec2::new(824.0, 150.0)),
            top,
            bottom,
        );
    }

    // Left rail.
    cx.hairline(at(176.0, 0.0), at(176.0, 676.0));
    cx.text(
        at(24.0, 30.0),
        Align2::LEFT_CENTER,
        "forgeline",
        font("inter-medium", 16.0),
        s.fg,
    );
    painter.circle_filled(at(104.0, 34.0), 2.5, s.accent);
    for (i, (name, ic)) in SECTIONS.iter().enumerate() {
        let y = if i < 3 {
            84.0 + i as f32 * 40.0
        } else {
            244.0 + (i - 3) as f32 * 40.0
        };
        let r = Rect::from_min_size(at(12.0, y), Vec2::new(152.0, 34.0));
        let resp = ui.interact(r, ui.scope_id().with(("rail", i)), Sense::click());
        if resp.clicked() {
            state.section = i;
        }
        let sel = state.section == i;
        if sel {
            cx.frame(
                r,
                if s.tinted_rail {
                    Color32::from_rgb(44, 30, 21)
                } else {
                    s.fill
                },
                None,
            );
            painter.rect_filled(
                Rect::from_min_size(r.min + Vec2::new(0.0, 7.0), Vec2::new(2.0, 20.0)),
                1.0,
                s.accent,
            );
        }
        let col = if sel || resp.hovered() { s.fg } else { s.dim };
        icon(
            &painter,
            Rect::from_center_size(r.min + Vec2::new(24.0, 17.0), Vec2::splat(16.0)),
            *ic,
            if sel { s.accent } else { col },
        );
        cx.text(
            r.min + Vec2::new(44.0, 17.0),
            Align2::LEFT_CENTER,
            name,
            font("inter-medium", 13.0),
            col,
        );
    }
    cx.hairline(at(24.0, 222.0), at(152.0, 222.0));

    // Header.
    cx.label(
        at(204.0, 30.0),
        Align2::LEFT_CENTER,
        "Build \u{b7} light bowgun",
        s.dim,
    );
    cx.text(
        at(204.0, 58.0),
        Align2::LEFT_CENTER,
        "Best build you can make now",
        font("inter-medium", s.title_size),
        s.fg,
    );
    let tag = cx.label(
        at(204.0, 86.0),
        Align2::LEFT_CENTER,
        "Sample data, not your save",
        s.warn,
    );
    painter.circle_filled(tag.left_center() + Vec2::new(-8.0, 0.0), 2.5, s.warn);

    // Solo / Multiplayer switch and monster selector.
    let sw = Rect::from_min_size(at(780.0, 36.0), Vec2::new(192.0, 30.0));
    cx.frame(sw, s.fill, Some(s.line));
    for (i, name) in ["Solo", "Multiplayer"].iter().enumerate() {
        let half = Rect::from_min_size(
            sw.min + Vec2::new(i as f32 * 72.0, 0.0),
            Vec2::new(if i == 0 { 72.0 } else { 120.0 }, 30.0),
        );
        let resp = ui.interact(half, ui.scope_id().with(("mode", i)), Sense::click());
        if resp.clicked() {
            state.multiplayer = i == 1;
        }
        let sel = state.multiplayer == (i == 1);
        if sel {
            cx.frame(half.shrink(3.0), s.accent, None);
        }
        cx.text(
            half.center(),
            Align2::CENTER_CENTER,
            name,
            font("inter-medium", 12.5),
            if sel { s.accent_ink } else { s.dim },
        );
    }
    let mon = Rect::from_min_size(at(580.0, 36.0), Vec2::new(188.0, 30.0));
    cx.frame(mon, s.fill, Some(s.line));
    cx.label(
        mon.min + Vec2::new(12.0, 15.0),
        Align2::LEFT_CENTER,
        "Monster",
        s.faint,
    );
    cx.text(
        mon.min + Vec2::new(70.0, 15.0),
        Align2::LEFT_CENTER,
        "Sample Wyvern",
        font("inter", 12.5),
        s.fg,
    );
    painter.add(Shape::convex_polygon(
        vec![
            mon.right_center() + Vec2::new(-16.0, -2.0),
            mon.right_center() + Vec2::new(-8.0, -2.0),
            mon.right_center() + Vec2::new(-12.0, 3.0),
        ],
        s.dim,
        Stroke::NONE,
    ));

    // Alternatives row.
    let mut x = 204.0;
    for (i, name) in ALTS.iter().enumerate() {
        let w = 18.0 + name.len() as f32 * 7.2;
        let r = Rect::from_min_size(at(x, 106.0), Vec2::new(w, 28.0));
        let resp = ui.interact(r, ui.scope_id().with(("alt", i)), Sense::click());
        if resp.clicked() {
            state.alt = i;
        }
        let sel = state.alt == i;
        cx.text(
            r.center(),
            Align2::CENTER_CENTER,
            name,
            font("inter-medium", 12.5),
            if sel || resp.hovered() { s.fg } else { s.dim },
        );
        if sel {
            painter.rect_filled(
                Rect::from_min_size(
                    r.left_bottom() + Vec2::new(6.0, -2.0),
                    Vec2::new(w - 12.0, 2.0),
                ),
                1.0,
                s.accent,
            );
        }
        x += w + 4.0;
    }
    cx.hairline(at(204.0, 134.0), at(972.0, 134.0));

    // Equipment column.
    cx.label(at(204.0, 156.0), Align2::LEFT_CENTER, "Equipment", s.dim);
    for (i, pc) in PIECES.iter().enumerate() {
        let r = Rect::from_min_size(at(204.0, 172.0 + i as f32 * 54.0), Vec2::new(392.0, 48.0));
        cx.frame(r, s.fill, None);
        if s.fill == Color32::TRANSPARENT && i > 0 {
            cx.hairline(
                r.left_top() - Vec2::new(0.0, 3.0),
                r.right_top() - Vec2::new(0.0, 3.0),
            );
        }
        let mut ix = r.min.x + 12.0;
        if s.numbered {
            cx.num(
                Pos2::new(ix, r.center().y),
                Align2::LEFT_CENTER,
                &format!("{:02}", i + 1),
                11.0,
                s.faint,
            );
            ix += 28.0;
        }
        let tile = Rect::from_min_size(Pos2::new(ix, r.center().y - 17.0), Vec2::splat(34.0));
        if s.icon_tiles {
            cx.frame(tile, Color32::from_rgb(40, 29, 22), None);
        }
        icon(
            &painter,
            tile.shrink(if s.icon_tiles { 7.0 } else { 3.0 }),
            pc.icon,
            if pc.missing > 0 {
                s.warn
            } else if s.icon_tiles {
                s.accent
            } else {
                s.fg
            },
        );
        let tx = tile.max.x + 12.0;
        cx.text(
            Pos2::new(tx, r.center().y - 8.0),
            Align2::LEFT_CENTER,
            pc.name,
            font("inter-medium", 13.5),
            s.fg,
        );
        cx.text(
            Pos2::new(tx, r.center().y + 9.0),
            Align2::LEFT_CENTER,
            pc.detail,
            font("inter", 11.5),
            s.dim,
        );
        // Decoration slots as small pips.
        for k in 0..pc.slots {
            painter.circle_stroke(
                Pos2::new(r.max.x - 120.0 + k as f32 * 11.0, r.center().y),
                3.5,
                Stroke::new(1.2, s.dim),
            );
        }
        let (txt, col) = if pc.missing == 0 {
            ("Owned".to_string(), s.ok)
        } else {
            (format!("Missing {}", pc.missing), s.warn)
        };
        cx.text(
            Pos2::new(r.max.x - 12.0, r.center().y),
            Align2::RIGHT_CENTER,
            &txt,
            font("inter-medium", 12.0),
            col,
        );
    }

    // Right column: damage and toughness.
    let rx = 636.0;
    let rw = 336.0;
    cx.label(at(rx, 156.0), Align2::LEFT_CENTER, "Damage", s.dim);
    cx.label(at(rx + rw, 156.0), Align2::RIGHT_CENTER, "Modelled", s.warn);
    headline(&cx, at(rx, 184.0), "High");
    cx.num(
        at(rx + rw, 184.0),
        Align2::RIGHT_CENTER,
        "+12% vs your gear",
        12.0,
        s.ok,
    );
    bar(&cx, at(rx, 206.0), rw, 0.78, Some(0.62), s.accent);

    cx.label(at(rx, 240.0), Align2::LEFT_CENTER, "Toughness", s.dim);
    headline(&cx, at(rx, 268.0), "Tankier");
    cx.num(
        at(rx + rw, 268.0),
        Align2::RIGHT_CENTER,
        "+1 hit survived",
        12.0,
        s.ok,
    );
    bar(&cx, at(rx, 290.0), rw, 0.55, Some(0.42), s.accent);

    // Skills strip.
    cx.hairline(at(rx, 316.0), at(rx + rw, 316.0));
    cx.label(at(rx, 336.0), Align2::LEFT_CENTER, "Skills", s.dim);
    for (i, (name, lvl, max)) in SKILLS.iter().enumerate() {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        let p0 = at(rx + col * 172.0, 360.0 + row * 38.0);
        cx.text(p0, Align2::LEFT_TOP, name, font("inter", 12.5), s.fg);
        match s.skills {
            SkillStyle::Squares => {
                for k in 0..*max {
                    let r = Rect::from_min_size(
                        p0 + Vec2::new(k as f32 * 13.0, 19.0),
                        Vec2::new(10.0, 5.0),
                    );
                    painter.rect_filled(r, 1.0, if k < *lvl { s.accent } else { s.line });
                }
            }
            SkillStyle::ThinBars => {
                let track = Rect::from_min_size(p0 + Vec2::new(0.0, 20.0), Vec2::new(150.0, 3.0));
                painter.rect_filled(track, 1.5, s.line);
                painter.rect_filled(
                    Rect::from_min_size(
                        track.min,
                        Vec2::new(150.0 * *lvl as f32 / *max as f32, 3.0),
                    ),
                    1.5,
                    s.accent,
                );
            }
            SkillStyle::Dots => {
                for k in 0..*max {
                    let c = p0 + Vec2::new(k as f32 * 13.0 + 4.0, 22.0);
                    if k < *lvl {
                        painter.circle_filled(c, 4.0, s.accent);
                    } else {
                        painter.circle_stroke(c, 3.5, Stroke::new(1.0, s.faint));
                    }
                }
            }
        }
        cx.num(
            p0 + Vec2::new(0.0 + 150.0, 0.0),
            Align2::RIGHT_TOP,
            &lvl.to_string(),
            12.0,
            s.dim,
        );
    }

    // Must-have chips.
    cx.hairline(at(rx, 478.0), at(rx + rw, 478.0));
    cx.label(at(rx, 498.0), Align2::LEFT_CENTER, "Must-haves", s.dim);
    let mut cxp = rx;
    for chip in ["Sharpshot 4", "Edge Focus 3"] {
        let w = 28.0 + chip.len() as f32 * 6.6;
        let r = Rect::from_min_size(at(cxp, 516.0), Vec2::new(w, 26.0));
        cx.frame(r, s.fill, Some(s.line));
        cx.text(
            r.min + Vec2::new(10.0, 13.0),
            Align2::LEFT_CENTER,
            chip,
            font("inter", 12.0),
            s.fg,
        );
        cx.text(
            r.max - Vec2::new(10.0, 13.0),
            Align2::RIGHT_CENTER,
            "\u{d7}",
            font("inter", 13.0),
            s.dim,
        );
        cxp += w + 8.0;
    }
    let add = Rect::from_min_size(at(cxp, 516.0), Vec2::new(26.0, 26.0));
    cx.frame(add, Color32::TRANSPARENT, Some(s.line));
    cx.text(
        add.center(),
        Align2::CENTER_CENTER,
        "+",
        font("inter", 14.0),
        s.dim,
    );

    // Reasoning sentence.
    cx.text(
        at(rx, 556.0),
        Align2::LEFT_TOP,
        "Highest damage that keeps both must-haves,\nusing only parts you can get now.",
        font("inter", 12.0),
        s.dim,
    );

    // Primary action: one per view.
    let bar_r = Rect::from_min_size(at(204.0, 594.0), Vec2::new(768.0, 56.0));
    cx.frame(bar_r, s.fill, Some(s.line));
    icon(
        &painter,
        Rect::from_center_size(bar_r.min + Vec2::new(30.0, 28.0), Vec2::splat(18.0)),
        Icon::Cart,
        s.accent,
    );
    cx.text(
        bar_r.min + Vec2::new(54.0, 19.0),
        Align2::LEFT_CENTER,
        "Next: Sample Helm \u{3b1}",
        font("inter-medium", 13.5),
        s.fg,
    );
    cx.text(
        bar_r.min + Vec2::new(54.0, 38.0),
        Align2::LEFT_CENTER,
        "2 parts missing, both from Sample Wyvern",
        font("inter", 12.0),
        s.dim,
    );
    let btn = Rect::from_min_size(bar_r.max - Vec2::new(172.0, 44.0), Vec2::new(160.0, 32.0));
    let resp = ui.interact(btn, ui.scope_id().with("primary"), Sense::click());
    cx.frame(btn, if resp.hovered() { s.fg } else { s.accent }, None);
    cx.text(
        btn.center(),
        Align2::CENTER_CENTER,
        "Show shopping list",
        font("inter-medium", 12.5),
        s.accent_ink,
    );

    // Status line.
    cx.hairline(at(0.0, 676.0), at(1000.0, 676.0));
    painter.circle_filled(at(24.0, 688.0), 2.5, s.ok);
    cx.text(
        at(34.0, 688.0),
        Align2::LEFT_CENTER,
        "Save synced 07:14",
        font("inter", 11.5),
        s.dim,
    );
    cx.text(
        at(176.0, 688.0),
        Align2::LEFT_CENTER,
        "Game data 3 days old",
        font("inter", 11.5),
        s.dim,
    );
    cx.num(
        at(976.0, 688.0),
        Align2::RIGHT_CENTER,
        "v0.2.0-sample",
        11.0,
        s.faint,
    );
}

fn headline(cx: &Ctx, at: Pos2, word: &str) {
    match cx.s.headline {
        Headline::MonoCaps => cx.text(
            at,
            Align2::LEFT_CENTER,
            &word.to_uppercase(),
            font("mono-medium", 20.0),
            cx.s.fg,
        ),
        Headline::Plain => cx.text(
            at,
            Align2::LEFT_CENTER,
            word,
            font("inter-medium", 22.0),
            cx.s.fg,
        ),
        Headline::Big => cx.text(
            at,
            Align2::LEFT_CENTER,
            word,
            font("inter-medium", 28.0),
            cx.s.accent,
        ),
    };
}

/// A flat value bar with an optional tick for the current gear.
fn bar(cx: &Ctx, at: Pos2, w: f32, value: f32, current: Option<f32>, col: Color32) {
    let track = Rect::from_min_size(at, Vec2::new(w, 6.0));
    cx.p.rect_filled(track, 3.0, cx.s.line);
    cx.p.rect_filled(Rect::from_min_size(at, Vec2::new(w * value, 6.0)), 3.0, col);
    if let Some(c) = current {
        let x = at.x + w * c;
        cx.p.line_segment(
            [Pos2::new(x, at.y - 4.0), Pos2::new(x, at.y + 10.0)],
            Stroke::new(1.5, cx.s.fg),
        );
    }
}

/// Renders one variant to a frame at `ppp` pixels per point, with no window or GPU.
pub fn render_frame(state: &mut State, ppp: f32) -> render::Frame {
    let ctx = egui::Context::default();
    install_fonts(&ctx);
    let mut rast = render::Rasteriser::default();
    let mut frame = None;
    // The first pass builds the font atlas and layout; the second is the settled picture.
    for _ in 0..2 {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
            viewports: std::iter::once((
                egui::ViewportId::ROOT,
                egui::ViewportInfo {
                    native_pixels_per_point: Some(ppp),
                    ..Default::default()
                },
            ))
            .collect(),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| show(ui, state));
        rast.apply(std::mem::take(&mut out.textures_delta));
        let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
        frame = Some(rast.paint(
            &prims,
            out.pixels_per_point,
            (SCREEN.x * ppp) as usize,
            (SCREEN.y * ppp) as usize,
            state.variant.style().bg,
        ));
    }
    frame.expect("two passes ran")
}
