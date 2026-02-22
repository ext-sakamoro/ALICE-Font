//! CJK basic strokes — composable stroke primitives for CJK characters
//!
//! Provides 8 fundamental stroke types that can be composed to build
//! simple CJK characters. Not a full CJK renderer — covers the basic
//! strokes (永字八法) as building blocks.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::GlyphSkeleton;
use crate::stroke::{Point2, Stroke};

/// CJK stroke type (永字八法 + ti)
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
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
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
        ] {
            let mut skel = GlyphSkeleton::empty();
            add_cjk_stroke(&mut skel, stroke_type, place, 0.5, 0.0);
            assert!(
                skel.stroke_count > 0,
                "Stroke {:?} should add at least one stroke",
                stroke_type
            );
        }
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
