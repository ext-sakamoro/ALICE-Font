//! Noto outline → SDF rasterizer.
//!
//! Looks up the auto-generated `NOTO_OUTLINES` table and rasterises a glyph
//! to a `GlyphSdf` by computing signed distance to the nearest contour edge
//! with even-odd inside test.
//!
//! No runtime TTF loading — the outline data is compiled in as Rust const.
//!
//! Author: Moroya Sakamoto

extern crate alloc;

use super::{GlyphSdf, GLYPH_SDF_SIZE};
use crate::stroke::Point2;

include!("noto_outlines_data.rs");

/// Look up the outline contours for a character. Returns `None` if not present.
#[must_use]
pub fn lookup(ch: char) -> Option<&'static [&'static [(f32, f32)]]> {
    // Binary search by codepoint (the table is generated in sorted order).
    let key = ch;
    NOTO_OUTLINES
        .binary_search_by_key(&key, |(c, _)| *c)
        .ok()
        .map(|i| NOTO_OUTLINES[i].1)
}

/// True iff a Noto outline exists for the given character.
#[must_use]
pub fn has(ch: char) -> bool {
    lookup(ch).is_some()
}

/// Rasterise the Noto outline of `ch` into a 32×32 SDF.
///
/// Returns `None` if no outline data is present for the character.
#[must_use]
pub fn rasterize(ch: char) -> Option<GlyphSdf> {
    let contours = lookup(ch)?;
    Some(rasterize_contours(contours))
}

/// Rasterise an arbitrary list of contours (normalised to `[0, 1]²`).
fn rasterize_contours(contours: &[&[(f32, f32)]]) -> GlyphSdf {
    let mut sdf = GlyphSdf::empty();
    let size = GLYPH_SDF_SIZE;
    let inv = 1.0 / (size - 1) as f32;
    sdf.advance = 1.0;
    sdf.bbox_min = Point2::new(0.0, 0.0);
    sdf.bbox_max = Point2::new(1.0, 1.0);

    for py in 0..size {
        for px in 0..size {
            // (u, v) ∈ [0, 1]². v=0 = bottom row (glyph baseline). Sample
            // in glyph-world coordinates so we match the y-up convention used
            // when normalising the source outline.
            let u = px as f32 * inv;
            let v = py as f32 * inv;
            let inside = point_in_polygons(contours, u, v);
            let dist = nearest_edge_distance(contours, u, v);
            // Signed: negative inside, positive outside.
            let signed = if inside { -dist } else { dist };
            sdf.data[py * size + px] = signed;
        }
    }
    sdf
}

/// Even-odd inside test across all contours.
fn point_in_polygons(contours: &[&[(f32, f32)]], px: f32, py: f32) -> bool {
    let mut inside = false;
    for c in contours {
        if c.len() < 3 {
            continue;
        }
        for w in c.windows(2) {
            let (x0, y0) = w[0];
            let (x1, y1) = w[1];
            if (y0 > py) != (y1 > py) {
                let dx = x1 - x0;
                let dy = y1 - y0;
                let t = (py - y0) / dy;
                let x_cross = x0 + dx * t;
                if px < x_cross {
                    inside = !inside;
                }
            }
        }
    }
    inside
}

/// Squared distance from point (px, py) to nearest contour edge.
fn nearest_edge_distance(contours: &[&[(f32, f32)]], px: f32, py: f32) -> f32 {
    let mut best_sq = f32::MAX;
    for c in contours {
        for w in c.windows(2) {
            let (x0, y0) = w[0];
            let (x1, y1) = w[1];
            let d_sq = point_seg_dist_sq(px, py, x0, y0, x1, y1);
            if d_sq < best_sq {
                best_sq = d_sq;
            }
        }
    }
    best_sq.sqrt()
}

#[inline]
fn point_seg_dist_sq(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len_sq = dx * dx + dy * dy;
    let t = if len_sq < 1e-9 {
        0.0
    } else {
        (((px - x0) * dx + (py - y0) * dy) / len_sq).clamp(0.0, 1.0)
    };
    let nx = x0 + t * dx;
    let ny = y0 + t * dy;
    let ex = px - nx;
    let ey = py - ny;
    ex * ex + ey * ey
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_entries() {
        assert!(NOTO_OUTLINES.len() > 100);
    }

    #[test]
    fn table_is_sorted() {
        for w in NOTO_OUTLINES.windows(2) {
            assert!(w[0].0 < w[1].0, "noto outline table is not sorted");
        }
    }

    #[test]
    fn lookup_present_chars() {
        // 株 should be in the extoria charset.
        assert!(has('株'));
        // 「あ」 is hiragana, should be there.
        assert!(has('あ'));
    }

    #[test]
    fn rasterize_kabu_has_inside_pixels() {
        let sdf = rasterize('株').expect("株 in table");
        assert!(
            sdf.data.iter().any(|d| *d < 0.0),
            "株 should have inside pixels"
        );
    }
}
