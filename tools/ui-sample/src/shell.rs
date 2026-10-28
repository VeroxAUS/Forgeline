// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 2 of docs/ux/design-plan.md: the app shell in the locked D3 "Modern Ember" style
//! (docs/ux/style-guide.md). A left rail, a section divider and a status line around a content
//! area. Unlike the element 1 sample it lays out from the window size, so it resizes: the rail
//! keeps its width, the status line stays at the bottom and the content takes the rest.
//!
//! Nothing animates and nothing repaints while idle. The content area is a labelled placeholder
//! until the later elements fill it.

use super::{font, icon, install_fonts, render, Icon};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Align2, Color32, Key, Modifiers, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

/// Smallest window the shell is designed for (design plan principle 8, element 20 tests it).
pub const MIN_SIZE: Vec2 = Vec2::new(1000.0, 700.0);

const RAIL_W: f32 = 176.0;
const STATUS_H: f32 = 24.0;
const MARGIN: f32 = 28.0;

const BG: Color32 = Color32::from_rgb(10, 10, 11);
const FG: Color32 = Color32::from_rgb(236, 236, 238);
const DIM: Color32 = Color32::from_rgb(140, 140, 148);
const FAINT: Color32 = Color32::from_rgb(82, 82, 90);
const LINE: Color32 = Color32::from_rgb(34, 34, 38);
const ACCENT: Color32 = Color32::from_rgb(255, 138, 61);
const OK: Color32 = Color32::from_rgb(76, 195, 138);
const WARN: Color32 = Color32::from_rgb(229, 176, 74);
const RAIL_TINT: Color32 = Color32::from_rgb(44, 30, 21);
const GRADIENT_TOP: Color32 = Color32::from_rgb(38, 24, 16);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Section {
    Build,
    Shopping,
    Why,
    Settings,
    Data,
    About,
}

impl Section {
    /// Planner sections first, then the divider, then the utility sections.
    pub const ALL: [Section; 6] = [
        Section::Build,
        Section::Shopping,
        Section::Why,
        Section::Settings,
        Section::Data,
        Section::About,
    ];
    /// How many sections sit above the divider.
    const PLANNER: usize = 3;

    pub fn name(self) -> &'static str {
        match self {
            Section::Build => "Build",
            Section::Shopping => "Shopping",
            Section::Why => "Why",
            Section::Settings => "Settings",
            Section::Data => "Data",
            Section::About => "About",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            Section::Build => "build",
            Section::Shopping => "shopping",
            Section::Why => "why",
            Section::Settings => "settings",
            Section::Data => "data",
            Section::About => "about",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Section::Build => Icon::Build,
            Section::Shopping => Icon::Cart,
            Section::Why => Icon::Why,
            Section::Settings => Icon::Gear,
            Section::Data => Icon::Data,
            Section::About => Icon::About,
        }
    }

    /// The element that fills this section, shown in the placeholder.
    fn filled_by(self) -> &'static str {
        match self {
            Section::Build => "Elements 4 to 9 fill this screen.",
            Section::Shopping => "Element 10 fills this screen.",
            Section::Why => "Elements 12 to 14 fill this screen.",
            Section::Settings => "Element 16 fills this screen.",
            Section::Data => "Element 15 fills this screen.",
            Section::About => "Element 16 fills this screen.",
        }
    }
}

/// What the status line says. Errors are plain words, one line, never a modal.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Status {
    /// Save read, build current.
    Synced,
    /// Save changed, recompute running in the background; the last build stays on screen.
    Recomputing,
    /// Something went wrong: what happened and what to do, in one line.
    Problem(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Shell {
    pub section: Section,
    pub status: Status,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            section: Section::Build,
            status: Status::Synced,
        }
    }
}

impl Default for Shell {
    fn default() -> Self {
        Self::new()
    }
}

fn mono(size: f32) -> egui::FontId {
    font("mono", size)
}

/// Draws the whole shell into `ui`, filling its available rectangle. The content area shows a
/// labelled placeholder.
pub fn show(ui: &mut Ui, shell: &mut Shell) {
    show_with(ui, shell, &mut |p, content, shell| {
        show_content(p, content, shell);
        placeholder(p, content, shell);
    });
}

/// Like `show`, but `body` draws the content area (header gradient included) itself.
pub fn show_with(ui: &mut Ui, shell: &mut Shell, body: &mut dyn FnMut(&Painter, Rect, &Shell)) {
    let screen = ui.max_rect();
    let painter = ui.painter_at(screen);
    painter.rect_filled(screen, 0.0, BG);

    // Keyboard: Ctrl+1 to Ctrl+6 jump to a section.
    for (i, sec) in Section::ALL.iter().enumerate() {
        let key = [
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
        ][i];
        if ui.input_mut(|inp| inp.consume_key(Modifiers::COMMAND, key)) {
            shell.section = *sec;
        }
    }

    let body_h = screen.height() - STATUS_H;
    let rail = Rect::from_min_size(screen.min, Vec2::new(RAIL_W, body_h));
    let content = Rect::from_min_max(
        screen.min + Vec2::new(RAIL_W, 0.0),
        screen.min + Vec2::new(screen.width(), body_h),
    );
    let status = Rect::from_min_max(screen.min + Vec2::new(0.0, body_h), screen.max);

    body(&painter, content, shell);
    show_rail(ui, &painter, rail, shell);
    show_status(&painter, status, &shell.status);
}

fn gradient(p: &Painter, r: Rect, top: Color32, bottom: Color32) {
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
    p.add(Shape::mesh(m));
}

fn show_rail(ui: &mut Ui, p: &Painter, rail: Rect, shell: &mut Shell) {
    p.line_segment(
        [rail.right_top(), rail.right_bottom()],
        Stroke::new(1.0, LINE),
    );

    // Wordmark with the accent dot.
    let word = p.text(
        rail.min + Vec2::new(24.0, 30.0),
        Align2::LEFT_CENTER,
        "forgeline",
        font("inter-medium", 16.0),
        FG,
    );
    p.circle_filled(word.right_center() + Vec2::new(6.0, 4.0), 2.5, ACCENT);

    let mut y = 72.0;
    for (i, sec) in Section::ALL.iter().enumerate() {
        if i == Section::PLANNER {
            // Section divider between the planner and the utility sections.
            y += 8.0;
            let ly = rail.min.y + y;
            p.line_segment(
                [
                    Pos2::new(rail.min.x + 24.0, ly),
                    Pos2::new(rail.max.x - 24.0, ly),
                ],
                Stroke::new(1.0, LINE),
            );
            y += 12.0;
        }
        let r = Rect::from_min_size(
            Pos2::new(rail.min.x + 12.0, rail.min.y + y),
            Vec2::new(RAIL_W - 24.0, 34.0),
        );
        let resp = ui.interact(r, ui.scope_id().with(("rail", i)), Sense::click());
        if resp.clicked() {
            shell.section = *sec;
        }
        let sel = shell.section == *sec;
        if sel {
            p.rect_filled(r, 2.0, RAIL_TINT);
            p.rect_filled(
                Rect::from_min_size(r.min + Vec2::new(0.0, 7.0), Vec2::new(2.0, 20.0)),
                1.0,
                ACCENT,
            );
        }
        let col = if sel || resp.hovered() { FG } else { DIM };
        icon(
            p,
            Rect::from_center_size(r.min + Vec2::new(24.0, 17.0), Vec2::splat(16.0)),
            sec.icon(),
            if sel { ACCENT } else { col },
        );
        p.text(
            r.min + Vec2::new(44.0, 17.0),
            Align2::LEFT_CENTER,
            sec.name(),
            font("inter-medium", 13.0),
            col,
        );
        y += 40.0;
    }
}

pub(crate) fn show_content(p: &Painter, content: Rect, shell: &Shell) {
    gradient(
        p,
        Rect::from_min_size(content.min, Vec2::new(content.width(), 150.0)),
        GRADIENT_TOP,
        BG,
    );
    let left = content.min.x + MARGIN;
    let sec = shell.section;
    let crumb = if sec == Section::Build {
        "BUILD \u{b7} LIGHT BOWGUN".to_string()
    } else {
        sec.name().to_uppercase()
    };
    p.text(
        Pos2::new(left, content.min.y + 30.0),
        Align2::LEFT_CENTER,
        crumb,
        mono(10.5),
        DIM,
    );
    p.text(
        Pos2::new(left, content.min.y + 58.0),
        Align2::LEFT_CENTER,
        sec.name(),
        font("inter-medium", 20.0),
        FG,
    );
}

/// A hairline frame saying what will live in the content area.
fn placeholder(p: &Painter, content: Rect, shell: &Shell) {
    let left = content.min.x + MARGIN;
    let sec = shell.section;
    let body = Rect::from_min_max(
        Pos2::new(left, content.min.y + 106.0),
        Pos2::new(content.max.x - MARGIN, content.max.y - MARGIN),
    );
    if body.width() > 40.0 && body.height() > 40.0 {
        p.rect_stroke(body, 2.0, Stroke::new(1.0, LINE), egui::StrokeKind::Inside);
        p.text(
            body.center() - Vec2::new(0.0, 10.0),
            Align2::CENTER_CENTER,
            "PLACEHOLDER",
            mono(10.5),
            FAINT,
        );
        p.text(
            body.center() + Vec2::new(0.0, 10.0),
            Align2::CENTER_CENTER,
            sec.filled_by(),
            font("inter", 12.5),
            DIM,
        );
    }
}

fn show_status(p: &Painter, r: Rect, status: &Status) {
    p.line_segment([r.left_top(), r.right_top()], Stroke::new(1.0, LINE));
    let cy = r.center().y;
    let (dot, text) = match status {
        Status::Synced => (OK, "Save synced 07:14".to_string()),
        Status::Recomputing => (DIM, "Save changed, updating the build\u{2026}".to_string()),
        Status::Problem(msg) => (WARN, msg.clone()),
    };
    p.circle_filled(Pos2::new(r.min.x + 24.0, cy), 2.5, dot);
    let t = p.text(
        Pos2::new(r.min.x + 34.0, cy),
        Align2::LEFT_CENTER,
        text,
        font("inter", 11.5),
        if matches!(status, Status::Problem(_)) {
            FG
        } else {
            DIM
        },
    );
    // Data age sits after the save state, never overlapping it.
    let x = (r.min.x + RAIL_W).max(t.right() + 28.0);
    p.text(
        Pos2::new(x, cy),
        Align2::LEFT_CENTER,
        "Game data 3 days old",
        font("inter", 11.5),
        DIM,
    );
    p.text(
        Pos2::new(r.max.x - 24.0, cy),
        Align2::RIGHT_CENTER,
        "v0.2.0-sample",
        mono(11.0),
        FAINT,
    );
}

/// Renders the shell at `size` points and `ppp` pixels per point, with no window or GPU.
pub fn render_frame(shell: &mut Shell, size: Vec2, ppp: f32) -> render::Frame {
    let ctx = egui::Context::default();
    install_fonts(&ctx);
    let mut rast = render::Rasteriser::default();
    let mut frame = None;
    for _ in 0..2 {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
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
        let mut out = ctx.run_ui(input, |ui| show(ui, shell));
        rast.apply(std::mem::take(&mut out.textures_delta));
        let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
        frame = Some(rast.paint(
            &prims,
            out.pixels_per_point,
            (size.x * ppp) as usize,
            (size.y * ppp) as usize,
            BG,
        ));
    }
    frame.expect("two passes ran")
}
