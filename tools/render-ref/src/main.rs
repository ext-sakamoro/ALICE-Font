//! Reference glyph renderer — rasterizes a real TTF font (Noto Sans JP) into
//! 256×256 PNGs so the ALICE-Font parametric stroke definitions can be tuned
//! by visual comparison.
//!
//! NOT used at runtime by ALICE-Font itself. The output PNGs are
//! developer reference material.
//!
//! Usage:
//!   cargo run --release -- 株 式 朱 弋
//!
//! Output: tools/render-ref/out/{char}.png
//!
//! Author: Moroya Sakamoto

use image::{ImageBuffer, Luma};
use std::path::PathBuf;
use ttf_parser::{Face, OutlineBuilder};

const SIZE: u32 = 256;
const MARGIN: f32 = 0.08; // 8% margin around glyph

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        // Default: extoria hero + a few extras.
        args = "株 式 会 社 エ ク ス ト ー リ ア 朱 弋 開 発 ホ ス ジ ネ"
            .split_whitespace()
            .map(String::from)
            .collect();
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let font_path = manifest.join("data").join("NotoSansJP-Regular.ttf");
    let font_data = std::fs::read(&font_path)?;
    let face = Face::parse(&font_data, 0)?;
    println!("Loaded {} ({} glyphs)", font_path.display(), face.number_of_glyphs());

    let out_dir = manifest.join("out");
    std::fs::create_dir_all(&out_dir)?;

    for token in &args {
        let ch = token.chars().next().expect("non-empty arg");
        let img = render_char(&face, ch);
        let path = out_dir.join(format!("{ch}.png"));
        img.save(&path)?;
        println!("  wrote {}", path.display());
    }
    Ok(())
}

fn render_char(face: &Face, ch: char) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let mut img = ImageBuffer::<Luma<u8>, Vec<u8>>::from_pixel(SIZE, SIZE, Luma([0]));
    let Some(glyph_id) = face.glyph_index(ch) else {
        eprintln!("  glyph not found for U+{:04X}", ch as u32);
        return img;
    };

    let mut builder = PathBuilder::new();
    if face.outline_glyph(glyph_id, &mut builder).is_none() {
        eprintln!("  no outline for U+{:04X}", ch as u32);
        return img;
    }
    let bbox = builder.bbox();
    if bbox.is_none() {
        return img;
    }
    let (gx0, gy0, gx1, gy1) = bbox.unwrap();
    let gw = gx1 - gx0;
    let gh = gy1 - gy0;
    if gw <= 0.0 || gh <= 0.0 {
        return img;
    }

    // Fit glyph into image with margin, preserving aspect ratio.
    let inner = SIZE as f32 * (1.0 - 2.0 * MARGIN);
    let scale = (inner / gw.max(gh)).min(inner / gw).min(inner / gh);
    let ox = (SIZE as f32 - gw * scale) * 0.5;
    let oy = (SIZE as f32 - gh * scale) * 0.5;

    // Rasterize by even-odd rule on each scanline.
    for py in 0..SIZE {
        // Transform pixel y to glyph y (note: ttf-parser uses y-up; image is y-down).
        let py_f = py as f32 + 0.5;
        let gy = gy1 - ((py_f - oy) / scale);
        let mut crossings = builder.scanline_crossings(gy);
        crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for win in crossings.chunks(2) {
            if win.len() < 2 { break; }
            let gx_l = win[0];
            let gx_r = win[1];
            let px_l = (gx_l - gx0) * scale + ox;
            let px_r = (gx_r - gx0) * scale + ox;
            let pxa = px_l.max(0.0).min(SIZE as f32 - 1.0) as u32;
            let pxb = px_r.max(0.0).min(SIZE as f32 - 1.0) as u32;
            for px in pxa..=pxb {
                img.put_pixel(px, py, Luma([255]));
            }
        }
    }
    img
}

/// Accumulates glyph outline contours and provides a simple scanline
/// crossings query — enough for a coarse rasterization for visual reference.
struct PathBuilder {
    contours: Vec<Vec<(f32, f32)>>,
    cur: Vec<(f32, f32)>,
    last: (f32, f32),
}

impl PathBuilder {
    fn new() -> Self {
        Self { contours: Vec::new(), cur: Vec::new(), last: (0.0, 0.0) }
    }

    fn bbox(&self) -> Option<(f32, f32, f32, f32)> {
        let mut min = (f32::MAX, f32::MAX);
        let mut max = (f32::MIN, f32::MIN);
        let mut found = false;
        for c in &self.contours {
            for p in c {
                min.0 = min.0.min(p.0);
                min.1 = min.1.min(p.1);
                max.0 = max.0.max(p.0);
                max.1 = max.1.max(p.1);
                found = true;
            }
        }
        if found {
            Some((min.0, min.1, max.0, max.1))
        } else {
            None
        }
    }

    /// Return x coordinates where horizontal line y crosses any contour edge.
    fn scanline_crossings(&self, y: f32) -> Vec<f32> {
        let mut out = Vec::new();
        for c in &self.contours {
            for w in c.windows(2) {
                let p0 = w[0];
                let p1 = w[1];
                if (p0.1 <= y && p1.1 > y) || (p1.1 <= y && p0.1 > y) {
                    let t = (y - p0.1) / (p1.1 - p0.1);
                    let x = p0.0 + t * (p1.0 - p0.0);
                    out.push(x);
                }
            }
        }
        out
    }

    fn flush(&mut self) {
        if !self.cur.is_empty() {
            let c = std::mem::take(&mut self.cur);
            self.contours.push(c);
        }
    }

    fn subdivide_quad(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), n: usize) -> Vec<(f32, f32)> {
        let mut out = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let x = u * u * p0.0 + 2.0 * u * t * p1.0 + t * t * p2.0;
            let y = u * u * p0.1 + 2.0 * u * t * p1.1 + t * t * p2.1;
            out.push((x, y));
        }
        out
    }

    fn subdivide_cubic(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), n: usize) -> Vec<(f32, f32)> {
        let mut out = Vec::with_capacity(n + 1);
        for i in 0..=n {
            let t = i as f32 / n as f32;
            let u = 1.0 - t;
            let x = u*u*u*p0.0 + 3.0*u*u*t*p1.0 + 3.0*u*t*t*p2.0 + t*t*t*p3.0;
            let y = u*u*u*p0.1 + 3.0*u*u*t*p1.1 + 3.0*u*t*t*p2.1 + t*t*t*p3.1;
            out.push((x, y));
        }
        out
    }
}

impl OutlineBuilder for PathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        self.cur.push((x, y));
        self.last = (x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.cur.push((x, y));
        self.last = (x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let pts = self.subdivide_quad(self.last, (x1, y1), (x, y), 16);
        for p in pts.into_iter().skip(1) {
            self.cur.push(p);
        }
        self.last = (x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let pts = self.subdivide_cubic(self.last, (x1, y1), (x2, y2), (x, y), 24);
        for p in pts.into_iter().skip(1) {
            self.cur.push(p);
        }
        self.last = (x, y);
    }
    fn close(&mut self) {
        if let Some(&first) = self.cur.first() {
            self.cur.push(first);
        }
        self.flush();
    }
}
