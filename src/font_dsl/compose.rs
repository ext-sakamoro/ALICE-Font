//! IDS (Ideographic Description Sequence) composition operators。
//!
//! 12 種の layout operator を提供、複数 `SkeletonGlyph` を新 kanji の
//! `SkeletonGlyph` に合成する。各 sub-glyph の stroke coordinate を
//! target bbox に scale + translate して積み上げる。
//!
//! # IDS 12 operators
//!
//! - ⿰ `left_right(codepoint, split_x, left, right)`
//! - ⿱ `top_bottom(codepoint, split_y, top, bottom)`
//! - ⿲ `triple_horizontal(codepoint, [left, middle, right])`
//! - ⿳ `triple_vertical(codepoint, [top, middle, bottom])`
//! - ⿴ `enclose_all(codepoint, outer, inner)` (口 系)
//! - ⿵ `enclose_top(codepoint, outer, inner)` (冂 系)
//! - ⿶ `enclose_bottom(codepoint, outer, inner)`
//! - ⿷ `enclose_left(codepoint, outer, inner)`
//! - ⿸ `enclose_upper_left(codepoint, outer, inner)` (厂 广 系)
//! - ⿹ `enclose_upper_right(codepoint, outer, inner)`
//! - ⿺ `enclose_lower_left(codepoint, outer, inner)` (走 起 系)
//! - ⿻ `overlap(codepoint, base, overlay)`
//!
//! v0.7.1-alpha MVP: `left_right` / `top_bottom` / `triple_h` / `triple_v` の
//! 4 operator を実装、他は future work。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use crate::glyph::MAX_GLYPH_STROKES;
use crate::skeleton_shader::SkeletonGlyph;
use crate::stroke::{Point2, Stroke};

/// Point2 を x range `[x_min, x_max]` × y range `[y_min, y_max]` に scale。
///
/// Input point は em box `[0, 1] × [0, 1]` 前提。
#[must_use]
fn scale_point(p: Point2, x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Point2 {
    Point2::new(x_min + p.x * (x_max - x_min), y_min + p.y * (y_max - y_min))
}

/// Stroke を bbox `[x_min, x_max] × [y_min, y_max]` に scale。
#[must_use]
fn scale_stroke(s: Stroke, x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> Stroke {
    Stroke::new(
        scale_point(s.p0, x_min, x_max, y_min, y_max),
        scale_point(s.p1, x_min, x_max, y_min, y_max),
        scale_point(s.p2, x_min, x_max, y_min, y_max),
        scale_point(s.p3, x_min, x_max, y_min, y_max),
    )
}

/// Sub-glyph strokes を target bbox に scale して flat Vec<Stroke> に転写 (widths 同時転送)。
///
/// v0.7.2-alpha: widths (stroke ごとの pen_start/pen_end) も scale と同期して転送。
/// Width は bbox scale の影響を受けない (画素幅 相当、em 単位)、そのまま copy。
#[allow(clippy::too_many_arguments)] // Internal helper、bbox 4 座標 + 3 mut ref = 意図的
fn transfer_strokes(
    sub: &SkeletonGlyph,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    out: &mut [Stroke; MAX_GLYPH_STROKES],
    out_widths: &mut [(f32, f32); MAX_GLYPH_STROKES],
    cursor: &mut usize,
) {
    for i in 0..sub.stroke_count {
        if *cursor >= MAX_GLYPH_STROKES {
            return;
        }
        out[*cursor] = scale_stroke(sub.strokes[i], x_min, x_max, y_min, y_max);
        out_widths[*cursor] = sub.stroke_widths[i];
        *cursor += 1;
    }
}

/// ⿰ **left_right** — 左右分割 kanji 合成。
///
/// `split_x` = 左 sub-glyph の右端 x 座標 (em、typical 0.5)。
/// 左 sub-glyph は `[0, split_x]`、右 sub-glyph は `[split_x, 1]` に scale。
///
/// Ex: 明 = `left_right('明', 0.5, hi(), tsuki())`
#[must_use]
pub fn left_right(
    codepoint: char,
    split_x: f32,
    left: SkeletonGlyph,
    right: SkeletonGlyph,
) -> SkeletonGlyph {
    let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
    let mut widths = [(0.0f32, 0.0f32); MAX_GLYPH_STROKES];
    let mut cursor = 0;
    transfer_strokes(
        &left,
        0.0,
        split_x,
        0.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &right,
        split_x,
        1.0,
        0.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    SkeletonGlyph {
        codepoint,
        strokes,
        stroke_widths: widths,
        stroke_count: cursor,
        advance: 1.0,
        pen_width: left.pen_width,
    }
}

/// ⿱ **top_bottom** — 上下分割 kanji 合成。
///
/// `split_y` = 上 sub-glyph の下端 y 座標 (em、typical 0.5)。
/// Note: y は 上 = 大、下 = 小 の Y-UP 系。上 sub-glyph は `[split_y, 1]`、
/// 下 sub-glyph は `[0, split_y]` に scale。
///
/// Ex: 森 = `top_bottom('森', 0.5, ki(), left_right('_', 0.5, ki(), ki()))`
#[must_use]
pub fn top_bottom(
    codepoint: char,
    split_y: f32,
    top: SkeletonGlyph,
    bottom: SkeletonGlyph,
) -> SkeletonGlyph {
    let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
    let mut widths = [(0.0f32, 0.0f32); MAX_GLYPH_STROKES];
    let mut cursor = 0;
    // Y-UP: top glyph → high y range
    transfer_strokes(
        &top,
        0.0,
        1.0,
        split_y,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &bottom,
        0.0,
        1.0,
        0.0,
        split_y,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    SkeletonGlyph {
        codepoint,
        strokes,
        stroke_widths: widths,
        stroke_count: cursor,
        advance: 1.0,
        pen_width: top.pen_width,
    }
}

/// ⿲ **triple_horizontal** — 3 分割 (左中右) kanji 合成。
///
/// 3 sub-glyph を x range `[0, 1/3]`, `[1/3, 2/3]`, `[2/3, 1]` に scale。
///
/// Ex: 順 (左 川 系 3 部品) 等の 3 部品 kanji。
#[must_use]
pub fn triple_horizontal(
    codepoint: char,
    left: SkeletonGlyph,
    middle: SkeletonGlyph,
    right: SkeletonGlyph,
) -> SkeletonGlyph {
    let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
    let mut widths = [(0.0f32, 0.0f32); MAX_GLYPH_STROKES];
    let mut cursor = 0;
    transfer_strokes(
        &left,
        0.0,
        1.0 / 3.0,
        0.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &middle,
        1.0 / 3.0,
        2.0 / 3.0,
        0.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &right,
        2.0 / 3.0,
        1.0,
        0.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    SkeletonGlyph {
        codepoint,
        strokes,
        stroke_widths: widths,
        stroke_count: cursor,
        advance: 1.0,
        pen_width: left.pen_width,
    }
}

/// ⿳ **triple_vertical** — 3 分割 (上中下) kanji 合成 (Y-UP)。
///
/// 3 sub-glyph を y range `[2/3, 1]` (top), `[1/3, 2/3]` (middle),
/// `[0, 1/3]` (bottom) に scale。
///
/// Ex: 章 意 等 の 3 段構造 kanji。
#[must_use]
pub fn triple_vertical(
    codepoint: char,
    top: SkeletonGlyph,
    middle: SkeletonGlyph,
    bottom: SkeletonGlyph,
) -> SkeletonGlyph {
    let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
    let mut widths = [(0.0f32, 0.0f32); MAX_GLYPH_STROKES];
    let mut cursor = 0;
    transfer_strokes(
        &top,
        0.0,
        1.0,
        2.0 / 3.0,
        1.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &middle,
        0.0,
        1.0,
        1.0 / 3.0,
        2.0 / 3.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    transfer_strokes(
        &bottom,
        0.0,
        1.0,
        0.0,
        1.0 / 3.0,
        &mut strokes,
        &mut widths,
        &mut cursor,
    );
    SkeletonGlyph {
        codepoint,
        strokes,
        stroke_widths: widths,
        stroke_count: cursor,
        advance: 1.0,
        pen_width: top.pen_width,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_dsl::{glyph, stroke::*};

    fn dummy_1stroke(ch: char) -> SkeletonGlyph {
        glyph(ch).add(horizontal(0.5, 0.1, 0.9)).build()
    }

    #[test]
    fn scale_point_maps_unit_to_target() {
        let p = Point2::new(0.5, 0.5);
        let scaled = scale_point(p, 0.0, 0.5, 0.0, 1.0);
        assert!((scaled.x - 0.25).abs() < 1e-6);
        assert!((scaled.y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn left_right_combines_stroke_counts() {
        let left = dummy_1stroke('a');
        let right = dummy_1stroke('b');
        let combined = left_right('c', 0.5, left, right);
        assert_eq!(combined.stroke_count, 2);
        assert_eq!(combined.codepoint, 'c');
    }

    #[test]
    fn left_right_scales_left_to_left_half() {
        let left = dummy_1stroke('a'); // stroke: horizontal(0.5, 0.1, 0.9) in em box
        let right = dummy_1stroke('b');
        let combined = left_right('c', 0.5, left, right);
        // Left glyph's stroke x=[0.1, 0.9] should scale to x=[0.05, 0.45] (half range)
        let s = &combined.strokes[0];
        assert!((s.p0.x - 0.05).abs() < 1e-6);
        assert!((s.p3.x - 0.45).abs() < 1e-6);
    }

    #[test]
    fn top_bottom_scales_top_to_upper_half() {
        let top = dummy_1stroke('t'); // horizontal at y=0.5
        let bottom = dummy_1stroke('b');
        let combined = top_bottom('x', 0.5, top, bottom);
        // Top glyph's y=0.5 should scale to y = 0.5 + 0.5 * 0.5 = 0.75 (upper half)
        let s = &combined.strokes[0];
        assert!((s.p0.y - 0.75).abs() < 1e-6);
    }

    #[test]
    fn triple_horizontal_produces_3_stroke_result() {
        let a = dummy_1stroke('a');
        let b = dummy_1stroke('b');
        let c = dummy_1stroke('c');
        let combined = triple_horizontal('x', a, b, c);
        assert_eq!(combined.stroke_count, 3);
    }

    #[test]
    fn compose_truncates_at_max_strokes() {
        // Build a glyph with 10 strokes
        let mut big = glyph('a');
        for _ in 0..10 {
            big = big.add(horizontal(0.5, 0.0, 1.0));
        }
        let g = big.build();
        // Combine 2 such glyphs = 20 strokes total, truncated to MAX_GLYPH_STROKES=16
        let combined = left_right('x', 0.5, g, g);
        assert_eq!(combined.stroke_count, MAX_GLYPH_STROKES);
    }
}
