//! Outline → [`GlyphSdf`]: the exact signed distance to the filled region of an [`OutlineGlyph`].
//!
//! The filled region follows the **non-zero winding** rule (OpenType). The distance is measured to
//! the boundary of that region, not to every contour edge: where contours overlap (CJK glyphs
//! often draw strokes as separate overlapping contours), the edges that lie inside the region are
//! not part of its boundary and do not take part.
//!
//! # How
//!
//! 1. Every contour edge is cut at every point where another edge meets it, so no piece is crossed
//!    in its interior.
//! 2. A piece is on the boundary when the winding number is zero on exactly one of its two sides
//!    (tested just beside its midpoint, closer than any other edge).
//! 3. Each sample is the distance to the nearest boundary piece, negative where the winding number
//!    is non-zero.
//!
//! All arithmetic is `f64` `+ - * /` and [`alice_det_math::ops::sqrt64`], each correctly rounded,
//! so the output is bit-identical on every platform. Values are in em units, cast to `f32` once.
//!
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

use super::OutlineGlyph;
use crate::glyph::{GlyphSdf, GLYPH_SDF_SIZE};
use crate::stroke::Point2;

/// Left edge of the tile, em
pub const TILE_EM_LEFT: f32 = 0.0;
/// Right edge of the tile, em
pub const TILE_EM_RIGHT: f32 = 1.0;
/// Bottom edge of the tile, em (the baseline is em y = 0)
pub const TILE_EM_BOTTOM: f32 = -0.2;
/// Top edge of the tile, em
pub const TILE_EM_TOP: f32 = 0.9;

/// Two values closer than this (em) are the same point
const EPS: f64 = 1e-12;

#[derive(Clone, Copy)]
struct Seg {
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
}

/// Rasterise an outline into a `GLYPH_SDF_SIZE`² signed distance field over the tile
/// [`TILE_EM_LEFT`, `TILE_EM_RIGHT`] × [`TILE_EM_BOTTOM`, `TILE_EM_TOP`].
///
/// Sample `(px, py)` is taken at the centre of its texel,
/// `x = LEFT + (px + 0.5) / N · (RIGHT − LEFT)`, `y = BOTTOM + (py + 0.5) / N · (TOP − BOTTOM)`;
/// row 0 is the bottom. Each contour is closed implicitly (an explicit closing point equal to the
/// first is also accepted). Negative inside, positive outside, in em. An outline with no filled
/// region gives every sample `1.0`, as [`GlyphSdf::empty`].
#[must_use]
pub fn rasterize(glyph: &OutlineGlyph) -> GlyphSdf {
    let mut sdf = GlyphSdf::empty();
    sdf.advance = glyph.advance;
    sdf.bbox_min = Point2::new(TILE_EM_LEFT, TILE_EM_BOTTOM);
    sdf.bbox_max = Point2::new(TILE_EM_RIGHT, TILE_EM_TOP);

    let segs = segments(glyph);
    let boundary = boundary_pieces(&segs);
    if boundary.is_empty() {
        return sdf;
    }

    let n = GLYPH_SDF_SIZE as f64;
    let left = f64::from(TILE_EM_LEFT);
    let bottom = f64::from(TILE_EM_BOTTOM);
    let w = f64::from(TILE_EM_RIGHT) - left;
    let h = f64::from(TILE_EM_TOP) - bottom;
    for py in 0..GLYPH_SDF_SIZE {
        let y = bottom + (py as f64 + 0.5) / n * h;
        for px in 0..GLYPH_SDF_SIZE {
            let x = left + (px as f64 + 0.5) / n * w;
            let d2 = boundary
                .iter()
                .map(|s| dist2(s, x, y))
                .fold(f64::INFINITY, f64::min);
            let d = alice_det_math::ops::sqrt64(d2);
            let signed = if winding(&segs, x, y) != 0 { -d } else { d };
            sdf.data[py * GLYPH_SDF_SIZE + px] = signed as f32;
        }
    }
    sdf
}

/// The edges of all contours (each closed), without zero-length edges
fn segments(glyph: &OutlineGlyph) -> Vec<Seg> {
    let mut out = Vec::new();
    for c in &glyph.contours {
        if c.len() < 2 {
            continue;
        }
        for i in 0..c.len() {
            let a = c[i];
            let b = c[(i + 1) % c.len()];
            let s = Seg {
                ax: f64::from(a[0]),
                ay: f64::from(a[1]),
                bx: f64::from(b[0]),
                by: f64::from(b[1]),
            };
            if s.ax != s.bx || s.ay != s.by {
                out.push(s);
            }
        }
    }
    out
}

/// Non-zero winding number at `(x, y)`: a ray toward +x, each edge crossing it upward counts +1
/// and downward −1, half-open in y so a shared vertex counts once
fn winding(segs: &[Seg], x: f64, y: f64) -> i32 {
    let mut w = 0;
    for s in segs {
        let up = s.ay <= y && s.by > y;
        let down = s.by <= y && s.ay > y;
        if up || down {
            let t = (y - s.ay) / (s.by - s.ay);
            let xc = s.ax + (s.bx - s.ax) * t;
            if x < xc {
                w += if up { 1 } else { -1 };
            }
        }
    }
    w
}

fn cross(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ax * by - ay * bx
}

/// Squared distance from `(x, y)` to segment `s`
fn dist2(s: &Seg, x: f64, y: f64) -> f64 {
    let dx = s.bx - s.ax;
    let dy = s.by - s.ay;
    let len2 = dx * dx + dy * dy;
    let t = (((x - s.ax) * dx + (y - s.ay) * dy) / len2).clamp(0.0, 1.0);
    let ex = x - (s.ax + dx * t);
    let ey = y - (s.ay + dy * t);
    ex * ex + ey * ey
}

/// Parameters in (0, 1) where `other` meets `s`: a crossing, an endpoint of one on the other, or
/// the ends of a collinear overlap
fn cuts(s: &Seg, other: &Seg, out: &mut Vec<f64>) {
    let dx = s.bx - s.ax;
    let dy = s.by - s.ay;
    let ex = other.bx - other.ax;
    let ey = other.by - other.ay;
    let cx = other.ax - s.ax;
    let cy = other.ay - s.ay;
    let len2 = dx * dx + dy * dy;
    let denom = cross(dx, dy, ex, ey);
    let mut push = |t: f64| {
        if t > EPS && t < 1.0 - EPS {
            out.push(t);
        }
    };
    if denom.abs() > EPS * EPS {
        let t = cross(cx, cy, ex, ey) / denom;
        let u = cross(cx, cy, dx, dy) / denom;
        if (-EPS..=1.0 + EPS).contains(&u) {
            push(t);
        }
    } else {
        // parallel: only a collinear overlap cuts, at the other's ends
        let len = alice_det_math::ops::sqrt64(len2);
        if (cross(cx, cy, dx, dy) / len).abs() <= EPS {
            push((cx * dx + cy * dy) / len2);
            push(((cx + ex) * dx + (cy + ey) * dy) / len2);
        }
    }
}

/// The pieces of the edges that lie on the boundary of the filled region
fn boundary_pieces(segs: &[Seg]) -> Vec<Seg> {
    let mut out = Vec::new();
    let mut ts = Vec::new();
    for (i, s) in segs.iter().enumerate() {
        ts.clear();
        ts.push(0.0);
        ts.push(1.0);
        for (j, o) in segs.iter().enumerate() {
            if i != j {
                cuts(s, o, &mut ts);
            }
        }
        ts.sort_by(f64::total_cmp);
        ts.dedup_by(|b, a| *b - *a <= EPS);

        let dx = s.bx - s.ax;
        let dy = s.by - s.ay;
        let len = alice_det_math::ops::sqrt64(dx * dx + dy * dy);
        // unit normal to the left of the edge
        let nx = -dy / len;
        let ny = dx / len;
        for k in 1..ts.len() {
            let (t0, t1) = (ts[k - 1], ts[k]);
            let piece = Seg {
                ax: s.ax + dx * t0,
                ay: s.ay + dy * t0,
                bx: s.ax + dx * t1,
                by: s.ay + dy * t1,
            };
            let tm = (t0 + t1) * 0.5;
            let mx = s.ax + dx * tm;
            let my = s.ay + dy * tm;
            // step aside less than the distance to any edge not through the midpoint (the half
            // length only keeps the step finite when no other edge exists; any bound would do)
            let mut reach = (t1 - t0) * len * 0.5;
            for o in segs {
                let d2 = dist2(o, mx, my);
                if d2 > EPS * EPS {
                    reach = reach.min(alice_det_math::ops::sqrt64(d2));
                }
            }
            let e = reach * 0.25;
            let left = winding(segs, mx + nx * e, my + ny * e) != 0;
            let right = winding(segs, mx - nx * e, my - ny * e) != 0;
            if left != right {
                out.push(piece);
            }
        }
    }
    out
}
