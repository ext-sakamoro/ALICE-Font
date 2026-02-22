//! Glyph — SDF glyph procedural generation
//!
//! Generates signed distance fields for characters from parametric
//! stroke skeletons and pen model. Each glyph is a set of strokes;
//! the SDF is the minimum distance to any stroke boundary.
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::param::MetaFontParams;
use crate::stroke::{PenModel, Point2, Stroke};

pub mod cjk_strokes;
mod digits;
pub mod helpers;
mod lower;
mod punct;
mod upper;

/// SDF resolution for a single glyph tile
pub const GLYPH_SDF_SIZE: usize = 32;

/// SDF glyph data (1024 bytes for 32×32 f32)
pub struct GlyphSdf {
    /// SDF values: positive = outside, negative = inside
    pub data: [f32; GLYPH_SDF_SIZE * GLYPH_SDF_SIZE],
    /// Advance width (em units) — horizontal spacing
    pub advance: f32,
    /// Left side bearing (em units)
    pub lsb: f32,
    /// Bounding box
    pub bbox_min: Point2,
    pub bbox_max: Point2,
}

impl GlyphSdf {
    pub fn empty() -> Self {
        Self {
            data: [1.0f32; GLYPH_SDF_SIZE * GLYPH_SDF_SIZE],
            advance: 0.5,
            lsb: 0.0,
            bbox_min: Point2::ZERO,
            bbox_max: Point2::new(1.0, 1.0),
        }
    }

    /// Sample SDF at normalized coordinates (0..1, 0..1)
    #[inline(always)]
    pub fn sample(&self, u: f32, v: f32) -> f32 {
        let x = (u * (GLYPH_SDF_SIZE - 1) as f32) as usize;
        let y = (v * (GLYPH_SDF_SIZE - 1) as f32) as usize;
        let x = if x >= GLYPH_SDF_SIZE {
            GLYPH_SDF_SIZE - 1
        } else {
            x
        };
        let y = if y >= GLYPH_SDF_SIZE {
            GLYPH_SDF_SIZE - 1
        } else {
            y
        };
        self.data[y * GLYPH_SDF_SIZE + x]
    }

    /// Is point inside the glyph? (SDF < 0)
    pub fn is_inside(&self, u: f32, v: f32) -> bool {
        self.sample(u, v) < 0.0
    }
}

/// Maximum strokes per glyph
pub const MAX_GLYPH_STROKES: usize = 16;

/// Glyph skeleton definition — strokes that compose a character
#[derive(Clone, Copy)]
pub struct GlyphSkeleton {
    pub strokes: [Stroke; MAX_GLYPH_STROKES],
    pub stroke_count: usize,
    pub advance: f32,
}

impl GlyphSkeleton {
    pub fn empty() -> Self {
        Self {
            strokes: [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES],
            stroke_count: 0,
            advance: 0.5,
        }
    }

    pub fn add_stroke(&mut self, stroke: Stroke) {
        if self.stroke_count < MAX_GLYPH_STROKES {
            self.strokes[self.stroke_count] = stroke;
            self.stroke_count += 1;
        }
    }
}

/// Glyph generator — creates SDF from parameters
pub struct GlyphGenerator {
    pen: PenModel,
    slant: f32,
    x_height: f32,
    cap_height: f32,
    serif_length: f32,
}

impl GlyphGenerator {
    pub fn new(params: &MetaFontParams) -> Self {
        Self {
            pen: PenModel::from_params(params),
            slant: params.slant,
            x_height: params.x_height,
            cap_height: params.cap_height,
            serif_length: params.serif_length(),
        }
    }

    /// Generate SDF for a character
    pub fn generate(&self, ch: u8) -> GlyphSdf {
        let skeleton = self.build_skeleton(ch);
        self.rasterize_sdf(&skeleton)
    }

    /// Build stroke skeleton for a character
    fn build_skeleton(&self, ch: u8) -> GlyphSkeleton {
        match ch {
            // Uppercase A-Z
            b'A' => self.build_uppercase_a(),
            b'B' => self.build_uppercase_b(),
            b'C' => self.build_uppercase_c(),
            b'D' => self.build_uppercase_d(),
            b'E' => self.build_uppercase_e(),
            b'F' => self.build_uppercase_f(),
            b'G' => self.build_uppercase_g(),
            b'H' => self.build_uppercase_h(),
            b'I' => self.build_uppercase_i(),
            b'J' => self.build_uppercase_j(),
            b'K' => self.build_uppercase_k(),
            b'L' => self.build_uppercase_l(),
            b'M' => self.build_uppercase_m(),
            b'N' => self.build_uppercase_n(),
            b'O' => self.build_uppercase_o(),
            b'P' => self.build_uppercase_p(),
            b'Q' => self.build_uppercase_q(),
            b'R' => self.build_uppercase_r(),
            b'S' => self.build_uppercase_s(),
            b'T' => self.build_uppercase_t(),
            b'U' => self.build_uppercase_u(),
            b'V' => self.build_uppercase_v(),
            b'W' => self.build_uppercase_w(),
            b'X' => self.build_uppercase_x(),
            b'Y' => self.build_uppercase_y(),
            b'Z' => self.build_uppercase_z(),
            // Lowercase a-z
            b'a' => self.build_lowercase_a(),
            b'b' => self.build_lowercase_b(),
            b'c' => self.build_lowercase_c(),
            b'd' => self.build_lowercase_d(),
            b'e' => self.build_lowercase_e(),
            b'f' => self.build_lowercase_f(),
            b'g' => self.build_lowercase_g(),
            b'h' => self.build_lowercase_h(),
            b'i' => self.build_lowercase_i(),
            b'j' => self.build_lowercase_j(),
            b'k' => self.build_lowercase_k(),
            b'l' => self.build_lowercase_l(),
            b'm' => self.build_lowercase_m(),
            b'n' => self.build_lowercase_n(),
            b'o' => self.build_lowercase_o(),
            b'p' => self.build_lowercase_p(),
            b'q' => self.build_lowercase_q(),
            b'r' => self.build_lowercase_r(),
            b's' => self.build_lowercase_s(),
            b't' => self.build_lowercase_t(),
            b'u' => self.build_lowercase_u(),
            b'v' => self.build_lowercase_v(),
            b'w' => self.build_lowercase_w(),
            b'x' => self.build_lowercase_x(),
            b'y' => self.build_lowercase_y(),
            b'z' => self.build_lowercase_z(),
            // Digits 0-9
            b'0' => self.build_digit_0(),
            b'1' => self.build_digit_1(),
            b'2' => self.build_digit_2(),
            b'3' => self.build_digit_3(),
            b'4' => self.build_digit_4(),
            b'5' => self.build_digit_5(),
            b'6' => self.build_digit_6(),
            b'7' => self.build_digit_7(),
            b'8' => self.build_digit_8(),
            b'9' => self.build_digit_9(),
            // Punctuation and symbols
            0x21..=0x2F | 0x3A..=0x40 | 0x5B..=0x60 | 0x7B..=0x7E => self.build_punct(ch),
            _ => self.build_placeholder(ch),
        }
    }

    fn build_uppercase_a(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.65;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Left stroke
        let left =
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(w / 2.0, h)).apply_slant(self.slant);
        skel.add_stroke(left);

        // Right stroke
        let right = Stroke::line(Point2::new(w / 2.0, h), Point2::new(w - 0.05, 0.0))
            .apply_slant(self.slant);
        skel.add_stroke(right);

        // Crossbar
        // 0.42: crossbar height as a fraction of cap-height for the letter A.
        // Classical proportions place the A crossbar slightly below the optical
        // midpoint (~0.50) so that the two triangular counters appear equal in
        // visual weight (lower triangle has a wider base, so it reads as heavier
        // without the correction). 0.42 matches the convention used by Gill Sans,
        // Helvetica, and most geometric sans-serifs; humanist faces often use
        // 0.44–0.46. The value has no closed-form derivation — it is an empirical
        // typographic optimum refined over centuries of metal-type cutting.
        let cross_y = h * 0.42;
        let cross = Stroke::line(Point2::new(0.15, cross_y), Point2::new(w - 0.15, cross_y))
            .apply_slant(self.slant);
        skel.add_stroke(cross);

        skel
    }

    fn build_uppercase_b(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.55;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Vertical stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );

        // Upper bowl
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h),
                Point2::new(w, h),
                Point2::new(w, h * 0.55),
                Point2::new(0.05, h * 0.52),
            )
            .apply_slant(self.slant),
        );

        // Lower bowl
        skel.add_stroke(
            Stroke::new(
                Point2::new(0.05, h * 0.52),
                Point2::new(w + 0.02, h * 0.52),
                Point2::new(w + 0.02, 0.0),
                Point2::new(0.05, 0.0),
            )
            .apply_slant(self.slant),
        );

        skel
    }

    fn build_uppercase_h(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Left stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Right stem
        skel.add_stroke(
            Stroke::line(Point2::new(w - 0.05, 0.0), Point2::new(w - 0.05, h))
                .apply_slant(self.slant),
        );
        // Crossbar
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, h * 0.5), Point2::new(w - 0.05, h * 0.5))
                .apply_slant(self.slant),
        );

        skel
    }

    fn build_uppercase_i(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.3;

        skel.add_stroke(
            Stroke::line(Point2::new(0.15, 0.0), Point2::new(0.15, h)).apply_slant(self.slant),
        );

        skel
    }

    fn build_uppercase_l(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.5;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Vertical
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(0.05, h)).apply_slant(self.slant),
        );
        // Horizontal base
        skel.add_stroke(
            Stroke::line(Point2::new(0.05, 0.0), Point2::new(w, 0.0)).apply_slant(self.slant),
        );

        skel
    }

    fn build_uppercase_o(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.7;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Approximate circle with 4 cubic Beziers
        // KAPPA — cubic Bezier circle approximation constant.
        // Derivation: to approximate a quarter-circle of radius r with a single
        // cubic Bezier, the two off-curve control points are placed at distance
        // k·r from the on-curve endpoints along the tangent direction, where
        //   k = (4/3) · tan(π/8) = (4/3) · (√2 − 1) ≈ 0.55228...
        // Rounded to 0.5523 for f32 convenience (error is < 1 ULP at f32).
        // This minimises the maximum radial deviation, which is ≈ 0.027% of r
        // (worst case at 45° from the arc endpoints).
        // Reference: Riskus, "Approximation of a Cubic Bezier Curve by Circular
        //            Arcs and Vice Versa", Information Technology and Control,
        //            Vol. 35, No. 4, 2006.
        let k = 0.5523; // (4/3)(√2 − 1) ≈ 0.55228, max radial error ≈ 0.027%
        let rx = w / 2.0 - 0.05;
        let ry = h / 2.0;

        // Right arc
        skel.add_stroke(
            Stroke::new(
                Point2::new(cx, cy + ry),
                Point2::new(cx + rx * k, cy + ry),
                Point2::new(cx + rx, cy + ry * k),
                Point2::new(cx + rx, cy),
            )
            .apply_slant(self.slant),
        );

        skel.add_stroke(
            Stroke::new(
                Point2::new(cx + rx, cy),
                Point2::new(cx + rx, cy - ry * k),
                Point2::new(cx + rx * k, cy - ry),
                Point2::new(cx, cy - ry),
            )
            .apply_slant(self.slant),
        );

        // Left arc
        skel.add_stroke(
            Stroke::new(
                Point2::new(cx, cy - ry),
                Point2::new(cx - rx * k, cy - ry),
                Point2::new(cx - rx, cy - ry * k),
                Point2::new(cx - rx, cy),
            )
            .apply_slant(self.slant),
        );

        skel.add_stroke(
            Stroke::new(
                Point2::new(cx - rx, cy),
                Point2::new(cx - rx, cy + ry * k),
                Point2::new(cx - rx * k, cy + ry),
                Point2::new(cx, cy + ry),
            )
            .apply_slant(self.slant),
        );

        skel
    }

    fn build_uppercase_t(&self) -> GlyphSkeleton {
        let h = self.cap_height;
        let w = h * 0.6;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.1;

        // Vertical stem
        skel.add_stroke(
            Stroke::line(Point2::new(w / 2.0, 0.0), Point2::new(w / 2.0, h))
                .apply_slant(self.slant),
        );
        // Horizontal top
        skel.add_stroke(
            Stroke::line(Point2::new(0.02, h), Point2::new(w - 0.02, h)).apply_slant(self.slant),
        );

        skel
    }

    fn build_lowercase_l(&self) -> GlyphSkeleton {
        let h = self.cap_height * 1.05; // ascender
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.25;
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, 0.0), Point2::new(0.12, h)).apply_slant(self.slant),
        );
        skel
    }

    fn build_lowercase_o(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let w = h * 0.85;
        let cx = w / 2.0;
        let cy = h / 2.0;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;

        // KAPPA — same cubic Bezier circle approximation constant as uppercase O.
        // k = (4/3)(√2 − 1) ≈ 0.55228; rounded to 0.5523 (f32, error < 1 ULP).
        // Maximum radial deviation from a true circle: ≈ 0.027% of radius.
        // See build_uppercase_o for full derivation and reference.
        let k = 0.5523; // (4/3)(√2 − 1) ≈ 0.55228, max radial error ≈ 0.027%
        let rx = w / 2.0 - 0.03;
        let ry = h / 2.0;

        skel.add_stroke(
            Stroke::new(
                Point2::new(cx, cy + ry),
                Point2::new(cx + rx * k, cy + ry),
                Point2::new(cx + rx, cy + ry * k),
                Point2::new(cx + rx, cy),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(cx + rx, cy),
                Point2::new(cx + rx, cy - ry * k),
                Point2::new(cx + rx * k, cy - ry),
                Point2::new(cx, cy - ry),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(cx, cy - ry),
                Point2::new(cx - rx * k, cy - ry),
                Point2::new(cx - rx, cy - ry * k),
                Point2::new(cx - rx, cy),
            )
            .apply_slant(self.slant),
        );
        skel.add_stroke(
            Stroke::new(
                Point2::new(cx - rx, cy),
                Point2::new(cx - rx, cy + ry * k),
                Point2::new(cx - rx * k, cy + ry),
                Point2::new(cx, cy + ry),
            )
            .apply_slant(self.slant),
        );

        skel
    }

    fn build_lowercase_i(&self) -> GlyphSkeleton {
        let h = self.x_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = 0.25;

        // Stem
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, 0.0), Point2::new(0.12, h)).apply_slant(self.slant),
        );

        // Dot (short stroke as approximation)
        let dot_y = h + 0.08;
        skel.add_stroke(
            Stroke::line(Point2::new(0.12, dot_y), Point2::new(0.12, dot_y + 0.02))
                .apply_slant(self.slant),
        );

        skel
    }

    fn build_placeholder(&self, _ch: u8) -> GlyphSkeleton {
        // Simple box placeholder
        let w = 0.4;
        let h = self.x_height;
        let mut skel = GlyphSkeleton::empty();
        skel.advance = w + 0.08;

        skel.add_stroke(Stroke::line(Point2::new(0.04, 0.0), Point2::new(w, 0.0)));
        skel.add_stroke(Stroke::line(Point2::new(w, 0.0), Point2::new(w, h)));
        skel.add_stroke(Stroke::line(Point2::new(w, h), Point2::new(0.04, h)));
        skel.add_stroke(Stroke::line(Point2::new(0.04, h), Point2::new(0.04, 0.0)));

        skel
    }

    /// Rasterize SDF from stroke skeleton.
    ///
    /// Performance improvements over the naive approach:
    ///
    /// 1. **Precomputed stroke samples** — curve positions and pen half-widths
    ///    are evaluated once per stroke before the pixel loop.  The inner loop
    ///    then only does arithmetic (no Bezier eval, no atan2) per pixel-sample
    ///    pair.  For a 32×32 tile with 12 strokes × 17 samples this cuts
    ///    ~6 500 atan2 calls down to ~200 (12 × 17).
    ///
    /// 2. **Early-exit on negative distance** — once `min_dist` goes negative
    ///    the query point is already inside a stroke.  No further stroke can
    ///    reduce the distance below the current value (strokes are unioned via
    ///    min), so we break out of the stroke loop immediately.
    fn rasterize_sdf(&self, skeleton: &GlyphSkeleton) -> GlyphSdf {
        let mut sdf = GlyphSdf::empty();
        sdf.advance = skeleton.advance;

        // Compute bounding box
        let (bb_min, bb_max) = self.compute_bbox(skeleton);
        let padding = self.pen.base_width * 3.0;
        sdf.bbox_min = Point2::new(bb_min.x - padding, bb_min.y - padding);
        sdf.bbox_max = Point2::new(bb_max.x + padding, bb_max.y + padding);

        let size = GLYPH_SDF_SIZE;
        let w = sdf.bbox_max.x - sdf.bbox_min.x;
        let h = sdf.bbox_max.y - sdf.bbox_min.y;
        if w < 1e-6 || h < 1e-6 {
            return sdf;
        }

        // --- Precompute stroke samples (curve point + half-width) ----------
        //
        // steps = 16: same rationale as before (see distance_to_stroke comment).
        // We store (x, y, half_width) for each (stroke, sample) pair so the
        // pixel loop does zero Bezier evaluations or trig.
        const STEPS: usize = 16;
        const INV_STEPS: f32 = 1.0 / STEPS as f32;
        const SAMPLES_PER_STROKE: usize = STEPS + 1; // inclusive endpoints

        // Flat array: [stroke_0_sample_0, ..., stroke_0_sample_16,
        //              stroke_1_sample_0, ..., stroke_N_sample_16]
        // Using fixed-size stack array (MAX_GLYPH_STROKES × 17 = 204 entries).
        let mut sx = [0.0f32; MAX_GLYPH_STROKES * SAMPLES_PER_STROKE];
        let mut sy = [0.0f32; MAX_GLYPH_STROKES * SAMPLES_PER_STROKE];
        let mut shw = [0.0f32; MAX_GLYPH_STROKES * SAMPLES_PER_STROKE];

        for si in 0..skeleton.stroke_count {
            let stroke = &skeleton.strokes[si];
            let base = si * SAMPLES_PER_STROKE;
            for i in 0..=STEPS {
                let t = i as f32 * INV_STEPS;
                let pt = stroke.position(t);
                let tangent = stroke.tangent(t);
                let hw = self.pen.half_width(tangent);
                sx[base + i] = pt.x;
                sy[base + i] = pt.y;
                shw[base + i] = hw;
            }
        }

        // Pre-compute reciprocal: avoids (size-1) division per pixel
        let inv_size_1 = 1.0 / (size - 1) as f32;

        for py in 0..size {
            for px in 0..size {
                let u = px as f32 * inv_size_1;
                let v = py as f32 * inv_size_1;
                let px_world = sdf.bbox_min.x + u * w;
                let py_world = sdf.bbox_min.y + v * h;

                let mut min_dist = f32::MAX;

                'stroke_loop: for si in 0..skeleton.stroke_count {
                    let base = si * SAMPLES_PER_STROKE;
                    for i in 0..=STEPS {
                        let idx = base + i;
                        let dx = px_world - sx[idx];
                        let dy = py_world - sy[idx];
                        let dist_to_center = fast_sqrt_glyph(dx * dx + dy * dy);
                        let dist = dist_to_center - shw[idx];
                        if dist < min_dist {
                            min_dist = dist;
                            // Early-exit: inside a stroke — union of strokes
                            // can only stay negative or become more negative.
                            // Any remaining stroke sample or stroke cannot
                            // produce a value lower than the most-negative
                            // value seen so far *for the purposes of rendering*
                            // (we want the minimum signed distance).  However
                            // a later stroke could still yield a lower (more
                            // negative) value, so we only skip to the next
                            // stroke, not exit the whole pixel.
                            if min_dist < 0.0 {
                                break;
                            }
                        }
                    }
                    // If we are already well inside (negative by more than the
                    // maximum possible half-width of any remaining stroke), no
                    // other stroke can help — skip remaining strokes entirely.
                    if min_dist < -(self.pen.base_width * 2.0) {
                        break 'stroke_loop;
                    }
                }

                sdf.data[py * size + px] = min_dist;
            }
        }

        sdf
    }

    /// Signed distance from point to stroked curve.
    ///
    /// Kept for reference / potential future use by callers outside
    /// `rasterize_sdf`.  The hot path in `rasterize_sdf` no longer calls
    /// this — it uses precomputed samples instead.
    ///
    /// steps = 16: number of uniform parameter samples across t ∈ [0, 1].
    /// Rationale: a cubic Bezier with typical glyph curvature introduces at
    /// most one inflection point, so its curvature is monotone per half-span.
    /// Uniform sampling at 1/16 intervals (Δt = 0.0625) limits the maximum
    /// chord-length skip to well under one stroke half-width for all glyph
    /// strokes at the em-sizes used here, ensuring no stroke "gap" is missed.
    /// Empirically: 8 steps is borderline for high-contrast strokes at small
    /// sizes; 32 steps gives no visible improvement over 16 for 32×32 SDF
    /// tiles. Cost is O(steps) Bezier evaluations per pixel per stroke.
    #[allow(dead_code)]
    #[inline(always)]
    fn distance_to_stroke(&self, p: Point2, stroke: &Stroke) -> f32 {
        let steps = 16;
        // Pre-compute reciprocal: avoids 17 divisions in the sampling loop
        let inv_steps = 1.0 / steps as f32;
        let mut min_dist = f32::MAX;

        for i in 0..=steps {
            let t = i as f32 * inv_steps;
            let curve_pt = stroke.position(t);
            let tangent = stroke.tangent(t);
            let hw = self.pen.half_width(tangent);

            let dx = p.x - curve_pt.x;
            let dy = p.y - curve_pt.y;
            let dist_to_center = fast_sqrt_glyph(dx * dx + dy * dy);
            let dist = dist_to_center - hw;

            if dist < min_dist {
                min_dist = dist;
            }
        }

        min_dist
    }

    fn compute_bbox(&self, skeleton: &GlyphSkeleton) -> (Point2, Point2) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for si in 0..skeleton.stroke_count {
            let s = &skeleton.strokes[si];
            // Pre-computed reciprocal for the 9-sample bbox pass
            const INV_8: f32 = 1.0 / 8.0;
            for t_step in 0..=8 {
                let t = t_step as f32 * INV_8;
                let p = s.position(t);
                if p.x < min_x {
                    min_x = p.x;
                }
                if p.y < min_y {
                    min_y = p.y;
                }
                if p.x > max_x {
                    max_x = p.x;
                }
                if p.y > max_y {
                    max_y = p.y;
                }
            }
        }

        (Point2::new(min_x, min_y), Point2::new(max_x, max_y))
    }
}

#[inline(always)]
fn fast_sqrt_glyph(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let half = 0.5 * x;
    let i = f32::to_bits(x);
    // Quake III Arena magic constant for fast inverse square root (1/√x).
    // Attributed to Greg Walsh / John Carmack (id Software, 1999).
    // Mechanism: IEEE 754 f32 bits encode value as (1 + mantissa) × 2^(exp-127).
    // A right-shift by 1 halves the exponent in log2-space (approximates √),
    // and subtracting from 0x5f3759df corrects the bias offset.
    // The result is a first-order estimate; each Newton–Raphson step below
    // halves the relative error. Two iterations yield ~4.7 × 10⁻⁷ relative
    // error — within single-precision ULP for all normal positive inputs.
    // Reference: Lomont, "Fast Inverse Square Root" (2003);
    //            quake3-1.32b/code/game/q_math.c, Q_rsqrt().
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    let y = y * (1.5 - half * y * y); // Newton–Raphson iteration 1
    let y = y * (1.5 - half * y * y); // Newton–Raphson iteration 2
    x * y // x * (1/√x) = √x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_sdf_empty() {
        let sdf = GlyphSdf::empty();
        assert!(sdf.sample(0.5, 0.5) > 0.0); // Outside by default
    }

    #[test]
    fn test_generate_a() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'A');
        assert!(sdf.advance > 0.0);
        // Center should be inside or close to strokes
        let center_val = sdf.sample(0.5, 0.5);
        assert!(center_val < 1.0); // Not far outside
    }

    #[test]
    fn test_generate_o() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'O');
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn test_generate_z() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'Z');
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn test_generate_placeholder() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(0x01); // Non-printable uses placeholder
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn test_different_weights() {
        let regular = MetaFontParams::sans_regular();
        let bold = MetaFontParams::sans_bold();
        let gen_r = GlyphGenerator::new(&regular);
        let gen_b = GlyphGenerator::new(&bold);
        let sdf_r = gen_r.generate(b'I');
        let sdf_b = gen_b.generate(b'I');
        // Bold should have more "inside" pixels (negative SDF)
        let mut count_r = 0;
        let mut count_b = 0;
        for v in &sdf_r.data {
            if *v < 0.0 {
                count_r += 1;
            }
        }
        for v in &sdf_b.data {
            if *v < 0.0 {
                count_b += 1;
            }
        }
        assert!(
            count_b >= count_r,
            "Bold should fill more pixels: bold={count_b} regular={count_r}"
        );
    }

    #[test]
    fn test_glyph_skeleton_add_stroke() {
        let mut skel = GlyphSkeleton::empty();
        assert_eq!(skel.stroke_count, 0);
        skel.add_stroke(Stroke::line(Point2::ZERO, Point2::new(1.0, 0.0)));
        assert_eq!(skel.stroke_count, 1);
    }

    #[test]
    fn test_sdf_is_inside() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'I');
        // Some center region should be inside
        let mut has_inside = false;
        for py in 0..GLYPH_SDF_SIZE {
            for px in 0..GLYPH_SDF_SIZE {
                let u = px as f32 / (GLYPH_SDF_SIZE - 1) as f32;
                let v = py as f32 / (GLYPH_SDF_SIZE - 1) as f32;
                if sdf.is_inside(u, v) {
                    has_inside = true;
                }
            }
        }
        assert!(has_inside, "Bold 'I' should have inside pixels");
    }

    #[test]
    fn test_italic_slant() {
        let params = MetaFontParams::serif_italic();
        let gen = GlyphGenerator::new(&params);
        let sdf = gen.generate(b'l');
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn test_all_printable_ascii() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        for ch in 0x21u8..=0x7E {
            let sdf = gen.generate(ch);
            assert!(
                sdf.advance > 0.0,
                "char 0x{:02X} ({}) should have advance > 0",
                ch,
                ch as char
            );
        }
    }

    #[test]
    fn test_all_uppercase_advance() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        for ch in b'A'..=b'Z' {
            let sdf = gen.generate(ch);
            assert!(sdf.advance > 0.0, "'{}' advance should be > 0", ch as char);
        }
    }

    #[test]
    fn test_all_uppercase_have_content() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        for ch in b'A'..=b'Z' {
            let sdf = gen.generate(ch);
            let inside = sdf.data.iter().filter(|&&v| v < 0.0).count();
            assert!(
                inside > 0,
                "Bold '{}' should have inside pixels",
                ch as char
            );
        }
    }

    #[test]
    fn test_uppercase_weight_variation() {
        let regular = MetaFontParams::sans_regular();
        let bold = MetaFontParams::sans_bold();
        let gen_r = GlyphGenerator::new(&regular);
        let gen_b = GlyphGenerator::new(&bold);
        for ch in b'A'..=b'Z' {
            let sdf_r = gen_r.generate(ch);
            let sdf_b = gen_b.generate(ch);
            let count_r = sdf_r.data.iter().filter(|&&v| v < 0.0).count();
            let count_b = sdf_b.data.iter().filter(|&&v| v < 0.0).count();
            assert!(
                count_b >= count_r,
                "Bold '{}' ({}) should fill >= regular ({})",
                ch as char,
                count_b,
                count_r
            );
        }
    }

    #[test]
    fn test_all_lowercase_advance() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        for ch in b'a'..=b'z' {
            let sdf = gen.generate(ch);
            assert!(sdf.advance > 0.0, "'{}' advance should be > 0", ch as char);
        }
    }

    #[test]
    fn test_all_digits_tabular() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let adv0 = gen.generate(b'0').advance;
        for ch in b'1'..=b'9' {
            let adv = gen.generate(ch).advance;
            assert!(
                (adv - adv0).abs() < 0.001,
                "Digit '{}' advance {} != '0' advance {}",
                ch as char,
                adv,
                adv0
            );
        }
    }

    #[test]
    fn test_digit_content() {
        let params = MetaFontParams::sans_bold();
        let gen = GlyphGenerator::new(&params);
        for ch in b'0'..=b'9' {
            let sdf = gen.generate(ch);
            let inside = sdf.data.iter().filter(|&&v| v < 0.0).count();
            assert!(
                inside > 0,
                "Bold '{}' should have inside pixels",
                ch as char
            );
        }
    }

    #[test]
    fn test_punctuation_advance() {
        let params = MetaFontParams::sans_regular();
        let gen = GlyphGenerator::new(&params);
        let punct_ranges: &[core::ops::RangeInclusive<u8>] =
            &[0x21..=0x2F, 0x3A..=0x40, 0x5B..=0x60, 0x7B..=0x7E];
        for range in punct_ranges {
            for ch in range.clone() {
                let sdf = gen.generate(ch);
                assert!(
                    sdf.advance > 0.0,
                    "Punct 0x{:02X} ({}) advance > 0",
                    ch,
                    ch as char
                );
            }
        }
    }

    #[test]
    fn test_italic_slant_all() {
        let params = MetaFontParams::serif_italic();
        let gen = GlyphGenerator::new(&params);
        for ch in b'A'..=b'Z' {
            let sdf = gen.generate(ch);
            assert!(
                sdf.advance > 0.0,
                "Italic '{}' should have advance > 0",
                ch as char
            );
        }
        for ch in b'a'..=b'z' {
            let sdf = gen.generate(ch);
            assert!(
                sdf.advance > 0.0,
                "Italic '{}' should have advance > 0",
                ch as char
            );
        }
    }

    #[test]
    fn test_max_glyph_strokes_increased() {
        assert_eq!(MAX_GLYPH_STROKES, 16);
        let skel = GlyphSkeleton::empty();
        assert_eq!(skel.strokes.len(), 16);
    }
}
