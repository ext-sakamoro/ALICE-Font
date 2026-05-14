//! Hiragana glyph rendering — U+3040 to U+309F
//!
//! S3 implementation. The five vowels (あいうえお) are wired up with
//! parametric stroke skeletons; the remaining 80 characters fall through
//! to a placeholder until later sub-steps of S3 land. See
//! `docs/CJK_KANA_SPEC.md` for the full specification.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::glyph::cjk_strokes::{add_cjk_stroke, CjkStrokeType, StrokePlacement};
use crate::glyph::{GlyphGenerator, GlyphSdf, GlyphSkeleton};
use crate::param::MetaFontParams;
use crate::stroke::{Point2, Stroke};

/// Standard advance for a full-width kana glyph (em units).
const KANA_ADVANCE: f32 = 1.0;

/// Generate the SDF for a hiragana character.
///
/// Falls back to an empty placeholder for any character not yet covered.
#[must_use]
pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    match build_skeleton(ch) {
        Some(skel) => {
            let gen = GlyphGenerator::new(params);
            gen.generate_from_skeleton(&skel)
        }
        None => {
            let mut sdf = GlyphSdf::empty();
            sdf.advance = KANA_ADVANCE;
            sdf
        }
    }
}

/// Build the stroke skeleton for a hiragana character. Returns `None` for
/// characters that aren't implemented yet (so the caller can render a
/// placeholder rather than panic).
fn build_skeleton(ch: char) -> Option<GlyphSkeleton> {
    match ch {
        'あ' => Some(build_a()),
        'い' => Some(build_i()),
        'う' => Some(build_u()),
        'え' => Some(build_e()),
        'お' => Some(build_o()),
        _ => None,
    }
}

// ----------------------------------------------------------------------------
// あ (U+3042) — 3 strokes: top horizontal, vertical-curved, lower loop
// ----------------------------------------------------------------------------
fn build_a() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top horizontal stroke with slight upward tilt.
    skel.add_stroke(Stroke::line(
        Point2::new(0.15, 0.72),
        Point2::new(0.85, 0.75),
    ));
    // 2: Vertical-curved spine that crosses the horizontal and exits lower-right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.55, 0.9),
        Point2::new(0.5, 0.6),
        Point2::new(0.5, 0.3),
        Point2::new(0.62, 0.05),
    ));
    // 3: Lower body — closed loop. Uses 4 stroke slots.
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.18, 0.1, 0.55, 0.45),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// い (U+3044) — 2 strokes: left long, right short
// ----------------------------------------------------------------------------
fn build_i() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Left stroke — vertical curve that bows slightly to the right and
    //    finishes with a hook to the lower-right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.22, 0.78),
        Point2::new(0.18, 0.5),
        Point2::new(0.2, 0.25),
        Point2::new(0.34, 0.1),
    ));
    // 2: Right shorter stroke.
    skel.add_stroke(Stroke::new(
        Point2::new(0.72, 0.62),
        Point2::new(0.74, 0.5),
        Point2::new(0.72, 0.38),
        Point2::new(0.66, 0.18),
    ));
    skel
}

// ----------------------------------------------------------------------------
// う (U+3046) — 2 strokes: top tick, lower open curve
// ----------------------------------------------------------------------------
fn build_u() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top short horizontal stroke.
    skel.add_stroke(Stroke::line(Point2::new(0.4, 0.82), Point2::new(0.6, 0.82)));
    // 2: Large open curve below — flat-bottom U shape.
    skel.add_stroke(Stroke::new(
        Point2::new(0.22, 0.58),
        Point2::new(0.4, 0.1),
        Point2::new(0.7, 0.1),
        Point2::new(0.85, 0.4),
    ));
    skel
}

// ----------------------------------------------------------------------------
// え (U+3048) — 2 strokes (top tick + folded body)
// ----------------------------------------------------------------------------
fn build_e() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top short tick.
    skel.add_stroke(Stroke::line(
        Point2::new(0.42, 0.82),
        Point2::new(0.55, 0.82),
    ));
    // 2a: Upper diagonal — comes in from top-left and folds toward the centre.
    skel.add_stroke(Stroke::new(
        Point2::new(0.28, 0.62),
        Point2::new(0.4, 0.55),
        Point2::new(0.5, 0.45),
        Point2::new(0.45, 0.35),
    ));
    // 2b: Lower-left diagonal.
    skel.add_stroke(Stroke::line(
        Point2::new(0.45, 0.35),
        Point2::new(0.22, 0.13),
    ));
    // 2c: Bottom sweep with curved exit.
    skel.add_stroke(Stroke::new(
        Point2::new(0.22, 0.13),
        Point2::new(0.4, 0.18),
        Point2::new(0.65, 0.15),
        Point2::new(0.86, 0.2),
    ));
    skel
}

// ----------------------------------------------------------------------------
// お (U+304A) — 3 strokes: top horizontal, vertical+hook, side accent
// ----------------------------------------------------------------------------
fn build_o() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top horizontal stroke.
    skel.add_stroke(Stroke::line(
        Point2::new(0.18, 0.7),
        Point2::new(0.75, 0.72),
    ));
    // 2a: Long vertical stem crossing the horizontal.
    skel.add_stroke(Stroke::line(Point2::new(0.42, 0.9), Point2::new(0.4, 0.22)));
    // 2b: Hook curling leftward at the bottom of the stem.
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.22),
        Point2::new(0.32, 0.08),
        Point2::new(0.2, 0.08),
        Point2::new(0.18, 0.2),
    ));
    // 3: Small accent on the right side.
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.52),
        Point2::new(0.88, 0.42),
        Point2::new(0.82, 0.3),
        Point2::new(0.72, 0.34),
    ));
    skel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_returns_empty_with_advance_for_unimplemented() {
        let sdf = generate('か', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn vowel_glyphs_have_inside_pixels() {
        let params = MetaFontParams::sans_regular();
        for ch in ['あ', 'い', 'う', 'え', 'お'] {
            let sdf = generate(ch, &params);
            assert!(sdf.advance > 0.0, "{ch}: advance should be positive");
            let inside = sdf.data.iter().any(|d| *d < 0.0);
            assert!(inside, "{ch}: rasterized glyph should have inside pixels");
            for d in sdf.data {
                assert!(d.is_finite(), "{ch}: SDF must be finite");
            }
        }
    }

    #[test]
    fn vowel_glyphs_bbox_in_unit_square() {
        let params = MetaFontParams::sans_regular();
        for ch in ['あ', 'い', 'う', 'え', 'お'] {
            let sdf = generate(ch, &params);
            assert!(
                sdf.bbox_min.x > -0.5 && sdf.bbox_max.x < 1.5,
                "{ch}: bbox x out of range"
            );
            assert!(
                sdf.bbox_min.y > -0.5 && sdf.bbox_max.y < 1.5,
                "{ch}: bbox y out of range"
            );
        }
    }
}
