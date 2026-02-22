//! Lowercase letter glyphs a-z
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::helpers::*;
use super::{GlyphGenerator, GlyphSkeleton};
use crate::stroke::{Point2, Stroke};

impl GlyphGenerator {
    pub(crate) fn build_lowercase_a(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        // Right stem
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, 0.0), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        // Bowl
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h),
                Point2::new(0.05, h),
                Point2::new(0.05, 0.0),
                Point2::new(w - 0.05, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_b(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = self.cap_height * 1.05;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        // Left stem (ascender)
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, asc)).apply_slant(self.slant),
        );
        // Bowl on right
        add_half_arc(&mut skel, 0.05, h / 2.0, w - 0.05, h / 2.0, 0, self.slant);
        skel
    }

    pub(crate) fn build_lowercase_c(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.7;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.03;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 3, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 2, self.slant);
        skel
    }

    pub(crate) fn build_lowercase_d(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = self.cap_height * 1.05;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        // Right stem (ascender)
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, 0.0), Point2::new(w - 0.05, asc))
                .apply_slant(self.slant),
        );
        // Bowl on left
        add_half_arc(
            &mut skel,
            w - 0.05,
            h / 2.0,
            w - 0.05,
            h / 2.0,
            1,
            self.slant,
        );
        skel
    }

    pub(crate) fn build_lowercase_e(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.8;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.03;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        // Horizontal bar at mid
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, cy), Point2::new(w - 0.05, cy)).apply_slant(self.slant),
        );
        // Upper arc
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 0, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 3, self.slant);
        // Lower arc (open at bottom-right)
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 2, self.slant);
        skel
    }

    pub(crate) fn build_lowercase_f(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = self.cap_height * 1.05;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        // Vertical stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.15, 0.0), Point2::new(0.15, asc * 0.9))
                .apply_slant(self.slant),
        );
        // Top hook
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.15, asc * 0.9),
                Point2::new(0.15, asc),
                Point2::new(0.30, asc),
                Point2::new(0.30, asc * 0.9),
            )
            .apply_slant(self.slant),
        );
        // Crossbar at x-height
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, h), Point2::new(0.28, h)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_g(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let desc = self.cap_height * 0.25;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        // Bowl (same as 'a')
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h),
                Point2::new(0.05, h),
                Point2::new(0.05, 0.0),
                Point2::new(w - 0.05, 0.0),
            )
            .apply_slant(self.slant),
        );
        // Right stem extending below baseline
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w - 0.05, -desc))
                .apply_slant(self.slant),
        );
        // Bottom hook
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, -desc),
                Point2::new(w - 0.05, -desc - 0.05),
                Point2::new(0.1, -desc - 0.05),
                Point2::new(0.1, -desc),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_h(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = self.cap_height * 1.05;
        let w = h * 0.75;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, asc)).apply_slant(self.slant),
        );
        // Shoulder and right stem
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(0.05, h + 0.02),
                Point2::new(w - 0.05, h + 0.02),
                Point2::new(w - 0.05, h),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w - 0.05, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_j(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let desc = self.cap_height * 0.25;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.28;
        // Stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.14, -desc * 0.5), Point2::new(0.14, h))
                .apply_slant(self.slant),
        );
        // Bottom hook
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.14, -desc * 0.5),
                Point2::new(0.14, -desc),
                Point2::new(0.02, -desc),
                Point2::new(0.02, -desc * 0.5),
            )
            .apply_slant(self.slant),
        );
        // Dot
        let dot_y = h + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.14, dot_y), Point2::new(0.14, dot_y + 0.02))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_k(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = self.cap_height * 1.05;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, asc)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w, h), Point2::new(0.05, h * 0.4)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, h * 0.48), Point2::new(w, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_m(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 1.1;
        let third = w / 3.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // First arch + stem
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(0.05, h + 0.02),
                Point2::new(third, h + 0.02),
                Point2::new(third, h),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(third, h), Point2::new(third, 0.0)).apply_slant(self.slant),
        );
        // Second arch + stem
        skel.add_stroke(
            Stroke::new(
                Point2::new(third, h),
                Point2::new(third, h + 0.02),
                Point2::new(third * 2.0, h + 0.02),
                Point2::new(third * 2.0, h),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(third * 2.0, h), Point2::new(third * 2.0, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_n(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.75;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(0.05, h + 0.02),
                Point2::new(w - 0.05, h + 0.02),
                Point2::new(w - 0.05, h),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w - 0.05, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_p(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let desc = self.cap_height * 0.25;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, -desc), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        add_half_arc(&mut skel, 0.05, h / 2.0, w - 0.05, h / 2.0, 0, self.slant);
        skel
    }

    pub(crate) fn build_lowercase_q(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let desc = self.cap_height * 0.25;
        let w = h * 0.8;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, -desc), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        add_half_arc(
            &mut skel,
            w - 0.05,
            h / 2.0,
            w - 0.05,
            h / 2.0,
            1,
            self.slant,
        );
        skel
    }

    pub(crate) fn build_lowercase_r(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.45;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Shoulder (no descending stem)
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(0.05, h + 0.02),
                Point2::new(w, h + 0.02),
                Point2::new(w, h * 0.7),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_s(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.03, h * 0.85),
                Point2::new(w - 0.03, h),
                Point2::new(0.03, h),
                Point2::new(0.03, h * 0.65),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.03, h * 0.65),
                Point2::new(0.03, h * 0.5),
                Point2::new(w - 0.03, h * 0.5),
                Point2::new(w - 0.03, h * 0.35),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.03, h * 0.35),
                Point2::new(w - 0.03, 0.0),
                Point2::new(0.03, 0.0),
                Point2::new(0.03, h * 0.15),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_t(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let asc = h * 1.3;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        // Stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.15, 0.0), Point2::new(0.15, asc)).apply_slant(self.slant),
        );
        // Crossbar
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, h), Point2::new(0.28, h)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_u(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.75;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(0.05, h * 0.25)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.25),
                Point2::new(0.05, 0.0),
                Point2::new(w - 0.05, 0.0),
                Point2::new(w - 0.05, h * 0.25),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h * 0.25), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_v(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w / 2.0, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, 0.0), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_w(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.9;
        let q = w / 4.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(q, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(q, 0.0), Point2::new(w / 2.0, h * 0.5))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, h * 0.5), Point2::new(w - q, 0.0))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - q, 0.0), Point2::new(w - 0.05, h)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_x(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w - 0.05, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_y(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let desc = self.cap_height * 0.25;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w / 2.0, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(0.1, -desc)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_lowercase_z(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w - 0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(w - 0.05, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }
}
