//! BIZ UDPGothic outline → SDF rasterizer.
//!
//! Rasterizes glyphs from em-normalized outline data (imported from
//! BIZ UDPGothic via tools/import-font) into a `GLYPH_SDF_SIZE`² signed
//! distance field. Uses even-odd fill rule for inside test and nearest-edge
//! distance for magnitude. Signed: negative inside, positive outside.
//!
//! Two weight tables are consulted:
//!   - FONT_OUTLINES_REGULAR — used when `params.weight < 0.6`
//!   - FONT_OUTLINES_BOLD    — used when `params.weight >= 0.6`
//!
//! No runtime TTF loading — the outline data is compiled in as Rust const.
//!
//! License: outline data is derived from BIZ UDPGothic (Morisawa, SIL OFL 1.1).
//! See `tools/import-font/fonts/OFL.txt` for terms.
//!
//! Author: Moroya Sakamoto

extern crate alloc;

use super::{GlyphSdf, GLYPH_SDF_SIZE};
use crate::param::MetaFontParams;
use crate::stroke::Point2;

include!("font_outlines_data.rs");

/// Tile em box — fixed across all glyphs so proportional sizes are consistent.
/// The tile spans em [`TILE_EM_LEFT`, `TILE_EM_RIGHT`] × [`TILE_EM_BOTTOM`, `TILE_EM_TOP`].
/// Baseline is at em y=0 (about 20% up from tile bottom).
const TILE_EM_LEFT: f32 = 0.0;
const TILE_EM_RIGHT: f32 = 1.0;
const TILE_EM_BOTTOM: f32 = -0.2;
const TILE_EM_TOP: f32 = 0.8;

/// Weight threshold: strokes >= 0.6 use the Bold table.
const BOLD_WEIGHT_THRESHOLD: f32 = 0.6;

/// Look up outline + advance for a character. Returns `None` if not present.
#[must_use]
pub fn lookup(ch: char, bold: bool) -> Option<(&'static [&'static [(f32, f32)]], f32)> {
    let table = if bold {
        FONT_OUTLINES_BOLD
    } else {
        FONT_OUTLINES_REGULAR
    };
    table
        .binary_search_by_key(&ch, |(c, _, _)| *c)
        .ok()
        .map(|i| (table[i].1, table[i].2))
}

/// True iff an outline exists for the given character in the Regular table.
#[must_use]
pub fn has(ch: char) -> bool {
    FONT_OUTLINES_REGULAR
        .binary_search_by_key(&ch, |(c, _, _)| *c)
        .is_ok()
}

/// Rasterize the outline of `ch` into an SDF for the given font parameters.
///
/// Returns `None` if no outline data is present for the character. Uses the
/// Bold table when `params.weight >= 0.6`, otherwise the Regular table.
#[must_use]
pub fn rasterize(ch: char, params: &MetaFontParams) -> Option<GlyphSdf> {
    let bold = params.weight >= BOLD_WEIGHT_THRESHOLD;
    let (contours, advance) = lookup(ch, bold)?;
    if contours.is_empty() {
        // Space or non-drawing glyph: carry the advance only.
        let mut sdf = GlyphSdf::empty();
        sdf.advance = advance;
        return Some(sdf);
    }
    Some(rasterize_from_contours(contours, advance))
}

/// Rasterize outline contours (em-normalized) into a `GLYPH_SDF_SIZE`² SDF.
///
/// Signed distance: negative inside the glyph, positive outside. Inside/outside
/// is determined by the even-odd fill rule. Magnitude is the Euclidean distance
/// to the nearest contour edge.
///
/// This is the shared rasterization core used by both the embedded BIZ UDPGothic
/// table (via [`rasterize`]) and external tools that parse arbitrary TTF outlines
/// at build time (e.g. `extoria-website-sdf-v2/tools/gen-atlas-multi`).
///
/// # Coordinate convention
///
/// Contours are em-normalized: `units_per_em -> 1.0`, TTF y-up preserved,
/// x in `[0, ~advance_em]`, y in `[-descent_em, ascent_em]`. Points inside
/// the em box `[TILE_EM_LEFT, TILE_EM_RIGHT] × [TILE_EM_BOTTOM, TILE_EM_TOP]`
/// (i.e. `[0, 1] × [-0.2, 0.8]`) get sampled into the output SDF.
#[must_use]
pub fn rasterize_from_contours(contours: &[&[(f32, f32)]], advance: f32) -> GlyphSdf {
    let mut sdf = GlyphSdf::empty();
    sdf.advance = advance;
    sdf.bbox_min = Point2::new(TILE_EM_LEFT, TILE_EM_BOTTOM);
    sdf.bbox_max = Point2::new(TILE_EM_RIGHT, TILE_EM_TOP);

    let size = GLYPH_SDF_SIZE;
    let em_w = TILE_EM_RIGHT - TILE_EM_LEFT;
    let em_h = TILE_EM_TOP - TILE_EM_BOTTOM;
    let inv_size_1 = 1.0 / (size - 1) as f32;

    for py in 0..size {
        for px in 0..size {
            let u = px as f32 * inv_size_1;
            let v = py as f32 * inv_size_1;
            let em_x = TILE_EM_LEFT + u * em_w;
            let em_y = TILE_EM_BOTTOM + v * em_h;

            let inside = point_in_polygons(contours, em_x, em_y);
            let dist_sq = nearest_edge_distance_sq(contours, em_x, em_y);
            let dist = fast_sqrt(dist_sq);
            sdf.data[py * size + px] = if inside { -dist } else { dist };
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
                let dy = y1 - y0;
                if dy.abs() > 1e-9 {
                    let t = (py - y0) / dy;
                    let x_cross = x0 + (x1 - x0) * t;
                    if px < x_cross {
                        inside = !inside;
                    }
                }
            }
        }
    }
    inside
}

/// Squared distance from point to nearest contour edge.
fn nearest_edge_distance_sq(contours: &[&[(f32, f32)]], px: f32, py: f32) -> f32 {
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
    best_sq
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

/// Quake III fast inverse square root, then multiply by `x` to get `sqrt(x)`.
#[inline(always)]
fn fast_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let half = 0.5 * x;
    let i = f32::to_bits(x);
    let i = 0x5f37_59df - (i >> 1);
    let y = f32::from_bits(i);
    let y = y * (1.5 - half * y * y);
    let y = y * (1.5 - half * y * y);
    x * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_have_entries() {
        assert!(FONT_OUTLINES_REGULAR.len() > 100);
        assert!(FONT_OUTLINES_BOLD.len() > 100);
    }

    #[test]
    fn tables_are_sorted() {
        for w in FONT_OUTLINES_REGULAR.windows(2) {
            assert!(w[0].0 < w[1].0, "regular table is not sorted");
        }
        for w in FONT_OUTLINES_BOLD.windows(2) {
            assert!(w[0].0 < w[1].0, "bold table is not sorted");
        }
    }

    #[test]
    fn lookup_ascii() {
        assert!(has('A'));
        assert!(has('a'));
        assert!(has('0'));
    }

    #[test]
    fn lookup_kanji() {
        assert!(has('株'));
        assert!(has('式'));
    }

    #[test]
    fn lookup_hiragana() {
        assert!(has('あ'));
        assert!(has('ん'));
    }

    #[test]
    fn rasterize_a_has_inside_pixels() {
        let sdf = rasterize('A', &MetaFontParams::sans_regular()).expect("A in table");
        assert!(
            sdf.data.iter().any(|d| *d < 0.0),
            "'A' should have inside pixels"
        );
    }

    #[test]
    fn rasterize_kabu_has_inside_pixels() {
        let sdf = rasterize('株', &MetaFontParams::sans_regular()).expect("株 in table");
        assert!(
            sdf.data.iter().any(|d| *d < 0.0),
            "'株' should have inside pixels"
        );
    }

    #[test]
    fn rasterize_space_has_advance() {
        let sdf = rasterize(' ', &MetaFontParams::sans_regular()).expect("space in table");
        assert!(sdf.advance > 0.0, "space should carry positive advance");
    }

    #[test]
    fn bold_fills_more_than_regular() {
        let sdf_regular = rasterize('B', &MetaFontParams::sans_regular()).unwrap();
        let sdf_bold = rasterize('B', &MetaFontParams::sans_bold()).unwrap();
        let inside_r = sdf_regular.data.iter().filter(|d| **d < 0.0).count();
        let inside_b = sdf_bold.data.iter().filter(|d| **d < 0.0).count();
        assert!(
            inside_b >= inside_r,
            "bold 'B' should fill >= regular ('bold'={inside_b}, 'regular'={inside_r})"
        );
    }
}
