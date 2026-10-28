// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A small CPU rasteriser for egui's tessellated output, so screenshots render in CI and in the
//! cloud with no GPU. It handles what egui produces for this sample: textured, vertex-coloured,
//! clipped triangles, blended in gamma space the way egui's own backends do.

use egui::epaint::{ImageDelta, Primitive};
use egui::{ClippedPrimitive, Color32, TextureId, TexturesDelta};
use std::collections::HashMap;

#[derive(Default)]
pub struct Rasteriser {
    textures: HashMap<TextureId, Texture>,
}

struct Texture {
    size: [usize; 2],
    pixels: Vec<Color32>,
}

pub struct Frame {
    pub width: usize,
    pub height: usize,
    /// Premultiplied RGBA, row by row.
    pub rgba: Vec<[f32; 4]>,
}

impl Rasteriser {
    pub fn apply(&mut self, mut delta: TexturesDelta) {
        for (id, deltas) in &delta.set {
            for d in deltas {
                self.set(*id, d);
            }
        }
        for id in &delta.free {
            self.textures.remove(id);
        }
        delta.clear();
    }

    fn set(&mut self, id: TextureId, d: &ImageDelta) {
        let egui::ImageData::Color(img) = &d.image;
        match d.pos {
            None => {
                self.textures.insert(
                    id,
                    Texture {
                        size: img.size,
                        pixels: img.pixels.clone(),
                    },
                );
            }
            Some([x0, y0]) => {
                let Some(tex) = self.textures.get_mut(&id) else {
                    return;
                };
                for y in 0..img.size[1] {
                    let dst = (y0 + y) * tex.size[0] + x0;
                    let src = y * img.size[0];
                    tex.pixels[dst..dst + img.size[0]]
                        .copy_from_slice(&img.pixels[src..src + img.size[0]]);
                }
            }
        }
    }

    /// Paints `prims` (in points) onto a new frame of `width` x `height` pixels.
    pub fn paint(
        &self,
        prims: &[ClippedPrimitive],
        ppp: f32,
        width: usize,
        height: usize,
        clear: Color32,
    ) -> Frame {
        let c = premul(clear);
        let mut frame = Frame {
            width,
            height,
            rgba: vec![c; width * height],
        };
        for prim in prims {
            let Primitive::Mesh(mesh) = &prim.primitive else {
                continue;
            };
            let Some(tex) = self.textures.get(&mesh.texture_id) else {
                continue;
            };
            let clip = [
                (prim.clip_rect.min.x * ppp).floor().max(0.0) as i32,
                (prim.clip_rect.min.y * ppp).floor().max(0.0) as i32,
                (prim.clip_rect.max.x * ppp).ceil().min(width as f32) as i32,
                (prim.clip_rect.max.y * ppp).ceil().min(height as f32) as i32,
            ];
            for tri in mesh.indices.chunks_exact(3) {
                let v = [
                    mesh.vertices[tri[0] as usize],
                    mesh.vertices[tri[1] as usize],
                    mesh.vertices[tri[2] as usize],
                ];
                draw_triangle(&mut frame, tex, &v, ppp, clip);
            }
        }
        frame
    }
}

fn premul(c: Color32) -> [f32; 4] {
    [
        c.r() as f32 / 255.0,
        c.g() as f32 / 255.0,
        c.b() as f32 / 255.0,
        c.a() as f32 / 255.0,
    ]
}

fn sample(tex: &Texture, u: f32, v: f32) -> [f32; 4] {
    // Bilinear, like the font atlas filter in egui's backends.
    let w = tex.size[0] as i32;
    let h = tex.size[1] as i32;
    let x = u * w as f32 - 0.5;
    let y = v * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let at = |xi: i32, yi: i32| {
        premul(tex.pixels[(yi.clamp(0, h - 1) * w + xi.clamp(0, w - 1)) as usize])
    };
    let (a, b, c, d) = (
        at(x0 as i32, y0 as i32),
        at(x0 as i32 + 1, y0 as i32),
        at(x0 as i32, y0 as i32 + 1),
        at(x0 as i32 + 1, y0 as i32 + 1),
    );
    let mut out = [0.0; 4];
    for i in 0..4 {
        let top = a[i] + (b[i] - a[i]) * fx;
        let bot = c[i] + (d[i] - c[i]) * fx;
        out[i] = top + (bot - top) * fy;
    }
    out
}

fn draw_triangle(
    frame: &mut Frame,
    tex: &Texture,
    v: &[egui::epaint::Vertex; 3],
    ppp: f32,
    clip: [i32; 4],
) {
    let p = |i: usize| (v[i].pos.x * ppp, v[i].pos.y * ppp);
    let (p0, p1, p2) = (p(0), p(1), p(2));
    let area = (p1.0 - p0.0) * (p2.1 - p0.1) - (p2.0 - p0.0) * (p1.1 - p0.1);
    if area.abs() < 1e-6 {
        return;
    }
    let min_x = (p0.0.min(p1.0).min(p2.0).floor() as i32).max(clip[0]);
    let min_y = (p0.1.min(p1.1).min(p2.1).floor() as i32).max(clip[1]);
    let max_x = (p0.0.max(p1.0).max(p2.0).ceil() as i32).min(clip[2]);
    let max_y = (p0.1.max(p1.1).max(p2.1).ceil() as i32).min(clip[3]);
    let cols = [premul(v[0].color), premul(v[1].color), premul(v[2].color)];
    for y in min_y..max_y {
        for x in min_x..max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = ((p1.0 - px) * (p2.1 - py) - (p2.0 - px) * (p1.1 - py)) / area;
            let w1 = ((p2.0 - px) * (p0.1 - py) - (p0.0 - px) * (p2.1 - py)) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let u = w0 * v[0].uv.x + w1 * v[1].uv.x + w2 * v[2].uv.x;
            let t = w0 * v[0].uv.y + w1 * v[1].uv.y + w2 * v[2].uv.y;
            let texel = sample(tex, u, t);
            let mut src = [0.0; 4];
            for i in 0..4 {
                let col = w0 * cols[0][i] + w1 * cols[1][i] + w2 * cols[2][i];
                src[i] = col * texel[i];
            }
            let dst = &mut frame.rgba[y as usize * frame.width + x as usize];
            let inv = 1.0 - src[3];
            for i in 0..4 {
                dst[i] = src[i] + dst[i] * inv;
            }
        }
    }
}

impl Frame {
    pub fn write_png(&self, path: &std::path::Path) -> std::io::Result<()> {
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        let mut enc = png::Encoder::new(file, self.width as u32, self.height as u32);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header()?;
        let data: Vec<u8> = self
            .rgba
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]].map(|c| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8))
            .collect();
        w.write_image_data(&data)?;
        Ok(())
    }
}
