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
        'か' => Some(build_ka()),
        'き' => Some(build_ki()),
        'く' => Some(build_ku()),
        'け' => Some(build_ke()),
        'こ' => Some(build_ko()),
        'さ' => Some(build_sa()),
        'し' => Some(build_shi()),
        'す' => Some(build_su()),
        'せ' => Some(build_se()),
        'そ' => Some(build_so()),
        'た' => Some(build_ta()),
        'ち' => Some(build_chi()),
        'つ' => Some(build_tsu()),
        'て' => Some(build_te()),
        'と' => Some(build_to()),
        'な' => Some(build_na()),
        'に' => Some(build_ni()),
        'ぬ' => Some(build_nu()),
        'ね' => Some(build_ne()),
        'の' => Some(build_no()),
        'は' => Some(build_ha()),
        'ひ' => Some(build_hi()),
        'ふ' => Some(build_fu()),
        'へ' => Some(build_he()),
        'ほ' => Some(build_ho()),
        'ま' => Some(build_ma()),
        'み' => Some(build_mi()),
        'む' => Some(build_mu()),
        'め' => Some(build_me()),
        'も' => Some(build_mo()),
        'や' => Some(build_ya()),
        'ゆ' => Some(build_yu()),
        'よ' => Some(build_yo()),
        'ら' => Some(build_ra()),
        'り' => Some(build_ri()),
        'る' => Some(build_ru()),
        'れ' => Some(build_re()),
        'ろ' => Some(build_ro()),
        'わ' => Some(build_wa()),
        'を' => Some(build_wo()),
        'ん' => Some(build_n()),
        // Dakuten (濁音) — base character + 「゛」 mark
        'が' => with_dakuten('か'),
        'ぎ' => with_dakuten('き'),
        'ぐ' => with_dakuten('く'),
        'げ' => with_dakuten('け'),
        'ご' => with_dakuten('こ'),
        'ざ' => with_dakuten('さ'),
        'じ' => with_dakuten('し'),
        'ず' => with_dakuten('す'),
        'ぜ' => with_dakuten('せ'),
        'ぞ' => with_dakuten('そ'),
        'だ' => with_dakuten('た'),
        'ぢ' => with_dakuten('ち'),
        'づ' => with_dakuten('つ'),
        'で' => with_dakuten('て'),
        'ど' => with_dakuten('と'),
        'ば' => with_dakuten('は'),
        'び' => with_dakuten('ひ'),
        'ぶ' => with_dakuten('ふ'),
        'べ' => with_dakuten('へ'),
        'ぼ' => with_dakuten('ほ'),
        'ゔ' => with_dakuten('う'),
        // Handakuten (半濁音) — base character + 「゜」 mark
        'ぱ' => with_handakuten('は'),
        'ぴ' => with_handakuten('ひ'),
        'ぷ' => with_handakuten('ふ'),
        'ぺ' => with_handakuten('へ'),
        'ぽ' => with_handakuten('ほ'),
        // Small kana (拗音 / 促音) — scaled-down + offset versions.
        'ぁ' => with_small('あ'),
        'ぃ' => with_small('い'),
        'ぅ' => with_small('う'),
        'ぇ' => with_small('え'),
        'ぉ' => with_small('お'),
        'っ' => with_small('つ'),
        'ゃ' => with_small('や'),
        'ゅ' => with_small('ゆ'),
        'ょ' => with_small('よ'),
        'ゎ' => with_small('わ'),
        _ => None,
    }
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/// Apply an affine transform (scale + translate) to every stroke in a skeleton.
fn transform_skeleton(skel: &mut GlyphSkeleton, scale: f32, offset_x: f32, offset_y: f32) {
    let apply = |p: Point2| Point2::new(p.x * scale + offset_x, p.y * scale + offset_y);
    for i in 0..skel.stroke_count {
        let s = &mut skel.strokes[i];
        s.p0 = apply(s.p0);
        s.p1 = apply(s.p1);
        s.p2 = apply(s.p2);
        s.p3 = apply(s.p3);
    }
}

/// Append the dakuten (゛) mark in the upper-right region of the glyph.
fn append_dakuten(skel: &mut GlyphSkeleton) {
    // Two short diagonal ticks, top-right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.92),
        Point2::new(0.82, 0.88),
        Point2::new(0.86, 0.84),
        Point2::new(0.88, 0.78),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.88, 0.92),
        Point2::new(0.92, 0.88),
        Point2::new(0.94, 0.84),
        Point2::new(0.94, 0.78),
    ));
}

/// Append the handakuten (゜) mark — small closed circle in the upper-right.
fn append_handakuten(skel: &mut GlyphSkeleton) {
    add_cjk_stroke(
        skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.78, 0.78, 0.18, 0.18),
        0.5,
        0.0,
    );
}

fn with_dakuten(base: char) -> Option<GlyphSkeleton> {
    let mut skel = build_skeleton(base)?;
    append_dakuten(&mut skel);
    Some(skel)
}

fn with_handakuten(base: char) -> Option<GlyphSkeleton> {
    let mut skel = build_skeleton(base)?;
    append_handakuten(&mut skel);
    Some(skel)
}

fn with_small(base: char) -> Option<GlyphSkeleton> {
    let mut skel = build_skeleton(base)?;
    // Scale to ~65% and push toward the upper-right (the typical position
    // for small kana when used in 拗音 like きゃ, きゅ, きょ).
    transform_skeleton(&mut skel, 0.65, 0.3, 0.2);
    skel.advance = KANA_ADVANCE * 0.65;
    Some(skel)
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

// ----------------------------------------------------------------------------
// か (U+304B) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ka() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top horizontal that turns down-right into a hook.
    skel.add_stroke(Stroke::line(
        Point2::new(0.12, 0.72),
        Point2::new(0.62, 0.72),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.62, 0.72),
        Point2::new(0.62, 0.5),
        Point2::new(0.5, 0.25),
        Point2::new(0.3, 0.1),
    ));
    // 2: Vertical-curved second stroke that crosses the top bar.
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.88),
        Point2::new(0.36, 0.55),
        Point2::new(0.32, 0.3),
        Point2::new(0.25, 0.12),
    ));
    // 3: Small accent stroke on the right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.58),
        Point2::new(0.85, 0.48),
        Point2::new(0.85, 0.35),
        Point2::new(0.78, 0.32),
    ));
    skel
}

// ----------------------------------------------------------------------------
// き (U+304D) — 4 strokes
// ----------------------------------------------------------------------------
fn build_ki() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.78, 0.82),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.12, 0.6), Point2::new(0.82, 0.6)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.62, 0.95),
        Point2::new(0.5, 0.65),
        Point2::new(0.38, 0.35),
        Point2::new(0.32, 0.18),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.18),
        Point2::new(0.42, 0.05),
        Point2::new(0.65, 0.1),
        Point2::new(0.82, 0.22),
    ));
    skel
}

// ----------------------------------------------------------------------------
// く (U+304F) — 1 stroke chevron
// ----------------------------------------------------------------------------
fn build_ku() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.85),
        Point2::new(0.45, 0.65),
        Point2::new(0.25, 0.4),
        Point2::new(0.22, 0.32),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.22, 0.32),
        Point2::new(0.35, 0.2),
        Point2::new(0.6, 0.1),
        Point2::new(0.82, 0.08),
    ));
    skel
}

// ----------------------------------------------------------------------------
// け (U+3051) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ke() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Left vertical stroke (curved slightly).
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.88),
        Point2::new(0.15, 0.55),
        Point2::new(0.18, 0.25),
        Point2::new(0.22, 0.08),
    ));
    // 2: Top horizontal bar.
    skel.add_stroke(Stroke::line(
        Point2::new(0.4, 0.72),
        Point2::new(0.85, 0.72),
    ));
    // 3: Right vertical with hook at bottom.
    skel.add_stroke(Stroke::line(Point2::new(0.6, 0.88), Point2::new(0.58, 0.2)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.58, 0.2),
        Point2::new(0.5, 0.08),
        Point2::new(0.42, 0.1),
        Point2::new(0.4, 0.2),
    ));
    skel
}

// ----------------------------------------------------------------------------
// こ (U+3053) — 2 strokes
// ----------------------------------------------------------------------------
fn build_ko() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top stroke (slightly curved horizontal).
    skel.add_stroke(Stroke::new(
        Point2::new(0.2, 0.72),
        Point2::new(0.4, 0.78),
        Point2::new(0.6, 0.78),
        Point2::new(0.78, 0.72),
    ));
    // 2: Bottom curved stroke with sweep up at the right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.25),
        Point2::new(0.35, 0.18),
        Point2::new(0.6, 0.18),
        Point2::new(0.82, 0.3),
    ));
    skel
}

// ----------------------------------------------------------------------------
// さ (U+3055) — 3 strokes
// ----------------------------------------------------------------------------
fn build_sa() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top horizontal.
    skel.add_stroke(Stroke::line(
        Point2::new(0.15, 0.78),
        Point2::new(0.8, 0.78),
    ));
    // 2: Vertical-curved spine crossing the bar.
    skel.add_stroke(Stroke::new(
        Point2::new(0.55, 0.9),
        Point2::new(0.45, 0.65),
        Point2::new(0.35, 0.4),
        Point2::new(0.3, 0.22),
    ));
    // 3: Lower curl.
    skel.add_stroke(Stroke::new(
        Point2::new(0.3, 0.22),
        Point2::new(0.45, 0.05),
        Point2::new(0.7, 0.1),
        Point2::new(0.82, 0.25),
    ));
    skel
}

// ----------------------------------------------------------------------------
// し (U+3057) — 1 stroke J-shape
// ----------------------------------------------------------------------------
fn build_shi() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.88),
        Point2::new(0.38, 0.55),
        Point2::new(0.34, 0.2),
        Point2::new(0.4, 0.08),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.08),
        Point2::new(0.55, 0.05),
        Point2::new(0.72, 0.15),
        Point2::new(0.82, 0.3),
    ));
    skel
}

// ----------------------------------------------------------------------------
// す (U+3059) — 2 strokes (horizontal + vertical with loop)
// ----------------------------------------------------------------------------
fn build_su() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top horizontal.
    skel.add_stroke(Stroke::line(
        Point2::new(0.18, 0.7),
        Point2::new(0.82, 0.72),
    ));
    // 2: Vertical crossing the bar.
    skel.add_stroke(Stroke::line(Point2::new(0.55, 0.88), Point2::new(0.5, 0.5)));
    // 3: Lower closed loop forming the bottom curl.
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.28, 0.1, 0.4, 0.4),
        0.5,
        0.0,
    );
    // 4: Tail exiting lower-right.
    skel.add_stroke(Stroke::new(
        Point2::new(0.5, 0.1),
        Point2::new(0.62, 0.08),
        Point2::new(0.72, 0.1),
        Point2::new(0.78, 0.18),
    ));
    skel
}

// ----------------------------------------------------------------------------
// せ (U+305B) — 3 strokes
// ----------------------------------------------------------------------------
fn build_se() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    // 1: Top diagonal stroke (left side).
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.85),
        Point2::new(0.2, 0.7),
        Point2::new(0.25, 0.55),
        Point2::new(0.3, 0.45),
    ));
    // 2: Top horizontal bar.
    skel.add_stroke(Stroke::line(Point2::new(0.1, 0.6), Point2::new(0.82, 0.6)));
    // 3: Vertical down + bottom curve.
    skel.add_stroke(Stroke::line(Point2::new(0.55, 0.85), Point2::new(0.5, 0.2)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.5, 0.2),
        Point2::new(0.6, 0.1),
        Point2::new(0.75, 0.15),
        Point2::new(0.82, 0.3),
    ));
    skel
}

// ----------------------------------------------------------------------------
// そ (U+305D) — single zigzag stroke
// ----------------------------------------------------------------------------
fn build_so() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.18, 0.85),
        Point2::new(0.78, 0.78),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.78),
        Point2::new(0.5, 0.65),
        Point2::new(0.3, 0.55),
        Point2::new(0.25, 0.45),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.25, 0.45),
        Point2::new(0.4, 0.35),
        Point2::new(0.7, 0.3),
        Point2::new(0.85, 0.35),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.85, 0.35),
        Point2::new(0.7, 0.18),
        Point2::new(0.4, 0.1),
        Point2::new(0.2, 0.12),
    ));
    skel
}

// ----------------------------------------------------------------------------
// た (U+305F) — 4 strokes
// ----------------------------------------------------------------------------
fn build_ta() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.1, 0.78), Point2::new(0.5, 0.78)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.92),
        Point2::new(0.28, 0.65),
        Point2::new(0.22, 0.35),
        Point2::new(0.18, 0.18),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.55, 0.5), Point2::new(0.88, 0.5)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.66, 0.4),
        Point2::new(0.58, 0.22),
        Point2::new(0.7, 0.08),
        Point2::new(0.88, 0.12),
    ));
    skel
}

// ----------------------------------------------------------------------------
// ち (U+3061) — 2 strokes
// ----------------------------------------------------------------------------
fn build_chi() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.2, 0.82),
        Point2::new(0.78, 0.82),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.58, 0.92),
        Point2::new(0.3, 0.7),
        Point2::new(0.18, 0.5),
        Point2::new(0.28, 0.3),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.28, 0.3),
        Point2::new(0.55, 0.18),
        Point2::new(0.72, 0.15),
        Point2::new(0.6, 0.05),
    ));
    skel
}

// ----------------------------------------------------------------------------
// つ (U+3064) — 1 stroke shallow curve
// ----------------------------------------------------------------------------
fn build_tsu() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.7),
        Point2::new(0.4, 0.78),
        Point2::new(0.65, 0.78),
        Point2::new(0.82, 0.7),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.82, 0.7),
        Point2::new(0.78, 0.45),
        Point2::new(0.55, 0.2),
        Point2::new(0.25, 0.2),
    ));
    skel
}

// ----------------------------------------------------------------------------
// て (U+3066) — 1 logical stroke
// ----------------------------------------------------------------------------
fn build_te() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.78, 0.85),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.62, 0.85),
        Point2::new(0.5, 0.6),
        Point2::new(0.32, 0.4),
        Point2::new(0.3, 0.2),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.3, 0.2),
        Point2::new(0.4, 0.08),
        Point2::new(0.6, 0.1),
        Point2::new(0.7, 0.2),
    ));
    skel
}

// ----------------------------------------------------------------------------
// と (U+3068) — 2 strokes
// ----------------------------------------------------------------------------
fn build_to() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.78),
        Point2::new(0.35, 0.6),
        Point2::new(0.38, 0.55),
        Point2::new(0.45, 0.52),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.3, 0.85),
        Point2::new(0.32, 0.5),
        Point2::new(0.3, 0.2),
        Point2::new(0.42, 0.1),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.1),
        Point2::new(0.6, 0.12),
        Point2::new(0.78, 0.2),
        Point2::new(0.85, 0.3),
    ));
    skel
}

// ----------------------------------------------------------------------------
// な (U+306A) — 4 strokes
// ----------------------------------------------------------------------------
fn build_na() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.1, 0.78), Point2::new(0.6, 0.78)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.92),
        Point2::new(0.28, 0.6),
        Point2::new(0.22, 0.3),
        Point2::new(0.18, 0.15),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.72, 0.65),
        Point2::new(0.78, 0.55),
        Point2::new(0.78, 0.45),
        Point2::new(0.7, 0.4),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.5, 0.1, 0.38, 0.3),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// に (U+306B) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ni() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.22, 0.88),
        Point2::new(0.18, 0.55),
        Point2::new(0.22, 0.25),
        Point2::new(0.3, 0.1),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.45, 0.72),
        Point2::new(0.85, 0.72),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.42, 0.28),
        Point2::new(0.88, 0.28),
    ));
    skel
}

// ----------------------------------------------------------------------------
// ぬ (U+306C) — 2 logical strokes
// ----------------------------------------------------------------------------
fn build_nu() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.28, 0.88),
        Point2::new(0.18, 0.5),
        Point2::new(0.22, 0.2),
        Point2::new(0.4, 0.08),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.6, 0.82),
        Point2::new(0.78, 0.6),
        Point2::new(0.62, 0.4),
        Point2::new(0.42, 0.45),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.38, 0.1, 0.4, 0.4),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// ね (U+306D) — 2 strokes (vertical + curve with loop)
// ----------------------------------------------------------------------------
fn build_ne() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.22, 0.88), Point2::new(0.2, 0.1)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.78),
        Point2::new(0.55, 0.7),
        Point2::new(0.78, 0.65),
        Point2::new(0.82, 0.45),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.38, 0.1, 0.45, 0.4),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// の (U+306E) — single loop with tail
// ----------------------------------------------------------------------------
fn build_no() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.72, 0.85),
        Point2::new(0.55, 0.85),
        Point2::new(0.32, 0.7),
        Point2::new(0.22, 0.5),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.18, 0.1, 0.6, 0.5),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// は (U+306F) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ha() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.2, 0.88),
        Point2::new(0.17, 0.55),
        Point2::new(0.2, 0.25),
        Point2::new(0.25, 0.08),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.4, 0.65),
        Point2::new(0.85, 0.65),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.6, 0.88),
        Point2::new(0.58, 0.32),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.42, 0.05, 0.45, 0.35),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// ひ (U+3072) — 1 stroke arrow-like curve
// ----------------------------------------------------------------------------
fn build_hi() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.55),
        Point2::new(0.35, 0.78),
        Point2::new(0.45, 0.78),
        Point2::new(0.55, 0.6),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.55, 0.6),
        Point2::new(0.62, 0.4),
        Point2::new(0.62, 0.2),
        Point2::new(0.5, 0.1),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.5, 0.1),
        Point2::new(0.7, 0.1),
        Point2::new(0.82, 0.2),
        Point2::new(0.86, 0.35),
    ));
    skel
}

// ----------------------------------------------------------------------------
// ふ (U+3075) — 4 strokes (top tick, main curve, 2 lower accents)
// ----------------------------------------------------------------------------
fn build_fu() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.45, 0.88),
        Point2::new(0.5, 0.82),
        Point2::new(0.55, 0.78),
        Point2::new(0.6, 0.72),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.62),
        Point2::new(0.42, 0.55),
        Point2::new(0.52, 0.5),
        Point2::new(0.6, 0.4),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.6, 0.4),
        Point2::new(0.55, 0.28),
        Point2::new(0.4, 0.22),
        Point2::new(0.28, 0.3),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.22),
        Point2::new(0.2, 0.16),
        Point2::new(0.22, 0.1),
        Point2::new(0.28, 0.05),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.72, 0.22),
        Point2::new(0.78, 0.16),
        Point2::new(0.82, 0.1),
        Point2::new(0.85, 0.05),
    ));
    skel
}

// ----------------------------------------------------------------------------
// へ (U+3078) — single chevron
// ----------------------------------------------------------------------------
fn build_he() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.1, 0.4), Point2::new(0.5, 0.7)));
    skel.add_stroke(Stroke::line(Point2::new(0.5, 0.7), Point2::new(0.9, 0.42)));
    skel
}

// ----------------------------------------------------------------------------
// ほ (U+307B) — 4 strokes (like は plus extra horizontal)
// ----------------------------------------------------------------------------
fn build_ho() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.2, 0.92),
        Point2::new(0.17, 0.55),
        Point2::new(0.2, 0.25),
        Point2::new(0.25, 0.08),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.42, 0.75),
        Point2::new(0.85, 0.75),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.42, 0.5), Point2::new(0.85, 0.5)));
    skel.add_stroke(Stroke::line(Point2::new(0.62, 0.92), Point2::new(0.6, 0.3)));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.45, 0.05, 0.42, 0.28),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// ま (U+307E) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ma() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.15, 0.78),
        Point2::new(0.8, 0.78),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.15, 0.5), Point2::new(0.8, 0.5)));
    skel.add_stroke(Stroke::line(
        Point2::new(0.5, 0.92),
        Point2::new(0.48, 0.18),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.28, 0.08, 0.42, 0.28),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// み (U+307F) — 2 logical strokes
// ----------------------------------------------------------------------------
fn build_mi() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.2, 0.78),
        Point2::new(0.45, 0.6),
        Point2::new(0.7, 0.55),
        Point2::new(0.8, 0.42),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.8, 0.42),
        Point2::new(0.55, 0.4),
        Point2::new(0.3, 0.3),
        Point2::new(0.18, 0.18),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.18),
        Point2::new(0.32, 0.05),
        Point2::new(0.55, 0.08),
        Point2::new(0.75, 0.22),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.5, 0.05, 0.32, 0.22),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// む (U+3080) — 4 strokes
// ----------------------------------------------------------------------------
fn build_mu() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.15, 0.62),
        Point2::new(0.78, 0.62),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.92),
        Point2::new(0.28, 0.45),
        Point2::new(0.22, 0.22),
        Point2::new(0.32, 0.08),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.08),
        Point2::new(0.55, 0.05),
        Point2::new(0.72, 0.18),
        Point2::new(0.82, 0.3),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.45),
        Point2::new(0.85, 0.38),
        Point2::new(0.85, 0.32),
        Point2::new(0.82, 0.28),
    ));
    skel
}

// ----------------------------------------------------------------------------
// め (U+3081) — 2 strokes
// ----------------------------------------------------------------------------
fn build_me() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.3, 0.85),
        Point2::new(0.18, 0.55),
        Point2::new(0.22, 0.22),
        Point2::new(0.42, 0.1),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.62, 0.85),
        Point2::new(0.82, 0.6),
        Point2::new(0.65, 0.4),
        Point2::new(0.42, 0.42),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.32, 0.08, 0.5, 0.42),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// も (U+3082) — 3 strokes
// ----------------------------------------------------------------------------
fn build_mo() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.42, 0.92),
        Point2::new(0.4, 0.25),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.25),
        Point2::new(0.5, 0.12),
        Point2::new(0.7, 0.1),
        Point2::new(0.82, 0.22),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.18, 0.7), Point2::new(0.78, 0.7)));
    skel.add_stroke(Stroke::line(Point2::new(0.15, 0.5), Point2::new(0.75, 0.5)));
    skel
}

// ----------------------------------------------------------------------------
// や (U+3084) — 3 strokes
// ----------------------------------------------------------------------------
fn build_ya() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.12, 0.62),
        Point2::new(0.35, 0.78),
        Point2::new(0.55, 0.78),
        Point2::new(0.72, 0.65),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.72, 0.65),
        Point2::new(0.7, 0.45),
        Point2::new(0.55, 0.25),
        Point2::new(0.35, 0.12),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.7, 0.5), Point2::new(0.88, 0.78)));
    skel
}

// ----------------------------------------------------------------------------
// ゆ (U+3086) — 2 strokes (oval + vertical)
// ----------------------------------------------------------------------------
fn build_yu() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.18, 0.18, 0.62, 0.55),
        0.5,
        0.0,
    );
    skel.add_stroke(Stroke::line(Point2::new(0.5, 0.92), Point2::new(0.5, 0.05)));
    skel
}

// ----------------------------------------------------------------------------
// よ (U+3088) — 2 strokes
// ----------------------------------------------------------------------------
fn build_yo() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.45, 0.92),
        Point2::new(0.4, 0.18),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.4, 0.18),
        Point2::new(0.5, 0.05),
        Point2::new(0.7, 0.08),
        Point2::new(0.82, 0.22),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.18, 0.6), Point2::new(0.75, 0.6)));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.42, 0.08, 0.38, 0.35),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// ら (U+3089) — 2 strokes
// ----------------------------------------------------------------------------
fn build_ra() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.92),
        Point2::new(0.4, 0.82),
        Point2::new(0.4, 0.78),
        Point2::new(0.42, 0.72),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.62),
        Point2::new(0.35, 0.55),
        Point2::new(0.55, 0.55),
        Point2::new(0.68, 0.5),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.68, 0.5),
        Point2::new(0.62, 0.35),
        Point2::new(0.42, 0.2),
        Point2::new(0.22, 0.18),
    ));
    skel
}

// ----------------------------------------------------------------------------
// り (U+308A) — 2 near-parallel strokes
// ----------------------------------------------------------------------------
fn build_ri() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.3, 0.85),
        Point2::new(0.28, 0.55),
        Point2::new(0.3, 0.3),
        Point2::new(0.4, 0.15),
    ));
    skel.add_stroke(Stroke::line(
        Point2::new(0.7, 0.85),
        Point2::new(0.68, 0.25),
    ));
    skel
}

// ----------------------------------------------------------------------------
// る (U+308B) — single curve with closed loop
// ----------------------------------------------------------------------------
fn build_ru() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.2, 0.78),
        Point2::new(0.78, 0.78),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.78),
        Point2::new(0.5, 0.65),
        Point2::new(0.3, 0.55),
        Point2::new(0.25, 0.42),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.25, 0.42),
        Point2::new(0.42, 0.32),
        Point2::new(0.6, 0.3),
        Point2::new(0.72, 0.22),
    ));
    add_cjk_stroke(
        &mut skel,
        CjkStrokeType::Loop,
        StrokePlacement::new(0.4, 0.05, 0.4, 0.32),
        0.5,
        0.0,
    );
    skel
}

// ----------------------------------------------------------------------------
// れ (U+308C) — 2 strokes
// ----------------------------------------------------------------------------
fn build_re() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.22, 0.88), Point2::new(0.2, 0.1)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.72),
        Point2::new(0.6, 0.65),
        Point2::new(0.78, 0.55),
        Point2::new(0.82, 0.4),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.82, 0.4),
        Point2::new(0.65, 0.3),
        Point2::new(0.5, 0.28),
        Point2::new(0.42, 0.32),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.32),
        Point2::new(0.55, 0.18),
        Point2::new(0.72, 0.1),
        Point2::new(0.88, 0.12),
    ));
    skel
}

// ----------------------------------------------------------------------------
// ろ (U+308D) — single curve (no closed loop)
// ----------------------------------------------------------------------------
fn build_ro() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.2, 0.78),
        Point2::new(0.78, 0.78),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.78, 0.78),
        Point2::new(0.5, 0.6),
        Point2::new(0.32, 0.5),
        Point2::new(0.25, 0.35),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.25, 0.35),
        Point2::new(0.42, 0.18),
        Point2::new(0.7, 0.1),
        Point2::new(0.88, 0.2),
    ));
    skel
}

// ----------------------------------------------------------------------------
// わ (U+308F) — 2 strokes
// ----------------------------------------------------------------------------
fn build_wa() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(Point2::new(0.22, 0.88), Point2::new(0.2, 0.1)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.42, 0.72),
        Point2::new(0.6, 0.65),
        Point2::new(0.78, 0.55),
        Point2::new(0.82, 0.4),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.82, 0.4),
        Point2::new(0.65, 0.3),
        Point2::new(0.5, 0.32),
        Point2::new(0.5, 0.18),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.5, 0.18),
        Point2::new(0.62, 0.08),
        Point2::new(0.78, 0.05),
        Point2::new(0.88, 0.12),
    ));
    skel
}

// ----------------------------------------------------------------------------
// を (U+3092) — 3 strokes
// ----------------------------------------------------------------------------
fn build_wo() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::line(
        Point2::new(0.12, 0.78),
        Point2::new(0.85, 0.78),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.5, 0.92),
        Point2::new(0.45, 0.65),
        Point2::new(0.4, 0.55),
        Point2::new(0.32, 0.45),
    ));
    skel.add_stroke(Stroke::line(Point2::new(0.15, 0.5), Point2::new(0.85, 0.5)));
    skel.add_stroke(Stroke::new(
        Point2::new(0.32, 0.45),
        Point2::new(0.45, 0.22),
        Point2::new(0.7, 0.1),
        Point2::new(0.88, 0.18),
    ));
    skel
}

// ----------------------------------------------------------------------------
// ん (U+3093) — single zigzag
// ----------------------------------------------------------------------------
fn build_n() -> GlyphSkeleton {
    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANA_ADVANCE;
    skel.add_stroke(Stroke::new(
        Point2::new(0.18, 0.78),
        Point2::new(0.2, 0.55),
        Point2::new(0.22, 0.3),
        Point2::new(0.2, 0.12),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.2, 0.12),
        Point2::new(0.3, 0.4),
        Point2::new(0.4, 0.65),
        Point2::new(0.6, 0.55),
    ));
    skel.add_stroke(Stroke::new(
        Point2::new(0.6, 0.55),
        Point2::new(0.78, 0.45),
        Point2::new(0.85, 0.25),
        Point2::new(0.88, 0.12),
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

    /// Characters that have a real skeleton implementation today.
    /// Kept in one place so subsequent S3 sub-steps can extend it.
    const IMPLEMENTED: &[char] = &[
        'あ', 'い', 'う', 'え', 'お', // a-row
        'か', 'き', 'く', 'け', 'こ', // ka-row
        'さ', 'し', 'す', 'せ', 'そ', // sa-row
        'た', 'ち', 'つ', 'て', 'と', // ta-row
        'な', 'に', 'ぬ', 'ね', 'の', // na-row
        'は', 'ひ', 'ふ', 'へ', 'ほ', // ha-row
        'ま', 'み', 'む', 'め', 'も', // ma-row
        'や', 'ゆ', 'よ', // ya-row
        'ら', 'り', 'る', 'れ', 'ろ', // ra-row
        'わ', 'を', 'ん', // wa-row + n
        // Dakuten / handakuten
        'が', 'ぎ', 'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど',
        'ば', 'び', 'ぶ', 'べ', 'ぼ', 'ゔ', 'ぱ', 'ぴ', 'ぷ', 'ぺ', 'ぽ', // Small kana
        'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'っ', 'ゃ', 'ゅ', 'ょ', 'ゎ',
    ];

    #[test]
    fn implemented_glyphs_have_inside_pixels() {
        let params = MetaFontParams::sans_regular();
        for &ch in IMPLEMENTED {
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
    fn implemented_glyphs_bbox_in_unit_square() {
        let params = MetaFontParams::sans_regular();
        for &ch in IMPLEMENTED {
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
