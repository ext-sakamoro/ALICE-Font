//! Punctuation and symbol glyphs
//!
//! Covers ASCII 0x21-0x2F, 0x3A-0x40, 0x5B-0x60, 0x7B-0x7E
//!
//! License: MIT
//! Author: Moroya Sakamoto

use super::helpers::*;
use super::{GlyphGenerator, GlyphSkeleton};
use crate::stroke::{Point2, Stroke};

impl GlyphGenerator {
    pub(crate) fn build_punct(&self, ch: u8) -> GlyphSkeleton {
        match ch {
            b'!' => self.build_exclamation(),
            b'"' => self.build_double_quote(),
            b'#' => self.build_hash(),
            b'$' => self.build_dollar(),
            b'%' => self.build_percent(),
            b'&' => self.build_ampersand(),
            b'\'' => self.build_single_quote(),
            b'(' => self.build_left_paren(),
            b')' => self.build_right_paren(),
            b'*' => self.build_asterisk(),
            b'+' => self.build_plus(),
            b',' => self.build_comma(),
            b'-' => self.build_hyphen(),
            b'.' => self.build_period(),
            b'/' => self.build_slash(),
            b':' => self.build_colon(),
            b';' => self.build_semicolon(),
            b'<' => self.build_less_than(),
            b'=' => self.build_equals(),
            b'>' => self.build_greater_than(),
            b'?' => self.build_question(),
            b'@' => self.build_at(),
            b'[' => self.build_left_bracket(),
            b'\\' => self.build_backslash(),
            b']' => self.build_right_bracket(),
            b'^' => self.build_caret(),
            b'_' => self.build_underscore(),
            b'`' => self.build_backtick(),
            b'{' => self.build_left_brace(),
            b'|' => self.build_pipe(),
            b'}' => self.build_right_brace(),
            b'~' => self.build_tilde(),
            _ => self.build_placeholder(ch),
        }
    }

    fn build_exclamation(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.25;
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, h * 0.25), Point2::new(0.12, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, 0.0), Point2::new(0.12, 0.02)).apply_slant(self.slant),
        );
        skel
    }

    fn build_double_quote(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.3;
        skel.add_stroke(
            Stroke::line(Point2::new(0.08, h * 0.7), Point2::new(0.08, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.22, h * 0.7), Point2::new(0.22, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_hash(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        let x1 = w * 0.3;
        let x2 = w * 0.7;
        let y1 = h * 0.3;
        let y2 = h * 0.65;
        skel.add_stroke(
            Stroke::line(Point2::new(x1, 0.0), Point2::new(x1, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(x2, 0.0), Point2::new(x2, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, y1), Point2::new(w, y1)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, y2), Point2::new(w, y2)).apply_slant(self.slant),
        );
        skel
    }

    fn build_dollar(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let cx = w / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // S shape (reuse S logic)
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.82),
                Point2::new(w - 0.05, h * 0.95),
                Point2::new(0.05, h * 0.95),
                Point2::new(0.05, h * 0.65),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.65),
                Point2::new(0.05, h * 0.5),
                Point2::new(w - 0.05, h * 0.5),
                Point2::new(w - 0.05, h * 0.35),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(w - 0.05, h * 0.35),
                Point2::new(w - 0.05, h * 0.05),
                Point2::new(0.05, h * 0.05),
                Point2::new(0.05, h * 0.18),
            )
            .apply_slant(self.slant),
        );
        // Vertical bar through center
        skel.add_stroke(
            Stroke::line(Point2::new(cx, 0.0), Point2::new(cx, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_percent(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let r = h * 0.12;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Diagonal
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, h), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        // Top-left circle
        add_ellipse(&mut skel, 0.12, h * 0.82, r, r, self.slant);
        // Bottom-right circle
        add_ellipse(&mut skel, w - 0.12, h * 0.18, r, r, self.slant);
        skel
    }

    fn build_ampersand(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Upper loop
        let r = h * 0.18;
        add_ellipse(&mut skel, w * 0.35, h * 0.78, r, r, self.slant);
        // Lower curve and tail
        skel.add_stroke(
            Stroke::new(
                Point2::new(w * 0.2, h * 0.6),
                Point2::new(0.02, h * 0.3),
                Point2::new(0.02, 0.0),
                Point2::new(w * 0.5, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w * 0.5, 0.0), Point2::new(w, h * 0.4))
                .apply_slant(self.slant),
        );
        skel
    }

    fn build_single_quote(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.18;
        skel.add_stroke(
            Stroke::line(Point2::new(0.09, h * 0.7), Point2::new(0.09, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_left_paren(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.28;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.22, h),
                Point2::new(0.05, h * 0.75),
                Point2::new(0.05, h * 0.25),
                Point2::new(0.22, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_right_paren(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.28;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.06, h),
                Point2::new(0.23, h * 0.75),
                Point2::new(0.23, h * 0.25),
                Point2::new(0.06, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_asterisk(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let cy = h * 0.75;
        let r = h * 0.12;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        let cx = 0.175;
        // 3 strokes at 60 degree angles
        skel.add_stroke(
            Stroke::line(Point2::new(cx, cy - r), Point2::new(cx, cy + r)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(
                Point2::new(cx - r * 0.866, cy - r * 0.5),
                Point2::new(cx + r * 0.866, cy + r * 0.5),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(
                Point2::new(cx - r * 0.866, cy + r * 0.5),
                Point2::new(cx + r * 0.866, cy - r * 0.5),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_plus(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let cy = h * 0.4;
        let w = h * 0.45;
        let r = h * 0.18;
        let cx = w / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(cx, cy - r), Point2::new(cx, cy + r)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(cx - r, cy), Point2::new(cx + r, cy)).apply_slant(self.slant),
        );
        skel
    }

    fn build_comma(&self) -> GlyphSkeleton {
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.1, 0.06),
                Point2::new(0.1, 0.0),
                Point2::new(0.06, -0.06),
                Point2::new(0.04, -0.08),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_hyphen(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.5), Point2::new(0.3, h * 0.5))
                .apply_slant(self.slant),
        );
        skel
    }

    fn build_period(&self) -> GlyphSkeleton {
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::line(Point2::new(0.1, 0.0), Point2::new(0.1, 0.02)).apply_slant(self.slant),
        );
        skel
    }

    fn build_slash(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        skel.add_stroke(
            Stroke::line(Point2::new(0.28, h), Point2::new(0.05, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    fn build_colon(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::line(Point2::new(0.1, 0.0), Point2::new(0.1, 0.02)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.1, h - 0.02), Point2::new(0.1, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_semicolon(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.1, 0.06),
                Point2::new(0.1, 0.0),
                Point2::new(0.06, -0.06),
                Point2::new(0.04, -0.08),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.1, h - 0.02), Point2::new(0.1, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_less_than(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.4;
        let mid = h * 0.4;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(w, h * 0.7), Point2::new(0.05, mid)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, mid), Point2::new(w, h * 0.1)).apply_slant(self.slant),
        );
        skel
    }

    fn build_equals(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.65), Point2::new(w, h * 0.65))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.35), Point2::new(w, h * 0.35))
                .apply_slant(self.slant),
        );
        skel
    }

    fn build_greater_than(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.4;
        let mid = h * 0.4;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.7), Point2::new(w, mid)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w, mid), Point2::new(0.05, h * 0.1)).apply_slant(self.slant),
        );
        skel
    }

    fn build_question(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.45;
        let cx = w / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        // Top arc
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.78),
                Point2::new(0.05, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.6),
            )
            .apply_slant(self.slant),
        );
        // Stem down to dot
        skel.add_stroke(
            Stroke::line(Point2::new(w, h * 0.6), Point2::new(cx, h * 0.25))
                .apply_slant(self.slant),
        );
        // Dot
        skel.add_stroke(
            Stroke::line(Point2::new(cx, 0.0), Point2::new(cx, 0.02)).apply_slant(self.slant),
        );
        skel
    }

    fn build_at(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.75;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0 - 0.02;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;
        // Outer circle (3 quadrants, open at bottom-right)
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 0, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 3, self.slant);
        add_quarter_arc(&mut skel, cx, cy, rx, ry, 2, self.slant);
        // Inner 'a' shape
        let ir = ry * 0.5;
        add_ellipse(&mut skel, cx + 0.02, cy, ir, ir, self.slant);
        skel
    }

    fn build_left_bracket(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.25;
        skel.add_stroke(
            Stroke::line(Point2::new(0.06, 0.0), Point2::new(0.06, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.06, h), Point2::new(0.2, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.06, 0.0), Point2::new(0.2, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    fn build_backslash(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.35;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h), Point2::new(0.28, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    fn build_right_bracket(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.25;
        skel.add_stroke(
            Stroke::line(Point2::new(0.18, 0.0), Point2::new(0.18, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.04, h), Point2::new(0.18, h)).apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(0.04, 0.0), Point2::new(0.18, 0.0)).apply_slant(self.slant),
        );
        skel
    }

    fn build_caret(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.4;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.6), Point2::new(w / 2.0, h))
                .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, h), Point2::new(w, h * 0.6)).apply_slant(self.slant),
        );
        skel
    }

    fn build_underscore(&self) -> GlyphSkeleton {
        let w = 0.45;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, -0.02), Point2::new(w, -0.02)).apply_slant(self.slant),
        );
        skel
    }

    fn build_backtick(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::line(Point2::new(0.06, h), Point2::new(0.12, h * 0.75)).apply_slant(self.slant),
        );
        skel
    }

    fn build_left_brace(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mid = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.3;
        // Upper curve
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.22, h),
                Point2::new(0.12, h),
                Point2::new(0.12, mid + 0.05),
                Point2::new(0.05, mid),
            )
            .apply_slant(self.slant),
        );
        // Lower curve
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, mid),
                Point2::new(0.12, mid - 0.05),
                Point2::new(0.12, 0.0),
                Point2::new(0.22, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_pipe(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.2;
        skel.add_stroke(
            Stroke::line(Point2::new(0.1, 0.0), Point2::new(0.1, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_right_brace(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mid = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.3;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.08, h),
                Point2::new(0.18, h),
                Point2::new(0.18, mid + 0.05),
                Point2::new(0.25, mid),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.25, mid),
                Point2::new(0.18, mid - 0.05),
                Point2::new(0.18, 0.0),
                Point2::new(0.08, 0.0),
            )
            .apply_slant(self.slant),
        );
        skel
    }

    fn build_tilde(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let cy = h * 0.8;
        let w = 0.4;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.06;
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, cy - 0.03),
                Point2::new(0.15, cy + 0.05),
                Point2::new(0.25, cy - 0.05),
                Point2::new(w, cy + 0.03),
            )
            .apply_slant(self.slant),
        );
        skel
    }
}
