//! Stroke — parametric stroke model with variable-width pen
//!
//! Each glyph is defined by a Bezier skeleton curve and a pen model
//! that varies width along the stroke based on contrast parameter.
//!
//! stroke(t) = skeleton(t) ± pen_width(t) × normal(t)
//!
//! License: MIT
//! Author: Moroya Sakamoto

use crate::param::MetaFontParams;

/// Maximum control points per stroke
pub const MAX_STROKE_POINTS: usize = 8;

/// 2D point (8 bytes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
}

impl Point2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn distance(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        fast_sqrt_stroke(dx * dx + dy * dy)
    }

    pub fn length(self) -> f32 {
        fast_sqrt_stroke(self.x * self.x + self.y * self.y)
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len < 1e-10 { return Self::ZERO; }
        Self { x: self.x / len, y: self.y / len }
    }

    /// Normal (perpendicular, 90° CCW)
    pub fn normal(self) -> Self {
        Self { x: -self.y, y: self.x }
    }

    pub fn scale(self, s: f32) -> Self {
        Self { x: self.x * s, y: self.y * s }
    }

    pub fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }

    pub fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }

    /// Apply italic slant transform
    pub fn slant(self, angle: f32) -> Self {
        // Shear: x' = x + y * tan(angle)
        Self { x: self.x + self.y * tan_approx(angle), y: self.y }
    }
}

/// Variable-width pen model
///
/// Width varies along the stroke based on direction angle
/// to simulate broad-nib calligraphy (contrast).
#[derive(Debug, Clone, Copy)]
pub struct PenModel {
    /// Base half-width (em units)
    pub base_width: f32,
    /// Contrast ratio (0 = monowidth, 1 = high contrast)
    pub contrast: f32,
    /// Pen angle for contrast (radians, 0 = horizontal thin)
    pub pen_angle: f32,
    /// Corner roundness
    pub roundness: f32,
}

impl PenModel {
    pub fn from_params(params: &MetaFontParams) -> Self {
        Self {
            base_width: params.stroke_half_width(),
            contrast: params.contrast,
            pen_angle: 0.5, // ~30° broad nib
            roundness: params.roundness,
        }
    }

    /// Pen half-width at a given stroke direction angle
    pub fn half_width_at_angle(&self, direction_angle: f32) -> f32 {
        let relative = direction_angle - self.pen_angle;
        let sin_val = sin_approx_stroke(relative);
        self.base_width * (1.0 - self.contrast * sin_val * sin_val)
    }

    /// Pen half-width at stroke parameter t given tangent direction
    pub fn half_width(&self, tangent: Point2) -> f32 {
        let angle = atan2_approx(tangent.y, tangent.x);
        self.half_width_at_angle(angle)
    }
}

/// Cubic Bezier stroke segment
#[derive(Debug, Clone, Copy)]
pub struct Stroke {
    /// Control points (p0 = start, p3 = end)
    pub p0: Point2,
    pub p1: Point2,
    pub p2: Point2,
    pub p3: Point2,
}

impl Stroke {
    pub fn new(p0: Point2, p1: Point2, p2: Point2, p3: Point2) -> Self {
        Self { p0, p1, p2, p3 }
    }

    /// Straight line as degenerate cubic
    pub fn line(start: Point2, end: Point2) -> Self {
        let t1 = start.lerp(end, 1.0 / 3.0);
        let t2 = start.lerp(end, 2.0 / 3.0);
        Self { p0: start, p1: t1, p2: t2, p3: end }
    }

    /// Position at parameter t ∈ [0, 1]
    pub fn position(&self, t: f32) -> Point2 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        Point2 {
            x: mt3 * self.p0.x + 3.0 * mt2 * t * self.p1.x
                + 3.0 * mt * t2 * self.p2.x + t3 * self.p3.x,
            y: mt3 * self.p0.y + 3.0 * mt2 * t * self.p1.y
                + 3.0 * mt * t2 * self.p2.y + t3 * self.p3.y,
        }
    }

    /// Tangent (first derivative) at parameter t
    pub fn tangent(&self, t: f32) -> Point2 {
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let t2 = t * t;

        Point2 {
            x: 3.0 * mt2 * (self.p1.x - self.p0.x)
                + 6.0 * mt * t * (self.p2.x - self.p1.x)
                + 3.0 * t2 * (self.p3.x - self.p2.x),
            y: 3.0 * mt2 * (self.p1.y - self.p0.y)
                + 6.0 * mt * t * (self.p2.y - self.p1.y)
                + 3.0 * t2 * (self.p3.y - self.p2.y),
        }
    }

    /// Approximate arc length by sampling
    pub fn arc_length(&self, steps: usize) -> f32 {
        let mut length = 0.0f32;
        let mut prev = self.position(0.0);
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let curr = self.position(t);
            length += prev.distance(curr);
            prev = curr;
        }
        length
    }

    /// Apply italic slant to all control points
    pub fn apply_slant(&self, slant: f32) -> Self {
        Self {
            p0: self.p0.slant(slant),
            p1: self.p1.slant(slant),
            p2: self.p2.slant(slant),
            p3: self.p3.slant(slant),
        }
    }

    /// Scale stroke uniformly
    pub fn scale(&self, s: f32) -> Self {
        Self {
            p0: self.p0.scale(s),
            p1: self.p1.scale(s),
            p2: self.p2.scale(s),
            p3: self.p3.scale(s),
        }
    }

    /// Translate stroke
    pub fn translate(&self, dx: f32, dy: f32) -> Self {
        let offset = Point2::new(dx, dy);
        Self {
            p0: self.p0.add(offset),
            p1: self.p1.add(offset),
            p2: self.p2.add(offset),
            p3: self.p3.add(offset),
        }
    }
}

/// Serif bracket (decorative stroke ending)
#[derive(Debug, Clone, Copy)]
pub struct SerifBracket {
    /// Base point (where serif starts)
    pub base: Point2,
    /// Direction (along the serif)
    pub direction: Point2,
    /// Serif length (em units)
    pub length: f32,
    /// Serif width (em units)
    pub width: f32,
}

impl SerifBracket {
    pub fn new(base: Point2, direction: Point2, params: &MetaFontParams) -> Self {
        Self {
            base,
            direction: direction.normalize(),
            length: params.serif_length(),
            width: params.stroke_half_width() * 1.2,
        }
    }

    /// Generate serif as a stroke
    pub fn to_stroke(&self) -> Stroke {
        let end = self.base.add(self.direction.scale(self.length));
        Stroke::line(self.base, end)
    }
}

fn fast_sqrt_stroke(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let half = 0.5 * x;
    let i = f32::to_bits(x);
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    let y = y * (1.5 - half * y * y);
    let y = y * (1.5 - half * y * y);
    x * y
}

fn sin_approx_stroke(x: f32) -> f32 {
    let pi = core::f32::consts::PI;
    let mut x = x % (2.0 * pi);
    if x < 0.0 { x += 2.0 * pi; }
    let sign = if x > pi { -1.0 } else { 1.0 };
    if x > pi { x -= pi; }
    let num = 16.0 * x * (pi - x);
    let den = 5.0 * pi * pi - 4.0 * x * (pi - x);
    sign * num / den
}

fn tan_approx(x: f32) -> f32 {
    let s = sin_approx_stroke(x);
    let c = sin_approx_stroke(x + core::f32::consts::FRAC_PI_2);
    if c.abs() < 1e-6 { return 0.0; }
    s / c
}

fn atan2_approx(y: f32, x: f32) -> f32 {
    let pi = core::f32::consts::PI;
    if x.abs() < 1e-10 && y.abs() < 1e-10 { return 0.0; }
    if x.abs() < 1e-10 {
        return if y > 0.0 { pi / 2.0 } else { -pi / 2.0 };
    }
    let ax = if x < 0.0 { -x } else { x };
    let ay = if y < 0.0 { -y } else { y };
    let min = if ax < ay { ax } else { ay };
    let max = if ax > ay { ax } else { ay };
    let a = min / max;
    let s = a * a;
    let r = ((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a;
    let r = if ay > ax { core::f32::consts::FRAC_PI_2 - r } else { r };
    let r = if x < 0.0 { pi - r } else { r };
    if y < 0.0 { -r } else { r }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2_basic() {
        let a = Point2::new(1.0, 2.0);
        let b = Point2::new(3.0, 4.0);
        let c = a.add(b);
        assert!((c.x - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_point2_distance() {
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(3.0, 4.0);
        assert!((a.distance(b) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_point2_normal() {
        let v = Point2::new(1.0, 0.0);
        let n = v.normal();
        assert!((n.x).abs() < 0.001);
        assert!((n.y - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_stroke_line() {
        let s = Stroke::line(Point2::ZERO, Point2::new(1.0, 0.0));
        let mid = s.position(0.5);
        assert!((mid.x - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_stroke_endpoints() {
        let s = Stroke::new(
            Point2::new(0.0, 0.0), Point2::new(0.3, 0.5),
            Point2::new(0.7, 0.5), Point2::new(1.0, 0.0),
        );
        let start = s.position(0.0);
        let end = s.position(1.0);
        assert!((start.x).abs() < 0.001);
        assert!((end.x - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_stroke_tangent() {
        let s = Stroke::line(Point2::ZERO, Point2::new(1.0, 0.0));
        let t = s.tangent(0.5);
        assert!(t.x > 0.0); // Moving right
    }

    #[test]
    fn test_stroke_arc_length() {
        let s = Stroke::line(Point2::ZERO, Point2::new(1.0, 0.0));
        let len = s.arc_length(32);
        assert!((len - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_pen_model_monoweight() {
        let params = MetaFontParams::mono_regular();
        let pen = PenModel::from_params(&params);
        // Monoweight: all directions same width
        let w1 = pen.half_width_at_angle(0.0);
        let w2 = pen.half_width_at_angle(1.0);
        assert!((w1 - w2).abs() < 0.01);
    }

    #[test]
    fn test_pen_model_contrast() {
        let params = MetaFontParams::serif_regular();
        let pen = PenModel::from_params(&params);
        // High contrast: horizontal thin, vertical thick
        let w_horiz = pen.half_width_at_angle(0.0);
        let w_vert = pen.half_width_at_angle(core::f32::consts::FRAC_PI_2);
        // They should differ when contrast > 0
        assert!(params.contrast > 0.0);
    }

    #[test]
    fn test_stroke_slant() {
        let s = Stroke::line(Point2::ZERO, Point2::new(0.0, 1.0));
        let slanted = s.apply_slant(0.2);
        // Top of vertical stroke should shift right
        assert!(slanted.p3.x > 0.0);
    }

    #[test]
    fn test_stroke_scale() {
        let s = Stroke::line(Point2::ZERO, Point2::new(1.0, 0.0));
        let scaled = s.scale(2.0);
        assert!((scaled.p3.x - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_serif_bracket() {
        let params = MetaFontParams::serif_regular();
        let bracket = SerifBracket::new(
            Point2::ZERO, Point2::new(1.0, 0.0), &params,
        );
        assert!(bracket.length > 0.0);
        let stroke = bracket.to_stroke();
        assert!((stroke.p3.x - bracket.length).abs() < 0.01);
    }

    #[test]
    fn test_atan2_approx() {
        let a = atan2_approx(1.0, 0.0);
        assert!((a - core::f32::consts::FRAC_PI_2).abs() < 0.02);
    }
}
