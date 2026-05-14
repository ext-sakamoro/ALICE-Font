//! CJK basic strokes — composable stroke primitives for CJK characters
//!
//! Provides 8 classical CJK strokes (永字八法 + 鉤) as a starting palette
//! for kanji composition, plus three kana-oriented primitives (Curve, Loop,
//! Hook) introduced for hiragana/katakana rendering. See `docs/CJK_KANA_SPEC.md`.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::GlyphSkeleton;
use crate::stroke::{Point2, Stroke};

/// CJK stroke type (永字八法 + ti + kana extensions)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CjkStrokeType {
    /// 横 (héng) — horizontal stroke, slight upward slope
    Heng = 0,
    /// 竪 (shù) — vertical stroke
    Shu = 1,
    /// 撇 (piě) — left-falling stroke, tapering
    Pie = 2,
    /// 捺 (nà) — right-falling stroke
    Na = 3,
    /// 点 (diǎn) — dot stroke
    Dian = 4,
    /// 折 (zhé) — turning stroke
    Zhe = 5,
    /// 提 (tí) — upward flick
    Ti = 6,
    /// 鉤 (gōu) — hook
    Gou = 7,
    /// Curve — generic smooth S-shape through the placement diagonal.
    /// Useful as a quick building block for kana shapes that don't fit a
    /// classical 永字八法 stroke.
    Curve = 8,
    /// Loop — closed elliptical path inscribed in the placement rectangle.
    /// Consumes 4 stroke slots (one per quadrant). Used by characters with
    /// closed sub-shapes such as ぬ, ね, る, め.
    Loop = 9,
    /// Hook — short vertical stroke ending with a leftward curl. Lighter
    /// than `Gou`; used in kana where a softer hook is wanted.
    Hook = 10,
}

/// Placement rectangle for a CJK stroke
#[derive(Debug, Clone, Copy)]
pub struct StrokePlacement {
    /// Origin X (em units)
    pub x: f32,
    /// Origin Y (em units)
    pub y: f32,
    /// Width (em units)
    pub w: f32,
    /// Height (em units)
    pub h: f32,
}

impl StrokePlacement {
    #[must_use]
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

/// Add a CJK stroke to a glyph skeleton
pub fn add_cjk_stroke(
    skel: &mut GlyphSkeleton,
    stroke_type: CjkStrokeType,
    place: StrokePlacement,
    weight: f32,
    slant: f32,
) {
    let _ = weight; // weight is handled by the pen model, not stroke geometry
    match stroke_type {
        CjkStrokeType::Heng => {
            // Horizontal with slight upward slope
            let rise = place.h * 0.03;
            skel.add_stroke(
                Stroke::line(
                    Point2::new(place.x, place.y),
                    Point2::new(place.x + place.w, place.y + rise),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Shu => {
            // Vertical with entry serif
            let entry_w = place.w * 0.15;
            // Entry (small horizontal at top)
            skel.add_stroke(
                Stroke::line(
                    Point2::new(place.x - entry_w, place.y + place.h),
                    Point2::new(place.x + entry_w, place.y + place.h),
                )
                .apply_slant(slant),
            );
            // Main vertical
            skel.add_stroke(
                Stroke::line(
                    Point2::new(place.x, place.y),
                    Point2::new(place.x, place.y + place.h),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Pie => {
            // Left-falling with taper (Bezier curve)
            skel.add_stroke(
                Stroke::new(
                    Point2::new(place.x + place.w, place.y + place.h),
                    Point2::new(place.x + place.w * 0.6, place.y + place.h * 0.6),
                    Point2::new(place.x + place.w * 0.2, place.y + place.h * 0.2),
                    Point2::new(place.x, place.y),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Na => {
            // Right-falling with weight shift
            skel.add_stroke(
                Stroke::new(
                    Point2::new(place.x, place.y + place.h),
                    Point2::new(place.x + place.w * 0.3, place.y + place.h * 0.5),
                    Point2::new(place.x + place.w * 0.7, place.y + place.h * 0.15),
                    Point2::new(place.x + place.w, place.y),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Dian => {
            // Short dot stroke (diagonal)
            skel.add_stroke(
                Stroke::new(
                    Point2::new(place.x, place.y + place.h),
                    Point2::new(place.x + place.w * 0.3, place.y + place.h * 0.7),
                    Point2::new(place.x + place.w * 0.7, place.y + place.h * 0.3),
                    Point2::new(place.x + place.w, place.y),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Zhe => {
            // Turning: horizontal then vertical
            let turn_x = place.x + place.w;
            let turn_y = place.y + place.h;
            // Horizontal portion
            skel.add_stroke(
                Stroke::line(Point2::new(place.x, turn_y), Point2::new(turn_x, turn_y))
                    .apply_slant(slant),
            );
            // Vertical down
            skel.add_stroke(
                Stroke::line(Point2::new(turn_x, turn_y), Point2::new(turn_x, place.y))
                    .apply_slant(slant),
            );
        }
        CjkStrokeType::Ti => {
            // Upward flick from bottom-left to upper-right
            skel.add_stroke(
                Stroke::new(
                    Point2::new(place.x, place.y),
                    Point2::new(place.x + place.w * 0.3, place.y + place.h * 0.4),
                    Point2::new(place.x + place.w * 0.6, place.y + place.h * 0.7),
                    Point2::new(place.x + place.w, place.y + place.h),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Gou => {
            // Hook: vertical then hook left
            // Vertical portion
            skel.add_stroke(
                Stroke::line(
                    Point2::new(place.x, place.y + place.h),
                    Point2::new(place.x, place.y + place.h * 0.15),
                )
                .apply_slant(slant),
            );
            // Hook
            skel.add_stroke(
                Stroke::new(
                    Point2::new(place.x, place.y + place.h * 0.15),
                    Point2::new(place.x, place.y),
                    Point2::new(place.x - place.w * 0.3, place.y),
                    Point2::new(place.x - place.w * 0.4, place.y + place.h * 0.15),
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Curve => {
            // Generic smooth S-shape across the placement diagonal.
            // Goes from bottom-left to top-right with two midway control
            // points slightly offset to create a soft S.
            let p0 = Point2::new(place.x, place.y);
            let p3 = Point2::new(place.x + place.w, place.y + place.h);
            let p1 = Point2::new(place.x + place.w * 0.25, place.y + place.h * 0.7);
            let p2 = Point2::new(place.x + place.w * 0.75, place.y + place.h * 0.3);
            skel.add_stroke(Stroke::new(p0, p1, p2, p3).apply_slant(slant));
        }
        CjkStrokeType::Loop => {
            // Closed elliptical path inscribed in the placement rectangle.
            // Built from four cubic Bezier quadrants using the standard
            // (4/3)(√2 - 1) ≈ 0.5523 circle-to-Bezier approximation, scaled
            // by the placement rectangle.
            const KAPPA: f32 = 0.552_284_8;
            let cx = place.x + place.w * 0.5;
            let cy = place.y + place.h * 0.5;
            let rx = place.w * 0.5;
            let ry = place.h * 0.5;
            let ox = rx * KAPPA;
            let oy = ry * KAPPA;
            // Right
            let right = Point2::new(cx + rx, cy);
            // Top
            let top = Point2::new(cx, cy + ry);
            // Left
            let left = Point2::new(cx - rx, cy);
            // Bottom
            let bottom = Point2::new(cx, cy - ry);

            // right -> top
            skel.add_stroke(
                Stroke::new(
                    right,
                    Point2::new(cx + rx, cy + oy),
                    Point2::new(cx + ox, cy + ry),
                    top,
                )
                .apply_slant(slant),
            );
            // top -> left
            skel.add_stroke(
                Stroke::new(
                    top,
                    Point2::new(cx - ox, cy + ry),
                    Point2::new(cx - rx, cy + oy),
                    left,
                )
                .apply_slant(slant),
            );
            // left -> bottom
            skel.add_stroke(
                Stroke::new(
                    left,
                    Point2::new(cx - rx, cy - oy),
                    Point2::new(cx - ox, cy - ry),
                    bottom,
                )
                .apply_slant(slant),
            );
            // bottom -> right
            skel.add_stroke(
                Stroke::new(
                    bottom,
                    Point2::new(cx + ox, cy - ry),
                    Point2::new(cx + rx, cy - oy),
                    right,
                )
                .apply_slant(slant),
            );
        }
        CjkStrokeType::Hook => {
            // Short vertical with a soft leftward curl at the end. Lighter
            // than `Gou`. The hook tail extends ~0.3 × w to the left.
            let top = Point2::new(place.x, place.y + place.h);
            let bend = Point2::new(place.x, place.y + place.h * 0.2);
            let tail = Point2::new(place.x - place.w * 0.3, place.y + place.h * 0.05);
            skel.add_stroke(Stroke::line(top, bend).apply_slant(slant));
            skel.add_stroke(
                Stroke::new(
                    bend,
                    Point2::new(place.x, place.y + place.h * 0.05),
                    Point2::new(place.x - place.w * 0.15, place.y),
                    tail,
                )
                .apply_slant(slant),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_stroke_types() {
        let place = StrokePlacement::new(0.1, 0.1, 0.4, 0.5);
        for stroke_type in [
            CjkStrokeType::Heng,
            CjkStrokeType::Shu,
            CjkStrokeType::Pie,
            CjkStrokeType::Na,
            CjkStrokeType::Dian,
            CjkStrokeType::Zhe,
            CjkStrokeType::Ti,
            CjkStrokeType::Gou,
            CjkStrokeType::Curve,
            CjkStrokeType::Loop,
            CjkStrokeType::Hook,
        ] {
            let mut skel = GlyphSkeleton::empty();
            add_cjk_stroke(&mut skel, stroke_type, place, 0.5, 0.0);
            assert!(
                skel.stroke_count > 0,
                "Stroke {stroke_type:?} should add at least one stroke"
            );
        }
    }

    #[test]
    fn test_curve_single_stroke() {
        let mut skel = GlyphSkeleton::empty();
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Curve,
            StrokePlacement::new(0.1, 0.1, 0.6, 0.6),
            0.5,
            0.0,
        );
        assert_eq!(skel.stroke_count, 1);
    }

    #[test]
    fn test_loop_uses_four_strokes() {
        let mut skel = GlyphSkeleton::empty();
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Loop,
            StrokePlacement::new(0.2, 0.2, 0.5, 0.5),
            0.5,
            0.0,
        );
        assert_eq!(skel.stroke_count, 4);
    }

    #[test]
    fn test_hook_uses_two_strokes() {
        let mut skel = GlyphSkeleton::empty();
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Hook,
            StrokePlacement::new(0.3, 0.1, 0.3, 0.7),
            0.5,
            0.0,
        );
        assert_eq!(skel.stroke_count, 2);
    }

    #[test]
    fn test_loop_closes_back_to_start() {
        // The four quadrants of a Loop should chain end-to-start.
        let mut skel = GlyphSkeleton::empty();
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Loop,
            StrokePlacement::new(0.0, 0.0, 1.0, 1.0),
            0.5,
            0.0,
        );
        let last = skel.strokes[3];
        let first = skel.strokes[0];
        // The last stroke's end point should match the first stroke's start.
        let dx = last.p3.x - first.p0.x;
        let dy = last.p3.y - first.p0.y;
        assert!(dx.abs() < 1e-5, "loop end x drift: {dx}");
        assert!(dy.abs() < 1e-5, "loop end y drift: {dy}");
    }

    #[test]
    fn test_compose_yi_one() {
        // 一 = single Heng stroke
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.7;
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Heng,
            StrokePlacement::new(0.05, 0.35, 0.6, 0.0),
            0.5,
            0.0,
        );
        assert_eq!(skel.stroke_count, 1);
    }

    #[test]
    fn test_compose_shi_ten() {
        // 十 = Heng + Shu
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.7;
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Heng,
            StrokePlacement::new(0.05, 0.35, 0.6, 0.0),
            0.5,
            0.0,
        );
        add_cjk_stroke(
            &mut skel,
            CjkStrokeType::Shu,
            StrokePlacement::new(0.35, 0.05, 0.0, 0.6),
            0.5,
            0.0,
        );
        assert!(skel.stroke_count >= 2);
    }
}
