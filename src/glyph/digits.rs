//! Digit glyphs 0-9 with tabular (fixed-width) advance
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::helpers::*;
use super::{GlyphGenerator, GlyphSkeleton};
use crate::stroke::{Point2, Stroke};

/// Tabular advance width for all digits (ensures numeric columns align)
const DIGIT_ADVANCE: f32 = 0.55;

impl GlyphGenerator {
    pub(crate) fn build_digit_0(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        add_ellipse(&mut skel, cx, cy, rx, ry, self.slant);
        skel
    }

    pub(crate) fn build_digit_1(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Vertical stem centered
        let x = DIGIT_ADVANCE / 2.0;
        skel.add_stroke(
            Stroke::line(Point2::new(x, 0.0), Point2::new(x, h)).apply_slant(self.slant),
        );
        // Left flag at top
        skel.add_stroke(
            Stroke::line(Point2::new(x - 0.1, h * 0.8), Point2::new(x, h)).apply_slant(self.slant),
        );
        // Base line
        skel.add_stroke(
            Stroke::line(Point2::new(x - 0.08, 0.0), Point2::new(x + 0.08, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_2(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Top curve
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.75),
                Point2::new(0.05, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.65),
            )
            .apply_slant(self.slant),
        );
        // Diagonal
        skel.add_stroke(
            Stroke::line(Point2::new(w, h * 0.65), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        // Base
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(w, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_3(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Upper arc
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.08, h * 0.85),
                Point2::new(0.08, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.6),
            )
            .apply_slant(self.slant),
        );
        // Mid connection
        skel.add_stroke(
            Stroke::line(Point2::new(w, h * 0.6), Point2::new(w * 0.6, h * 0.5))
                .apply_slant(self.slant),
        );
        // Lower arc
        skel.add_stroke(
            Stroke::new(
                Point2::new(w * 0.6, h * 0.5),
                Point2::new(w + 0.02, h * 0.4),
                Point2::new(w + 0.02, 0.0),
                Point2::new(0.08, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_4(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let bar_y = h * 0.35;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Diagonal from top-left to bar
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(0.05, bar_y)).apply_slant(self.slant),
        );
        // Horizontal bar
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, bar_y), Point2::new(w, bar_y)).apply_slant(self.slant),
        );
        // Right stem (full height)
        skel.add_stroke(
            Stroke::line(Point2::new(w * 0.7, 0.0), Point2::new(w * 0.7, h))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_5(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Top bar
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w, h)).apply_slant(self.slant),
        );
        // Vertical drop
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(0.05, h * 0.55)).apply_slant(self.slant),
        );
        // Lower bowl
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.55),
                Point2::new(w, h * 0.55),
                Point2::new(w, 0.0),
                Point2::new(0.05, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_6(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.52;
        let cx = w / 2.0;
        let bowl_cy = h * 0.28;
        let bowl_ry = h * 0.28;
        let bowl_rx = w / 2.0 - 0.03;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Upper curve from top to left side
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.85),
                Point2::new(w * 0.6, h),
                Point2::new(0.05, h * 0.7),
                Point2::new(0.05, h * 0.4),
            )
            .apply_slant(self.slant),
        );
        // Lower bowl
        add_ellipse(&mut skel, cx, bowl_cy, bowl_rx, bowl_ry, self.slant);
        skel
    }

    pub(crate) fn build_digit_7(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Top bar
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w, h)).apply_slant(self.slant),
        );
        // Diagonal
        skel.add_stroke(
            Stroke::line(Point2::new(w, h), Point2::new(w * 0.35, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_digit_8(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let cx = w / 2.0;
        let upper_cy = h * 0.73;
        let upper_ry = h * 0.25;
        let lower_cy = h * 0.27;
        let lower_ry = h * 0.27;
        let rx = w / 2.0 - 0.03;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        add_ellipse(&mut skel, cx, upper_cy, rx * 0.9, upper_ry, self.slant);
        add_ellipse(&mut skel, cx, lower_cy, rx, lower_ry, self.slant);
        skel
    }

    pub(crate) fn build_digit_9(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.52;
        let cx = w / 2.0;
        let bowl_cy = h * 0.72;
        let bowl_ry = h * 0.28;
        let bowl_rx = w / 2.0 - 0.03;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = DIGIT_ADVANCE;
        // Upper bowl
        add_ellipse(&mut skel, cx, bowl_cy, bowl_rx, bowl_ry, self.slant);
        // Tail from right side to bottom
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.6),
                Point2::new(w - 0.05, h * 0.3),
                Point2::new(w * 0.4, 0.0),
                Point2::new(0.08, h * 0.15),
            )
            .apply_slant(self.slant),
        );
        skel
    }
}
