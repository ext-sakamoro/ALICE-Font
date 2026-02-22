//! Uppercase letter glyphs A-Z
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::helpers::*;
use super::{GlyphGenerator, GlyphSkeleton};
use crate::stroke::{Point2, Stroke};

impl GlyphGenerator {
    pub(crate) fn build_uppercase_c(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.65;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Open arc: left + top-left + bottom-left (3 quadrants)
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 3, self.slant); // top-left
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 2, self.slant); // bottom-left
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 0, self.slant); // top-right (partial)
        if self.serif_length > 0.0 {
            add_serif_at(
                &mut skel,
                Point2::new(cx + rx * 0.7, h),
                Point2::new(1.0, 0.0),
                self.serif_length,
                self.slant,
            );
        }
        skel
    }

    pub(crate) fn build_uppercase_d(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Vertical stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Right bowl
        add_half_arc(&mut skel, 0.05, h / 2.0, w - 0.05, h / 2.0, 0, self.slant);
        skel
    }

    pub(crate) fn build_uppercase_e(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.5), Point2::new(w * 0.85, h * 0.5))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(w, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_f(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.48;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.5), Point2::new(w * 0.8, h * 0.5))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_g(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.68;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 3, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 2, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 1, self.slant);
        // Horizontal bar at mid-height on right
        skel.add_stroke(
            Stroke::line(Point2::new(cx, cy), Point2::new(cx + rx, cy)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_j(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.4;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Vertical stem on right
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h * 0.2), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        // Bottom hook (quarter arc)
        add_quarter_arc(
            &mut skel,
            w * 0.5,
            h * 0.2,
            w * 0.45,
            h * 0.2,
            1,
            self.slant,
        );
        skel
    }

    pub(crate) fn build_uppercase_k(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.58;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w, h), Point2::new(0.05, h * 0.45)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.15, h * 0.52), Point2::new(w, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_m(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.75;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w / 2.0, h * 0.3))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, h * 0.3), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w - 0.05, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_n(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w - 0.05, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, 0.0), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_p(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Upper bowl
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.5),
                Point2::new(0.05, h * 0.5),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_q(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.7;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.12;
        add_ellipse(&mut skel, cx, cy, rx, ry, self.slant);
        // Tail
        skel.add_stroke(
            Stroke::line(
                Point2::new(cx + rx * 0.3, cy - ry * 0.3),
                Point2::new(cx + rx + 0.05, cy - ry - 0.02),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_r(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Upper bowl (same as P)
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.5),
                Point2::new(0.05, h * 0.5),
            )
            .apply_slant(self.slant),
        );
        // Leg
        skel.add_stroke(
            Stroke::line(Point2::new(0.15, h * 0.5), Point2::new(w, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_s(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // S as two opposing arcs
        // Upper arc: opens right
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.85),
                Point2::new(w - 0.05, h),
                Point2::new(0.05, h),
                Point2::new(0.05, h * 0.7),
            )
            .apply_slant(self.slant),
        );
        // Mid connection
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.7),
                Point2::new(0.05, h * 0.5),
                Point2::new(w - 0.05, h * 0.5),
                Point2::new(w - 0.05, h * 0.3),
            )
            .apply_slant(self.slant),
        );
        // Lower arc: opens left
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.3),
                Point2::new(w - 0.05, 0.0),
                Point2::new(0.05, 0.0),
                Point2::new(0.05, h * 0.15),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_u(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(0.05, h * 0.25)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w - 0.05, h * 0.25))
                .apply_slant(self.slant),
        );
        // Bottom curve
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.25),
                Point2::new(0.05, 0.0),
                Point2::new(w - 0.05, 0.0),
                Point2::new(w - 0.05, h * 0.25),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_v(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.65;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w / 2.0, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, 0.0), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_w(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.85;
        let q = w / 4.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
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

    pub(crate) fn build_uppercase_x(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w - 0.05, 0.0)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_y(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mid = h * 0.45;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(w / 2.0, mid)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(w / 2.0, mid))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, mid), Point2::new(w / 2.0, 0.0))
                .apply_slant(self.slant),
        );
        skel
    }

    pub(crate) fn build_uppercase_z(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
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
