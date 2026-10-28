// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Element 3 of docs/ux/design-plan.md: the icon pipeline. Icons are read at runtime from an
//! [`IconSource`] (later: the user's own install), decoded off the UI thread, downscaled, kept in
//! a byte-capped in-memory cache and drawn in a tile. While an icon is loading, or when it has no
//! icon, the tile shows the plain glyph, so nothing shifts when an icon arrives.
//!
//! Rules this keeps (CLAUDE.md, design plan principle 7):
//! - one worker thread, one icon at a time, started on the first request and gone when the cache
//!   is dropped; nothing runs while idle and a repaint is requested only when an icon arrives;
//! - the cache has a hard byte cap and evicts the least recently drawn icon;
//! - an icon that cannot be read is remembered, not retried every frame;
//! - nothing is written to disk and no game art is committed. The only source here is synthetic.
//!
//! Not verified: which game files hold the icons, their format, or whether they can be read.
//! That needs the maintainer's install; a real source plugs into [`IconSource`] without changing
//! anything else here.

pub use super::Icon as Glyph;
use super::{font, icon, install_fonts, render, Icon};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{
    Align2, Color32, ColorImage, Context, Painter, Pos2, Rect, Shape, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Ui, Vec2,
};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

/// Hard cap on decoded icon bytes held in memory (design plan section 8, RAM headroom).
pub const CACHE_CAP_BYTES: usize = 1024 * 1024;
/// Longest edge of a cached icon in pixels. Tiles are 34 points, so 64 stays sharp at 1.5x.
pub const MAX_EDGE: usize = 64;

const BG: Color32 = Color32::from_rgb(10, 10, 11);
const FG: Color32 = Color32::from_rgb(236, 236, 238);
const DIM: Color32 = Color32::from_rgb(140, 140, 148);
const FAINT: Color32 = Color32::from_rgb(82, 82, 90);
const LINE: Color32 = Color32::from_rgb(34, 34, 38);
const ACCENT: Color32 = Color32::from_rgb(255, 138, 61);
const OK: Color32 = Color32::from_rgb(76, 195, 138);
const WARN: Color32 = Color32::from_rgb(229, 176, 74);
const TILE_FILL: Color32 = Color32::from_rgb(40, 29, 22);
const GRADIENT_TOP: Color32 = Color32::from_rgb(38, 24, 16);
const TILE: f32 = 34.0;

/// A decoded icon: straight RGBA, `width * height * 4` bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawIcon {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

/// Where icons come from. `load` may be slow and runs on the worker thread, never the UI thread.
/// `None` means there is no readable icon for that id.
pub trait IconSource: Send + Sync + 'static {
    fn load(&self, id: u32) -> Option<RawIcon>;
}

/// Shrinks `icon` with a box filter so its longest edge is at most `max_edge`.
pub fn downscale(icon: RawIcon, max_edge: usize) -> RawIcon {
    let longest = icon.width.max(icon.height);
    if longest <= max_edge || icon.width == 0 || icon.height == 0 {
        return icon;
    }
    let f = longest.div_ceil(max_edge);
    let (w, h) = (icon.width.div_ceil(f), icon.height.div_ceil(f));
    let mut out = vec![0u8; w * h * 4];
    for oy in 0..h {
        for ox in 0..w {
            let mut sum = [0u32; 4];
            let mut n = 0;
            for y in oy * f..((oy + 1) * f).min(icon.height) {
                for x in ox * f..((ox + 1) * f).min(icon.width) {
                    let i = (y * icon.width + x) * 4;
                    for (s, v) in sum.iter_mut().zip(&icon.rgba[i..i + 4]) {
                        *s += u32::from(*v);
                    }
                    n += 1;
                }
            }
            for (c, s) in sum.iter().enumerate() {
                out[(oy * w + ox) * 4 + c] = (s / n) as u8;
            }
        }
    }
    RawIcon {
        width: w,
        height: h,
        rgba: out,
    }
}

enum Slot {
    /// Asked for, not back yet.
    Pending,
    /// On screen as a texture.
    Ready {
        tex: TextureHandle,
        bytes: usize,
        last_used: u64,
    },
    /// The source has no icon for this id; not asked again.
    Missing,
}

/// What a tile is showing for an id.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconState {
    NotAsked,
    Loading,
    Ready,
    Missing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CacheStats {
    pub ready: usize,
    pub missing: usize,
    pub pending: usize,
    pub bytes: usize,
}

pub struct IconCache {
    ctx: Context,
    source: Arc<dyn IconSource>,
    slots: HashMap<u32, Slot>,
    cap: usize,
    bytes: usize,
    /// Counts `poll` calls; an icon drawn this tick is never evicted this tick.
    tick: u64,
    worker: Option<Sender<u32>>,
    results: Option<Receiver<(u32, Option<RawIcon>)>>,
}

impl IconCache {
    pub fn new(ctx: &Context, source: Arc<dyn IconSource>) -> Self {
        Self::with_cap(ctx, source, CACHE_CAP_BYTES)
    }

    pub fn with_cap(ctx: &Context, source: Arc<dyn IconSource>, cap: usize) -> Self {
        IconCache {
            ctx: ctx.clone(),
            source,
            slots: HashMap::new(),
            cap,
            bytes: 0,
            tick: 0,
            worker: None,
            results: None,
        }
    }

    pub fn stats(&self) -> CacheStats {
        let mut s = CacheStats {
            bytes: self.bytes,
            ..Default::default()
        };
        for slot in self.slots.values() {
            match slot {
                Slot::Pending => s.pending += 1,
                Slot::Ready { .. } => s.ready += 1,
                Slot::Missing => s.missing += 1,
            }
        }
        s
    }

    pub fn state(&self, id: u32) -> IconState {
        match self.slots.get(&id) {
            None => IconState::NotAsked,
            Some(Slot::Pending) => IconState::Loading,
            Some(Slot::Ready { .. }) => IconState::Ready,
            Some(Slot::Missing) => IconState::Missing,
        }
    }

    /// Takes in finished icons. Call once per frame, before drawing.
    pub fn poll(&mut self) {
        self.tick += 1;
        let Some(rx) = &self.results else { return };
        let done: Vec<_> = rx.try_iter().collect();
        for (id, raw) in done {
            match raw {
                Some(raw) if raw.width > 0 && raw.height > 0 => {
                    let img =
                        ColorImage::from_rgba_unmultiplied([raw.width, raw.height], &raw.rgba);
                    let tex =
                        self.ctx
                            .load_texture(format!("icon-{id}"), img, TextureOptions::LINEAR);
                    self.bytes += raw.rgba.len();
                    self.slots.insert(
                        id,
                        Slot::Ready {
                            tex,
                            bytes: raw.rgba.len(),
                            last_used: 0,
                        },
                    );
                }
                _ => {
                    self.slots.insert(id, Slot::Missing);
                }
            }
        }
        self.evict();
    }

    /// Drops least recently drawn icons until the cache is under its cap again.
    fn evict(&mut self) {
        while self.bytes > self.cap {
            let victim = self
                .slots
                .iter()
                .filter_map(|(id, s)| match s {
                    Slot::Ready { last_used, .. } if *last_used + 1 < self.tick => {
                        Some((*last_used, *id))
                    }
                    _ => None,
                })
                .min();
            let Some((_, id)) = victim else { break };
            if let Some(Slot::Ready { bytes, .. }) = self.slots.remove(&id) {
                self.bytes -= bytes;
            }
        }
    }

    fn request(&mut self, id: u32) {
        if self.worker.is_none() {
            let (req_tx, req_rx) = channel::<u32>();
            let (res_tx, res_rx) = channel();
            let source = self.source.clone();
            let ctx = self.ctx.clone();
            std::thread::Builder::new()
                .name("forgeline-icons".into())
                .spawn(move || {
                    // Blocks until asked; ends when the cache (the sender) is dropped.
                    while let Ok(id) = req_rx.recv() {
                        let raw = source.load(id).map(|r| downscale(r, MAX_EDGE));
                        if res_tx.send((id, raw)).is_err() {
                            break;
                        }
                        ctx.request_repaint();
                    }
                })
                .expect("spawn icon worker");
            self.worker = Some(req_tx);
            self.results = Some(res_rx);
        }
        self.slots.insert(id, Slot::Pending);
        if let Some(tx) = &self.worker {
            let _ = tx.send(id);
        }
    }

    /// Draws a 34-point icon tile: the icon when it is ready, otherwise the plain glyph.
    /// Asks for the icon the first time it is drawn.
    pub fn tile(&mut self, p: &Painter, r: Rect, id: u32, fallback: Icon) {
        p.rect_filled(r, 2.0, TILE_FILL);
        if !self.slots.contains_key(&id) {
            self.request(id);
        }
        if let Some(Slot::Ready { tex, last_used, .. }) = self.slots.get_mut(&id) {
            *last_used = self.tick;
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            p.image(tex.id(), r.shrink(3.0), uv, Color32::WHITE);
        } else {
            icon(p, r.shrink(7.0), fallback, ACCENT);
        }
    }

    /// Blocks until nothing is pending (tests and screenshots only).
    pub fn wait_idle(&mut self, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        loop {
            self.poll();
            if self.stats().pending == 0 {
                return true;
            }
            if start.elapsed() > timeout {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

/// Synthetic icons for the sample and the tests: a coloured badge per id, 128 pixels square.
/// Every seventh id (6, 13, ...) has no icon, to show the fallback.
pub struct SampleSource {
    pub delay: Duration,
}

impl IconSource for SampleSource {
    fn load(&self, id: u32) -> Option<RawIcon> {
        std::thread::sleep(self.delay);
        if id % 7 == 6 {
            return None;
        }
        const N: usize = 128;
        let hue = (id as f32 * 0.17).fract();
        let (r, g, b) = hsv(hue, 0.55, 0.9);
        let mut rgba = Vec::with_capacity(N * N * 4);
        for y in 0..N {
            for x in 0..N {
                let (dx, dy) = (x as f32 - 63.5, y as f32 - 63.5);
                let d = (dx * dx + dy * dy).sqrt();
                let shade = 1.0 - (d / 90.0).min(1.0) * 0.5;
                if d < 58.0 {
                    rgba.extend([(r * shade) as u8, (g * shade) as u8, (b * shade) as u8, 255]);
                } else {
                    rgba.extend([0, 0, 0, 0]);
                }
            }
        }
        Some(RawIcon {
            width: N,
            height: N,
            rgba,
        })
    }
}

fn hsv(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let (p, q, t) = (v * (1.0 - s), v * (1.0 - s * f), v * (1.0 - s * (1.0 - f)));
    let (r, g, b) = match i as i32 % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    (r * 255.0, g * 255.0, b * 255.0)
}

/// The sample screen: eight equipment rows with their icon tiles and the cache numbers.
pub struct Demo {
    pub cache: IconCache,
}

const ROWS: [(&str, Icon); 8] = [
    ("Sample Bowgun", Icon::Weapon),
    ("Sample Helm", Icon::Head),
    ("Sample Mail", Icon::Chest),
    ("Sample Vambraces", Icon::Arms),
    ("Sample Coil", Icon::Waist),
    ("Sample Greaves", Icon::Legs),
    ("Sample Charm (no icon)", Icon::Charm),
    ("Sample Helm II", Icon::Head),
];

impl Demo {
    pub fn new(ctx: &Context, source: Arc<dyn IconSource>) -> Self {
        Demo {
            cache: IconCache::new(ctx, source),
        }
    }
}

pub fn show(ui: &mut Ui, demo: &mut Demo) {
    demo.cache.poll();
    let screen = ui.max_rect();
    let p = ui.painter_at(screen);
    p.rect_filled(screen, 0.0, BG);
    let mut m = Mesh::default();
    let top = Rect::from_min_size(screen.min, Vec2::new(screen.width(), 150.0));
    for (pos, color) in [
        (top.left_top(), GRADIENT_TOP),
        (top.right_top(), GRADIENT_TOP),
        (top.right_bottom(), BG),
        (top.left_bottom(), BG),
    ] {
        m.vertices.push(Vertex {
            pos,
            uv: WHITE_UV,
            color,
        });
    }
    m.indices.extend([0, 1, 2, 0, 2, 3]);
    p.add(Shape::mesh(m));

    let left = screen.min.x + 28.0;
    let right = screen.max.x - 28.0;
    p.text(
        Pos2::new(left, screen.min.y + 30.0),
        Align2::LEFT_CENTER,
        "ICON PIPELINE \u{b7} SAMPLE",
        font("mono", 10.5),
        DIM,
    );
    p.text(
        Pos2::new(left, screen.min.y + 58.0),
        Align2::LEFT_CENTER,
        "Icons",
        font("inter-medium", 20.0),
        FG,
    );
    p.text(
        Pos2::new(right, screen.min.y + 30.0),
        Align2::RIGHT_CENTER,
        "SAMPLE ICONS, NOT GAME ART",
        font("mono", 10.5),
        WARN,
    );
    p.text(
        Pos2::new(left, screen.min.y + 92.0),
        Align2::LEFT_CENTER,
        "Icons load in the background. Until one arrives, or when there is none, the tile keeps its plain glyph.",
        font("inter", 12.0),
        DIM,
    );

    let mut y = screen.min.y + 130.0;
    for (i, (name, glyph)) in ROWS.iter().enumerate() {
        let id = i as u32;
        if i > 0 {
            p.line_segment(
                [Pos2::new(left, y - 3.0), Pos2::new(right, y - 3.0)],
                Stroke::new(1.0, LINE),
            );
        }
        let row = Rect::from_min_size(Pos2::new(left, y), Vec2::new(right - left, 48.0));
        p.text(
            Pos2::new(left, row.center().y),
            Align2::LEFT_CENTER,
            format!("{:02}", i + 1),
            font("mono", 11.0),
            FAINT,
        );
        let tile = Rect::from_center_size(
            Pos2::new(left + 46.0 + TILE / 2.0, row.center().y),
            Vec2::splat(TILE),
        );
        demo.cache.tile(&p, tile, id, *glyph);
        p.text(
            Pos2::new(tile.right() + 14.0, row.center().y),
            Align2::LEFT_CENTER,
            *name,
            font("inter-medium", 13.5),
            FG,
        );
        let (label, col) = match demo.cache.state(id) {
            IconState::Ready => ("ICON", OK),
            IconState::NotAsked | IconState::Loading => ("LOADING", DIM),
            IconState::Missing => ("NO ICON", WARN),
        };
        p.text(
            Pos2::new(right, row.center().y),
            Align2::RIGHT_CENTER,
            label,
            font("mono", 10.5),
            col,
        );
        y += 54.0;
    }

    let s = demo.cache.stats();
    let body = Rect::from_min_max(Pos2::new(left, y + 8.0), Pos2::new(right, y + 64.0));
    p.rect_stroke(body, 2.0, Stroke::new(1.0, LINE), StrokeKind::Inside);
    p.text(
        body.min + Vec2::new(16.0, 18.0),
        Align2::LEFT_CENTER,
        "CACHE",
        font("mono", 10.5),
        DIM,
    );
    p.text(
        body.min + Vec2::new(16.0, 38.0),
        Align2::LEFT_CENTER,
        format!(
            "{} loaded \u{b7} {} without icon \u{b7} {} loading \u{b7} {} KB of {} KB cap",
            s.ready,
            s.missing,
            s.pending,
            s.bytes / 1024,
            demo.cache.cap / 1024
        ),
        font("mono", 11.0),
        FG,
    );
}

/// Renders the demo with no window or GPU. Keeps its egui context and rasteriser between frames,
/// because the icon textures live in them.
pub struct Headless {
    pub ctx: Context,
    rast: render::Rasteriser,
}

impl Headless {
    pub fn new() -> Self {
        let ctx = Context::default();
        install_fonts(&ctx);
        Headless {
            ctx,
            rast: render::Rasteriser::default(),
        }
    }

    /// One frame of the demo at `size` points and `ppp` pixels per point.
    pub fn frame(&mut self, demo: &mut Demo, size: Vec2, ppp: f32) -> render::Frame {
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
        let mut out = self.ctx.run_ui(input, |ui| show(ui, demo));
        self.rast.apply(std::mem::take(&mut out.textures_delta));
        let prims = self.ctx.tessellate(out.shapes, out.pixels_per_point);
        self.rast.paint(
            &prims,
            out.pixels_per_point,
            (size.x * ppp) as usize,
            (size.y * ppp) as usize,
            BG,
        )
    }
}

impl Default for Headless {
    fn default() -> Self {
        Self::new()
    }
}
