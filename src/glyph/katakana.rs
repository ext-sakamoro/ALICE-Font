//! Katakana glyph rendering — U+30A0 to U+30FF
//!
//! S4 implementation. 46 清音 + 濁音 + 半濁音 + 小書き + 長音記号。
//! See `docs/CJK_KANA_SPEC.md`.
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use crate::glyph::cjk_strokes::{add_cjk_stroke, CjkStrokeType, StrokePlacement};
use crate::glyph::{GlyphGenerator, GlyphSdf, GlyphSkeleton};
use crate::param::MetaFontParams;
use crate::stroke::{Point2, Stroke};

const KANA_ADVANCE: f32 = 1.0;

/// Generate the SDF for a katakana character.
#[must_use]
pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    build_skeleton(ch).map_or_else(
        || {
            let mut sdf = GlyphSdf::empty();
            sdf.advance = KANA_ADVANCE;
            sdf
        },
        |skel| {
            let gen = GlyphGenerator::new(params);
            gen.generate_from_skeleton(&skel)
        },
    )
}

pub(crate) fn build_skeleton(ch: char) -> Option<GlyphSkeleton> {
    match ch {
        'ア' => Some(build_a()),
        'イ' => Some(build_i()),
        'ウ' => Some(build_u()),
        'エ' => Some(build_e()),
        'オ' => Some(build_o()),
        'カ' => Some(build_ka()),
        'キ' => Some(build_ki()),
        'ク' => Some(build_ku()),
        'ケ' => Some(build_ke()),
        'コ' => Some(build_ko()),
        'サ' => Some(build_sa()),
        'シ' => Some(build_shi()),
        'ス' => Some(build_su()),
        'セ' => Some(build_se()),
        'ソ' => Some(build_so()),
        'タ' => Some(build_ta()),
        'チ' => Some(build_chi()),
        'ツ' => Some(build_tsu()),
        'テ' => Some(build_te()),
        'ト' => Some(build_to()),
        'ナ' => Some(build_na()),
        'ニ' => Some(build_ni()),
        'ヌ' => Some(build_nu()),
        'ネ' => Some(build_ne()),
        'ノ' => Some(build_no()),
        'ハ' => Some(build_ha()),
        'ヒ' => Some(build_hi()),
        'フ' => Some(build_fu()),
        'ヘ' => Some(build_he()),
        'ホ' => Some(build_ho()),
        'マ' => Some(build_ma()),
        'ミ' => Some(build_mi()),
        'ム' => Some(build_mu()),
        'メ' => Some(build_me()),
        'モ' => Some(build_mo()),
        'ヤ' => Some(build_ya()),
        'ユ' => Some(build_yu()),
        'ヨ' => Some(build_yo()),
        'ラ' => Some(build_ra()),
        'リ' => Some(build_ri()),
        'ル' => Some(build_ru()),
        'レ' => Some(build_re()),
        'ロ' => Some(build_ro()),
        'ワ' => Some(build_wa()),
        'ヲ' => Some(build_wo()),
        'ン' => Some(build_n()),
        'ー' => Some(build_choon()),
        // Dakuten
        'ガ' => with_dakuten('カ'),
        'ギ' => with_dakuten('キ'),
        'グ' => with_dakuten('ク'),
        'ゲ' => with_dakuten('ケ'),
        'ゴ' => with_dakuten('コ'),
        'ザ' => with_dakuten('サ'),
        'ジ' => with_dakuten('シ'),
        'ズ' => with_dakuten('ス'),
        'ゼ' => with_dakuten('セ'),
        'ゾ' => with_dakuten('ソ'),
        'ダ' => with_dakuten('タ'),
        'ヂ' => with_dakuten('チ'),
        'ヅ' => with_dakuten('ツ'),
        'デ' => with_dakuten('テ'),
        'ド' => with_dakuten('ト'),
        'バ' => with_dakuten('ハ'),
        'ビ' => with_dakuten('ヒ'),
        'ブ' => with_dakuten('フ'),
        'ベ' => with_dakuten('ヘ'),
        'ボ' => with_dakuten('ホ'),
        'ヴ' => with_dakuten('ウ'),
        // Handakuten
        'パ' => with_handakuten('ハ'),
        'ピ' => with_handakuten('ヒ'),
        'プ' => with_handakuten('フ'),
        'ペ' => with_handakuten('ヘ'),
        'ポ' => with_handakuten('ホ'),
        // Small kana
        'ァ' => with_small('ア'),
        'ィ' => with_small('イ'),
        'ゥ' => with_small('ウ'),
        'ェ' => with_small('エ'),
        'ォ' => with_small('オ'),
        'ッ' => with_small('ツ'),
        'ャ' => with_small('ヤ'),
        'ュ' => with_small('ユ'),
        'ョ' => with_small('ヨ'),
        'ヮ' => with_small('ワ'),
        _ => None,
    }
}

// ----------------------------------------------------------------------------
// Helpers (mirrored from hiragana.rs)
// ----------------------------------------------------------------------------

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

const fn append_dakuten(skel: &mut GlyphSkeleton) {
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
    transform_skeleton(&mut skel, 0.65, 0.3, 0.2);
    skel.advance = KANA_ADVANCE * 0.65;
    Some(skel)
}

// ----------------------------------------------------------------------------
// 清音 — 46 chars
// ----------------------------------------------------------------------------

fn build_a() -> GlyphSkeleton {
    // ア — 2 strokes: top horizontal hook + diagonal
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.78),
        Point2::new(0.85, 0.78),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.78),
        Point2::new(0.8, 0.55),
        Point2::new(0.65, 0.35),
        Point2::new(0.45, 0.18),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.55, 0.62),
        Point2::new(0.42, 0.45),
        Point2::new(0.28, 0.25),
        Point2::new(0.18, 0.1),
    ));
    s
}

fn build_i() -> GlyphSkeleton {
    // イ — 2 strokes: pie + vertical
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.68, 0.92),
        Point2::new(0.52, 0.7),
        Point2::new(0.35, 0.5),
        Point2::new(0.15, 0.35),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.55, 0.65),
        Point2::new(0.5, 0.08),
    ));
    s
}

fn build_u() -> GlyphSkeleton {
    // ウ — the top tick, the top stroke, the short left side, and the right side that sweeps down to the lower
    // left (read against BIZ UDPGothic's outline: there is no vertical through the middle)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.48, 0.92),
        Point2::new(0.52, 0.82),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.68),
        Point2::new(0.82, 0.68),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.68),
        Point2::new(0.18, 0.45),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.82, 0.68),
        Point2::new(0.82, 0.42),
        Point2::new(0.7, 0.2),
        Point2::new(0.35, 0.1),
    ));
    s
}

fn build_e() -> GlyphSkeleton {
    // エ — 3 strokes: top horizontal + vertical + bottom horizontal
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.82),
        Point2::new(0.82, 0.82),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.5, 0.82), Point2::new(0.5, 0.18)));
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.18),
        Point2::new(0.88, 0.18),
    ));
    s
}

fn build_o() -> GlyphSkeleton {
    // オ — the horizontal, the vertical with its hook to the left, and a stroke falling from the crossing to the lower left (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.15, 0.7), Point2::new(0.85, 0.7)));
    s.add_stroke(Stroke::line(
        Point2::new(0.45, 0.92),
        Point2::new(0.4, 0.22),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.4, 0.22),
        Point2::new(0.32, 0.08),
        Point2::new(0.18, 0.1),
        Point2::new(0.12, 0.22),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.42, 0.62),
        Point2::new(0.34, 0.5),
        Point2::new(0.24, 0.4),
        Point2::new(0.12, 0.3),
    ));
    s
}

fn build_ka() -> GlyphSkeleton {
    // カ — the top stroke runs right and drops down the right side with a hook to the left; the second falls from the top to the lower left (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.78),
        Point2::new(0.72, 0.78),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.72, 0.78),
        Point2::new(0.74, 0.45),
        Point2::new(0.72, 0.15),
        Point2::new(0.6, 0.1),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.4, 0.92),
        Point2::new(0.35, 0.6),
        Point2::new(0.28, 0.35),
        Point2::new(0.18, 0.15),
    ));
    s
}

fn build_ki() -> GlyphSkeleton {
    // キ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.85),
        Point2::new(0.78, 0.78),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.6),
        Point2::new(0.82, 0.55),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.52, 0.88),
        Point2::new(0.45, 0.1),
    ));
    s
}

fn build_ku() -> GlyphSkeleton {
    // ク — the short first stroke falls left from the top; the second runs right along the top and
    // turns into one long sweep down to the lower left (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.36, 0.92),
        Point2::new(0.3, 0.8),
        Point2::new(0.22, 0.7),
        Point2::new(0.12, 0.62),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.33, 0.84),
        Point2::new(0.8, 0.84),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.8, 0.84),
        Point2::new(0.76, 0.55),
        Point2::new(0.58, 0.28),
        Point2::new(0.25, 0.1),
    ));
    s
}

fn build_ke() -> GlyphSkeleton {
    // ケ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.32, 0.92),
        Point2::new(0.28, 0.7),
        Point2::new(0.22, 0.45),
        Point2::new(0.12, 0.25),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.32, 0.7), Point2::new(0.82, 0.7)));
    s.add_stroke(Stroke::line(
        Point2::new(0.65, 0.85),
        Point2::new(0.55, 0.1),
    ));
    s
}

fn build_ko() -> GlyphSkeleton {
    // コ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.82),
        Point2::new(0.82, 0.82),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.82, 0.82),
        Point2::new(0.78, 0.2),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.2), Point2::new(0.85, 0.2)));
    s
}

fn build_sa() -> GlyphSkeleton {
    // サ — the horizontal, a short left vertical, and a long right stroke that turns to the lower left (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.65),
        Point2::new(0.85, 0.65),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.35, 0.88),
        Point2::new(0.32, 0.4),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.65, 0.88),
        Point2::new(0.66, 0.5),
        Point2::new(0.6, 0.28),
        Point2::new(0.42, 0.1),
    ));
    s
}

fn build_shi() -> GlyphSkeleton {
    // シ — two short strokes, then one long stroke rising from the lower left to the upper right (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.3, 0.72),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.15, 0.6), Point2::new(0.3, 0.5)));
    s.add_stroke(Stroke::new(
        Point2::new(0.18, 0.25),
        Point2::new(0.4, 0.35),
        Point2::new(0.65, 0.45),
        Point2::new(0.85, 0.55),
    ));
    s
}

fn build_su() -> GlyphSkeleton {
    // ス — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.85, 0.82),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.82),
        Point2::new(0.65, 0.6),
        Point2::new(0.45, 0.35),
        Point2::new(0.25, 0.12),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.4, 0.4),
        Point2::new(0.55, 0.3),
        Point2::new(0.72, 0.18),
        Point2::new(0.85, 0.1),
    ));
    s
}

fn build_se() -> GlyphSkeleton {
    // セ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.12, 0.78),
        Point2::new(0.5, 0.78),
        Point2::new(0.85, 0.7),
        Point2::new(0.82, 0.55),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.4, 0.88),
        Point2::new(0.38, 0.25),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.38, 0.25),
        Point2::new(0.55, 0.15),
        Point2::new(0.75, 0.15),
        Point2::new(0.88, 0.22),
    ));
    s
}

fn build_so() -> GlyphSkeleton {
    // ソ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.45, 0.88),
        Point2::new(0.55, 0.7),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.7, 0.82),
        Point2::new(0.55, 0.6),
        Point2::new(0.35, 0.35),
        Point2::new(0.18, 0.15),
    ));
    s
}

fn build_ta() -> GlyphSkeleton {
    // タ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.78, 0.88),
        Point2::new(0.5, 0.7),
        Point2::new(0.3, 0.55),
        Point2::new(0.15, 0.35),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.3, 0.55),
        Point2::new(0.78, 0.55),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.78, 0.55),
        Point2::new(0.7, 0.35),
        Point2::new(0.55, 0.2),
        Point2::new(0.35, 0.08),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.65, 0.4),
        Point2::new(0.7, 0.35),
        Point2::new(0.75, 0.32),
        Point2::new(0.78, 0.3),
    ));
    s
}

fn build_chi() -> GlyphSkeleton {
    // チ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    // the first stroke falls short from the upper right to the left (read against BIZ UDPGothic's outline)
    s.add_stroke(Stroke::new(
        Point2::new(0.8, 0.94),
        Point2::new(0.6, 0.88),
        Point2::new(0.4, 0.84),
        Point2::new(0.2, 0.8),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.65),
        Point2::new(0.85, 0.6),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.55, 0.75),
        Point2::new(0.45, 0.1),
    ));
    s
}

fn build_tsu() -> GlyphSkeleton {
    // ツ — two short strokes falling to the right, then one long stroke from the upper right down
    // to the lower left (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.24, 0.62),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.4, 0.86),
        Point2::new(0.47, 0.68),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.86),
        Point2::new(0.8, 0.55),
        Point2::new(0.62, 0.28),
        Point2::new(0.32, 0.1),
    ));
    s
}

fn build_te() -> GlyphSkeleton {
    // テ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.85, 0.82),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.22, 0.6), Point2::new(0.78, 0.6)));
    s.add_stroke(Stroke::new(
        Point2::new(0.55, 0.6),
        Point2::new(0.5, 0.4),
        Point2::new(0.42, 0.22),
        Point2::new(0.32, 0.12),
    ));
    s
}

fn build_to() -> GlyphSkeleton {
    // ト — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.35, 0.92),
        Point2::new(0.32, 0.1),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.35, 0.55),
        Point2::new(0.5, 0.5),
        Point2::new(0.7, 0.4),
        Point2::new(0.85, 0.3),
    ));
    s
}

fn build_na() -> GlyphSkeleton {
    // ナ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.12, 0.7), Point2::new(0.82, 0.7)));
    s.add_stroke(Stroke::line(Point2::new(0.5, 0.85), Point2::new(0.45, 0.1)));
    s
}

fn build_ni() -> GlyphSkeleton {
    // ニ — 2 horizontals
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.68, 0.78),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.22),
        Point2::new(0.85, 0.22),
    ));
    s
}

fn build_nu() -> GlyphSkeleton {
    // ヌ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.85, 0.82),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.82),
        Point2::new(0.65, 0.6),
        Point2::new(0.45, 0.4),
        Point2::new(0.25, 0.2),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.5, 0.45),
        Point2::new(0.6, 0.3),
        Point2::new(0.72, 0.18),
        Point2::new(0.85, 0.1),
    ));
    s
}

fn build_ne() -> GlyphSkeleton {
    // ネ — 4 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.45, 0.92),
        Point2::new(0.5, 0.82),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.7), Point2::new(0.82, 0.7)));
    s.add_stroke(Stroke::line(Point2::new(0.5, 0.7), Point2::new(0.45, 0.2)));
    s.add_stroke(Stroke::new(
        Point2::new(0.32, 0.45),
        Point2::new(0.22, 0.32),
        Point2::new(0.18, 0.22),
        Point2::new(0.2, 0.12),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.6, 0.45),
        Point2::new(0.7, 0.3),
        Point2::new(0.78, 0.18),
        Point2::new(0.85, 0.12),
    ));
    s
}

fn build_no() -> GlyphSkeleton {
    // ノ — single pie
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.72, 0.92),
        Point2::new(0.55, 0.65),
        Point2::new(0.4, 0.4),
        Point2::new(0.15, 0.12),
    ));
    s
}

fn build_ha() -> GlyphSkeleton {
    // ハ — pie + na (V-shape)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.45, 0.88),
        Point2::new(0.35, 0.6),
        Point2::new(0.25, 0.35),
        Point2::new(0.12, 0.15),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.55, 0.88),
        Point2::new(0.65, 0.6),
        Point2::new(0.75, 0.35),
        Point2::new(0.88, 0.15),
    ));
    s
}

fn build_hi() -> GlyphSkeleton {
    // ヒ — a short stroke rising to the right from the vertical, and the vertical from the top that turns right along the bottom (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.27, 0.52),
        Point2::new(0.8, 0.72),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.27, 0.9), Point2::new(0.27, 0.2)));
    s.add_stroke(Stroke::new(
        Point2::new(0.27, 0.2),
        Point2::new(0.5, 0.18),
        Point2::new(0.7, 0.18),
        Point2::new(0.85, 0.2),
    ));
    s
}

fn build_fu() -> GlyphSkeleton {
    // フ — single horizontal-hook
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.78),
        Point2::new(0.85, 0.78),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.78),
        Point2::new(0.7, 0.55),
        Point2::new(0.5, 0.3),
        Point2::new(0.3, 0.12),
    ));
    s
}

fn build_he() -> GlyphSkeleton {
    // ヘ — chevron (same as ひらがな へ)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.1, 0.4), Point2::new(0.5, 0.7)));
    s.add_stroke(Stroke::line(Point2::new(0.5, 0.7), Point2::new(0.9, 0.42)));
    s
}

fn build_ho() -> GlyphSkeleton {
    // ホ — 4 strokes (cross + 2 ticks)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.15, 0.6), Point2::new(0.85, 0.6)));
    s.add_stroke(Stroke::line(
        Point2::new(0.5, 0.88),
        Point2::new(0.48, 0.12),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.35, 0.45),
        Point2::new(0.28, 0.32),
        Point2::new(0.22, 0.22),
        Point2::new(0.15, 0.12),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.62, 0.45),
        Point2::new(0.7, 0.32),
        Point2::new(0.78, 0.22),
        Point2::new(0.85, 0.12),
    ));
    s
}

fn build_ma() -> GlyphSkeleton {
    // マ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.85, 0.82),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.82),
        Point2::new(0.6, 0.55),
        Point2::new(0.4, 0.32),
        Point2::new(0.2, 0.15),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.55, 0.5),
        Point2::new(0.5, 0.32),
        Point2::new(0.42, 0.2),
        Point2::new(0.35, 0.1),
    ));
    s
}

fn build_mi() -> GlyphSkeleton {
    // ミ — 3 horizontals
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.82),
        Point2::new(0.65, 0.78),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.55), Point2::new(0.7, 0.5)));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.22),
        Point2::new(0.85, 0.22),
    ));
    s
}

fn build_mu() -> GlyphSkeleton {
    // ム — a stroke falls from the top to the lower left and turns right along the bottom; a short stroke falls to the lower right at the right (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.42, 0.88),
        Point2::new(0.32, 0.65),
        Point2::new(0.22, 0.42),
        Point2::new(0.15, 0.2),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.2),
        Point2::new(0.85, 0.24),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.66, 0.46),
        Point2::new(0.86, 0.2),
    ));
    s
}

fn build_me() -> GlyphSkeleton {
    // メ — 2 crossed strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.62, 0.92),
        Point2::new(0.45, 0.65),
        Point2::new(0.3, 0.4),
        Point2::new(0.12, 0.15),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.22, 0.78),
        Point2::new(0.45, 0.55),
        Point2::new(0.65, 0.3),
        Point2::new(0.88, 0.1),
    ));
    s
}

fn build_mo() -> GlyphSkeleton {
    // モ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.78, 0.78),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.55),
        Point2::new(0.82, 0.55),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.4, 0.88), Point2::new(0.42, 0.2)));
    s.add_stroke(Stroke::new(
        Point2::new(0.42, 0.2),
        Point2::new(0.55, 0.1),
        Point2::new(0.72, 0.12),
        Point2::new(0.82, 0.22),
    ));
    s
}

fn build_ya() -> GlyphSkeleton {
    // ヤ — a stroke rising to the right that hooks down at its end, and a long stroke from the top down (read against BIZ UDPGothic's outline)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.12, 0.58),
        Point2::new(0.88, 0.74),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.88, 0.74),
        Point2::new(0.74, 0.52),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.55, 0.88),
        Point2::new(0.45, 0.1),
    ));
    s
}

fn build_yu() -> GlyphSkeleton {
    // ユ — a top stroke that turns down at its right end, and the long bottom stroke (read against
    // BIZ UDPGothic's outline: the vertical stands at the right, not in the middle)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.2, 0.78),
        Point2::new(0.68, 0.78),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.68, 0.78),
        Point2::new(0.7, 0.22),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.1, 0.22), Point2::new(0.9, 0.22)));
    s
}

fn build_yo() -> GlyphSkeleton {
    // ヨ — 3 horizontals + right vertical
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.82, 0.82),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.82, 0.82),
        Point2::new(0.78, 0.22),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.3, 0.55),
        Point2::new(0.78, 0.55),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.22),
        Point2::new(0.85, 0.22),
    ));
    s
}

fn build_ra() -> GlyphSkeleton {
    // ラ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.35, 0.88),
        Point2::new(0.65, 0.78),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.6), Point2::new(0.78, 0.6)));
    s.add_stroke(Stroke::new(
        Point2::new(0.78, 0.6),
        Point2::new(0.65, 0.4),
        Point2::new(0.42, 0.22),
        Point2::new(0.22, 0.12),
    ));
    s
}

fn build_ri() -> GlyphSkeleton {
    // リ — a short left stroke, and a long right one that turns to the lower left (read against
    // BIZ UDPGothic's outline: the left stroke stops about halfway down)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.3, 0.86), Point2::new(0.3, 0.48)));
    s.add_stroke(Stroke::new(
        Point2::new(0.7, 0.88),
        Point2::new(0.71, 0.45),
        Point2::new(0.66, 0.25),
        Point2::new(0.45, 0.1),
    ));
    s
}

fn build_ru() -> GlyphSkeleton {
    // ル — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.32, 0.88),
        Point2::new(0.28, 0.55),
        Point2::new(0.3, 0.3),
        Point2::new(0.42, 0.12),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.65, 0.88),
        Point2::new(0.62, 0.25),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.62, 0.25),
        Point2::new(0.68, 0.15),
        Point2::new(0.78, 0.1),
        Point2::new(0.88, 0.18),
    ));
    s
}

fn build_re() -> GlyphSkeleton {
    // レ — single L-shape stroke
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::new(
        Point2::new(0.32, 0.88),
        Point2::new(0.3, 0.55),
        Point2::new(0.35, 0.3),
        Point2::new(0.48, 0.15),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.48, 0.15),
        Point2::new(0.62, 0.25),
        Point2::new(0.78, 0.4),
        Point2::new(0.9, 0.55),
    ));
    s
}

fn build_ro() -> GlyphSkeleton {
    // ロ — 3 strokes (square)
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.82, 0.78),
    ));
    s.add_stroke(Stroke::line(Point2::new(0.82, 0.78), Point2::new(0.8, 0.2)));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.2), Point2::new(0.82, 0.2)));
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.78), Point2::new(0.2, 0.2)));
    s
}

fn build_wa() -> GlyphSkeleton {
    // ワ — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.82, 0.78),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.78),
        Point2::new(0.18, 0.55),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.82, 0.78),
        Point2::new(0.78, 0.55),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.78, 0.55),
        Point2::new(0.62, 0.4),
        Point2::new(0.42, 0.25),
        Point2::new(0.22, 0.12),
    ));
    s
}

fn build_wo() -> GlyphSkeleton {
    // ヲ — 3 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(
        Point2::new(0.15, 0.82),
        Point2::new(0.85, 0.82),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.85, 0.82),
        Point2::new(0.8, 0.62),
    ));
    s.add_stroke(Stroke::line(
        Point2::new(0.18, 0.62),
        Point2::new(0.85, 0.62),
    ));
    s.add_stroke(Stroke::new(
        Point2::new(0.85, 0.62),
        Point2::new(0.65, 0.4),
        Point2::new(0.4, 0.22),
        Point2::new(0.18, 0.12),
    ));
    s
}

fn build_n() -> GlyphSkeleton {
    // ン — 2 strokes
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.18, 0.7), Point2::new(0.3, 0.6)));
    s.add_stroke(Stroke::new(
        Point2::new(0.18, 0.32),
        Point2::new(0.4, 0.45),
        Point2::new(0.65, 0.6),
        Point2::new(0.85, 0.7),
    ));
    s
}

fn build_choon() -> GlyphSkeleton {
    // ー — single long horizontal
    let mut s = GlyphSkeleton::empty();
    s.advance = KANA_ADVANCE;
    s.add_stroke(Stroke::line(Point2::new(0.1, 0.48), Point2::new(0.9, 0.5)));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMPLEMENTED: &[char] = &[
        // 清音 46
        'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ',
        'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ',
        'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ヲ',
        'ン', 'ー', // Dakuten
        'ガ', 'ギ', 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド',
        'バ', 'ビ', 'ブ', 'ベ', 'ボ', 'ヴ', // Handakuten
        'パ', 'ピ', 'プ', 'ペ', 'ポ', // Small kana
        'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ッ', 'ャ', 'ュ', 'ョ', 'ヮ',
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

    #[test]
    fn unimplemented_returns_empty_with_advance() {
        let sdf = generate('ヰ', &MetaFontParams::sans_regular()); // archaic, not implemented
        assert!(sdf.advance > 0.0);
    }
}
