//! Analytic oracles — closed-form / independent-reference checks for the
//! numeric laws in ALICE-Font.
//!
//! Every expected value here comes from a closed form or an independent f64
//! reference written in this file; none is produced by calling the crate
//! function under test.  `src/*` unit tests pin behaviour; this file pins
//! *truth*.  Preset `MetaFontParams::sans_regular()` is the default path a
//! consumer takes (the crate has no `Default`), so it is used throughout.
//!
//! Oracle sources:
//! - straight stroke = degenerate cubic ⇒ position / tangent / arc length are
//!   the segment closed form
//! - cubic arc length: f64 composite Simpson of |B'(t)| (independent)
//! - fast inverse sqrt: f64 `hypot` (Lomont 2003, 2 Newton steps ⇒ ≤ 4.7e-6 rel)
//! - pen: r(θ) = w·(1 − c·sin²(θ − φ)); Bhaskara I sine bound |Δsin| ≤ 1.63e-3
//! - slant: shear x' = x + y·tan(α); Bhaskara-derived |Δtan| ≤ 2.9e-3 for |α| ≤ 0.5
//! - rasterised SDF of a straight stroke = capsule (segment ⊕ disc) closed form
//! - union of strokes = pointwise min

use alice_font::glyph::GlyphSkeleton;
use alice_font::stroke::{PenModel, Point2, Stroke};
use alice_font::{GlyphGenerator, GlyphSdf, MetaFontParams};

// ───────────────────────── helpers (independent of the crate) ─────────────

/// Closed-form distance from `p` to the segment `a`–`b` (f64).
fn segment_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (p.0 - a.0, p.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 == 0.0 {
        0.0
    } else {
        ((apx * abx + apy * aby) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (a.0 + t * abx - p.0, a.1 + t * aby - p.1);
    (cx * cx + cy * cy).sqrt()
}

/// Cubic Bezier derivative magnitude in f64 (independent of `Stroke`).
fn cubic_speed(p: [(f64, f64); 4], t: f64) -> f64 {
    let mt = 1.0 - t;
    let dx = 3.0 * mt * mt * (p[1].0 - p[0].0)
        + 6.0 * mt * t * (p[2].0 - p[1].0)
        + 3.0 * t * t * (p[3].0 - p[2].0);
    let dy = 3.0 * mt * mt * (p[1].1 - p[0].1)
        + 6.0 * mt * t * (p[2].1 - p[1].1)
        + 3.0 * t * t * (p[3].1 - p[2].1);
    (dx * dx + dy * dy).sqrt()
}

/// Composite Simpson arc length with `n` (even) panels — the reference.
fn cubic_arc_length_simpson(p: [(f64, f64); 4], n: usize) -> f64 {
    assert!(n.is_multiple_of(2));
    let h = 1.0 / n as f64;
    let mut s = cubic_speed(p, 0.0) + cubic_speed(p, 1.0);
    for i in 1..n {
        let w = if i % 2 == 1 { 4.0 } else { 2.0 };
        s += w * cubic_speed(p, i as f64 * h);
    }
    s * h / 3.0
}

fn sans_mono_pen() -> MetaFontParams {
    // contrast 0 ⇒ the pen half-width is direction independent, so a straight
    // stroke is exactly a capsule of radius `stroke_half_width()`.
    let mut p = MetaFontParams::sans_regular();
    p.contrast = 0.0;
    p
}

/// World coordinates of pixel (px, py) exactly as `GlyphSdf` lays them out
/// (u = px / (N − 1) across the padded bbox).
fn pixel_world(sdf: &GlyphSdf, px: usize, py: usize) -> (f64, f64) {
    let n = (alice_font::glyph::GLYPH_SDF_SIZE - 1) as f64;
    let w = (sdf.bbox_max.x - sdf.bbox_min.x) as f64;
    let h = (sdf.bbox_max.y - sdf.bbox_min.y) as f64;
    (
        sdf.bbox_min.x as f64 + px as f64 / n * w,
        sdf.bbox_min.y as f64 + py as f64 / n * h,
    )
}

// ───────────────────────── stroke geometry ────────────────────────────────

#[test]
fn line_stroke_position_and_tangent_are_the_segment_closed_form() {
    let a = Point2::new(0.13, -0.2);
    let b = Point2::new(0.87, 0.61);
    let s = Stroke::line(a, b);
    for i in 0..=100 {
        let t = i as f32 / 100.0;
        let p = s.position(t);
        // oracle: a + t (b − a)
        let ex = a.x as f64 + t as f64 * (b.x - a.x) as f64;
        let ey = a.y as f64 + t as f64 * (b.y - a.y) as f64;
        assert!(
            (p.x as f64 - ex).abs() < 2e-6,
            "position.x t={t}: {} vs {ex}",
            p.x
        );
        assert!(
            (p.y as f64 - ey).abs() < 2e-6,
            "position.y t={t}: {} vs {ey}",
            p.y
        );
        // oracle: d/dt of a + t(b−a) = b − a, constant along the line
        let d = s.tangent(t);
        assert!(
            ((d.x - (b.x - a.x)) as f64).abs() < 2e-6,
            "tangent.x t={t}: {}",
            d.x
        );
        assert!(
            ((d.y - (b.y - a.y)) as f64).abs() < 2e-6,
            "tangent.y t={t}: {}",
            d.y
        );
    }
}

#[test]
fn line_stroke_arc_length_is_the_segment_length_for_every_step_count() {
    let a = Point2::new(0.0, 0.0);
    let b = Point2::new(0.6, 0.8); // length 1.0 exactly
    let s = Stroke::line(a, b);
    // precision parameter sweep: a straight line has no discretisation error,
    // so the answer must not depend on `steps`
    for steps in [1usize, 2, 3, 4, 16, 64, 256, 1024] {
        let l = s.arc_length(steps) as f64;
        assert!(
            (l - 1.0).abs() < 2e-5,
            "arc_length(steps={steps}) = {l}, oracle 1.0"
        );
    }
}

#[test]
fn cubic_arc_length_matches_simpson_reference_and_is_monotone_in_steps() {
    let ctrl = [(0.0, 0.0), (0.3, 0.9), (0.7, -0.4), (1.0, 0.5)];
    let s = Stroke::new(
        Point2::new(0.0, 0.0),
        Point2::new(0.3, 0.9),
        Point2::new(0.7, -0.4),
        Point2::new(1.0, 0.5),
    );
    // oracle: f64 composite Simpson, 20 000 panels (converged far below 1e-9)
    let reference = cubic_arc_length_simpson(ctrl, 20_000);
    let mut prev = 0.0f64;
    for steps in [1usize, 2, 4, 8, 16, 64, 256, 1024] {
        let l = s.arc_length(steps) as f64;
        // chord length is a lower bound of arc length and refines monotonically
        assert!(
            l <= reference * (1.0 + 2e-5),
            "steps={steps}: chord {l} > arc {reference}"
        );
        assert!(
            l + 1e-6 >= prev,
            "steps={steps}: {l} < previous {prev} (not monotone)"
        );
        prev = l;
    }
    let l256 = s.arc_length(256) as f64;
    assert!(
        ((l256 - reference) / reference).abs() < 1e-3,
        "arc_length(256) = {l256}, Simpson reference {reference}"
    );
}

#[test]
fn point2_distance_fast_sqrt_stays_within_lomont_two_step_bound() {
    // oracle: f64 hypot; Lomont (2003): magic 0x5f3759df + 2 Newton–Raphson
    // steps ⇒ max relative error 4.7e-6 (one step: 1.75e-3)
    let mut worst = 0.0f64;
    let mut worst_at = (0.0f32, 0.0f32);
    let mut mag = 1e-3f32;
    while mag < 1e3 {
        for k in 0..64 {
            let ang = k as f32 * core::f32::consts::TAU / 64.0;
            let dx = mag * ang.cos();
            let dy = mag * ang.sin();
            let got = Point2::new(dx, dy).distance(Point2::ZERO) as f64;
            let exact = (dx as f64).hypot(dy as f64);
            let rel = ((got - exact) / exact).abs();
            if rel > worst {
                worst = rel;
                worst_at = (dx, dy);
            }
        }
        mag *= 1.07;
    }
    assert!(
        worst <= 4.7e-6,
        "fast sqrt worst relative error {worst:e} at {worst_at:?} (Lomont 2-step bound 4.7e-6)"
    );
    // `length()` and `distance()` share the law
    assert!(((Point2::new(3.0, 4.0).length() - 5.0) as f64).abs() < 5.0 * 4.7e-6);
}

// ───────────────────────── pen model ──────────────────────────────────────

#[test]
fn pen_half_width_matches_broad_nib_closed_form_within_bhaskara_bound() {
    let params = MetaFontParams::sans_regular();
    let pen = PenModel::from_params(&params);
    let w = params.stroke_half_width() as f64;
    let c = params.contrast as f64;
    let phi = pen.pen_angle as f64;
    // Bhaskara I sine approximation: |sin_approx − sin| ≤ 1.63e-3 on [0, π]
    // ⇒ |Δsin²| ≤ 2·1.63e-3 (+ second order) ⇒ |Δr| ≤ w·c·3.3e-3
    let bound = w * c * 3.4e-3 + 1e-7;
    for i in 0..=720 {
        let theta = -core::f64::consts::TAU + i as f64 * core::f64::consts::TAU / 180.0;
        let got = pen.half_width_at_angle(theta as f32) as f64;
        let s = (theta - phi).sin();
        let expected = w * (1.0 - c * s * s); // oracle: broad-nib ellipse law
        assert!(
            (got - expected).abs() <= bound,
            "θ={theta}: half_width {got} vs {expected} (bound {bound})"
        );
    }
    // extremes sit on the nib axis: max at θ = φ, min at θ = φ ± π/2
    let at_phi = pen.half_width_at_angle(phi as f32) as f64;
    let at_perp = pen.half_width_at_angle((phi + core::f64::consts::FRAC_PI_2) as f32) as f64;
    assert!((at_phi - w).abs() <= bound);
    assert!((at_perp - w * (1.0 - c)).abs() <= bound);
}

#[test]
fn pen_half_width_is_direction_independent_at_zero_contrast_and_scale_invariant() {
    let pen = PenModel::from_params(&sans_mono_pen());
    let w = pen.base_width;
    for i in 0..360 {
        let theta = i as f32 * core::f32::consts::TAU / 360.0;
        // c = 0 ⇒ r(θ) = w exactly (the law degenerates to a constant)
        assert_eq!(pen.half_width_at_angle(theta).to_bits(), w.to_bits());
        // the tangent form depends only on direction, not on |tangent|
        let t = Point2::new(theta.cos(), theta.sin());
        let a = PenModel::from_params(&MetaFontParams::sans_regular()).half_width(t);
        let b = PenModel::from_params(&MetaFontParams::sans_regular()).half_width(t.scale(37.5));
        assert!(
            (a - b).abs() < 1e-5,
            "θ={theta}: |t| scaling changed width {a} → {b}"
        );
    }
}

#[test]
fn slant_is_a_shear_by_tan_of_the_angle() {
    // oracle: x' = x + y·tan α, y' = y ; Bhaskara-derived tangent bound for
    // |α| ≤ 0.5: (|Δs| + |tan α|·|Δc|) / cos α ≤ 1.63e-3·(1 + 0.546)/0.8776 = 2.9e-3
    for i in -50..=50 {
        let alpha = i as f32 * 0.01;
        for (x, y) in [(0.0f32, 1.0f32), (0.4, 0.7), (-0.3, 0.25), (1.0, -1.0)] {
            let q = Point2::new(x, y).slant(alpha);
            let expected = x as f64 + y as f64 * (alpha as f64).tan();
            assert!(
                (q.x as f64 - expected).abs() <= 2.9e-3 * y.abs() as f64 + 1e-7,
                "slant({alpha}) of ({x},{y}): x' {} vs {expected}",
                q.x
            );
            assert_eq!(q.y.to_bits(), y.to_bits(), "shear must not touch y");
        }
    }
    // α = 0 is the identity exactly
    let p = Point2::new(0.123, 0.456);
    assert_eq!(p.slant(0.0), p);
}

// ───────────────────────── parameters ─────────────────────────────────────

#[test]
fn params_wire_roundtrip_is_bit_exact_and_lerp_hits_both_endpoints() {
    let presets = [
        MetaFontParams::sans_regular(),
        MetaFontParams::sans_hairline(),
        MetaFontParams::sans_bold(),
        MetaFontParams::serif_regular(),
        MetaFontParams::serif_italic(),
        MetaFontParams::mono_regular(),
        MetaFontParams::display_heavy(),
    ];
    for a in &presets {
        let back = MetaFontParams::decode(&a.encode());
        assert_eq!(&back, a, "encode/decode must be the identity");
        assert_eq!(MetaFontParams::try_decode(&a.encode()), Some(*a));
        for b in &presets {
            // oracle: lerp(0) = a, lerp(1) = b (endpoint exactness of the
            // affine blend), lerp(½) = arithmetic mean
            assert_eq!(a.lerp(b, 0.0), *a, "lerp(t=0) must return self");
            assert_eq!(a.lerp(b, 1.0), *b, "lerp(t=1) must return other");
            let mid = a.lerp(b, 0.5);
            assert!(((mid.weight - (a.weight + b.weight) * 0.5) as f64).abs() < 1e-7);
            assert!(((mid.slant - (a.slant + b.slant) * 0.5) as f64).abs() < 1e-7);
        }
    }
    // Point2 shares the lerp law
    let (p, q) = (Point2::new(0.45, -0.3), Point2::new(0.1, 0.7));
    assert_eq!(p.lerp(q, 0.0), p);
    assert_eq!(p.lerp(q, 1.0), q);
}

#[test]
fn derived_widths_follow_their_affine_laws() {
    // oracle: stroke_half_width = 0.01 + 0.08·weight, thick = ·(1 + 0.8c), thin = ·(1 − 0.5c)
    for i in 0..=20 {
        let mut p = MetaFontParams::sans_regular();
        p.weight = i as f32 / 20.0;
        p.contrast = (i % 5) as f32 / 4.0;
        let hw = 0.01 + 0.08 * p.weight as f64;
        assert!((p.stroke_half_width() as f64 - hw).abs() < 1e-7);
        assert!((p.thick_half_width() as f64 - hw * (1.0 + 0.8 * p.contrast as f64)).abs() < 1e-7);
        assert!((p.thin_half_width() as f64 - hw * (1.0 - 0.5 * p.contrast as f64)).abs() < 1e-7);
        assert!(p.thin_half_width() <= p.stroke_half_width());
        assert!(p.stroke_half_width() <= p.thick_half_width());
    }
}

// ───────────────────────── rasterised glyph SDF ───────────────────────────

/// Straight stroke, contrast 0: the exact SDF is the capsule
/// d(p) = dist(p, segment) − half_width at every pixel of the tile.
#[test]
fn rasterised_straight_stroke_sdf_is_the_capsule_closed_form() {
    let params = sans_mono_pen();
    let hw = params.stroke_half_width() as f64;
    let a = Point2::new(0.15, 0.1);
    let b = Point2::new(0.72, 0.64);
    let mut sk = GlyphSkeleton::empty();
    sk.add_stroke(Stroke::line(a, b));
    let sdf = GlyphGenerator::new(&params).generate_from_skeleton(&sk);

    let n = alice_font::glyph::GLYPH_SDF_SIZE;
    let mut worst = 0.0f64;
    let mut worst_px = (0, 0);
    for py in 0..n {
        for px in 0..n {
            let p = pixel_world(&sdf, px, py);
            let expected =
                segment_distance(p, (a.x as f64, a.y as f64), (b.x as f64, b.y as f64)) - hw;
            let got = sdf.data[py * n + px] as f64;
            let err = (got - expected).abs();
            if err > worst {
                worst = err;
                worst_px = (px, py);
            }
            // sign contract: negative ⇔ inside the capsule
            if expected < -1e-4 {
                assert!(
                    got < 0.0,
                    "pixel {px},{py}: inside ({expected}) but sdf {got} ≥ 0"
                );
            }
            if expected > 1e-4 {
                assert!(
                    got > 0.0,
                    "pixel {px},{py}: outside ({expected}) but sdf {got} ≤ 0"
                );
            }
        }
    }
    // fast sqrt (4.7e-6 rel) + f32 pixel placement are the only legitimate
    // error sources; anything above 1e-4 em is a law error
    assert!(
        worst < 1e-4,
        "worst |sdf − capsule| = {worst} em at pixel {worst_px:?} (half-width {hw})"
    );
}

/// The distance field of a union of strokes is the pointwise minimum of the
/// member capsules.
#[test]
fn rasterised_two_stroke_sdf_is_the_min_of_the_capsules() {
    let params = sans_mono_pen();
    let hw = params.stroke_half_width() as f64;
    let (a, b) = (Point2::new(0.1, 0.1), Point2::new(0.8, 0.8));
    let (c, d) = (Point2::new(0.1, 0.8), Point2::new(0.8, 0.1));
    let mut sk = GlyphSkeleton::empty();
    sk.add_stroke(Stroke::line(a, b));
    sk.add_stroke(Stroke::line(c, d));
    let sdf = GlyphGenerator::new(&params).generate_from_skeleton(&sk);
    let n = alice_font::glyph::GLYPH_SDF_SIZE;
    let mut worst = 0.0f64;
    for py in 0..n {
        for px in 0..n {
            let p = pixel_world(&sdf, px, py);
            let d1 = segment_distance(p, (a.x as f64, a.y as f64), (b.x as f64, b.y as f64));
            let d2 = segment_distance(p, (c.x as f64, c.y as f64), (d.x as f64, d.y as f64));
            let expected = d1.min(d2) - hw;
            worst = worst.max((sdf.data[py * n + px] as f64 - expected).abs());
        }
    }
    assert!(worst < 1e-4, "worst |sdf − min(capsules)| = {worst} em");
}

/// SDF invariant: the field is 1-Lipschitz for a direction-independent pen
/// (exact capsule chain), and at most (1 + max |Δhw| / |segment|)-Lipschitz
/// for a contrast pen (the interpolated-radius law adds that slope);
/// translating the skeleton translates the field.
#[test]
fn rasterised_sdf_is_one_lipschitz_and_translation_invariant() {
    let n = alice_font::glyph::GLYPH_SDF_SIZE;
    let curve = Stroke::new(
        Point2::new(0.1, 0.2),
        Point2::new(0.3, 0.9),
        Point2::new(0.7, -0.1),
        Point2::new(0.9, 0.6),
    );
    for params in [sans_mono_pen(), MetaFontParams::sans_regular()] {
        let gen = GlyphGenerator::new(&params);
        let pen = PenModel::from_params(&params);
        // law-derived Lipschitz constant: 1 + max over the 16 chords of
        // |hw(t_{i+1}) − hw(t_i)| / chord length (0 for contrast 0)
        let mut slope = 0.0f64;
        for i in 0..16 {
            let (t0, t1) = (i as f32 / 16.0, (i + 1) as f32 / 16.0);
            let (p0, p1) = (curve.position(t0), curve.position(t1));
            let dh = (pen.half_width(curve.tangent(t1)) - pen.half_width(curve.tangent(t0))) as f64;
            slope = slope.max(dh.abs() / p0.distance(p1) as f64);
        }
        if params.contrast == 0.0 {
            assert_eq!(slope, 0.0);
        }
        let lip = 1.0 + slope;
        let mut sk = GlyphSkeleton::empty();
        sk.add_stroke(curve);
        let sdf = gen.generate_from_skeleton(&sk);
        let dx = (sdf.bbox_max.x - sdf.bbox_min.x) as f64 / (n - 1) as f64;
        let dy = (sdf.bbox_max.y - sdf.bbox_min.y) as f64 / (n - 1) as f64;
        for py in 0..n {
            for px in 0..n {
                let v = sdf.data[py * n + px] as f64;
                if px + 1 < n {
                    let r = sdf.data[py * n + px + 1] as f64;
                    assert!(
                        (r - v).abs() <= dx * lip * (1.0 + 1e-4) + 1e-6,
                        "contrast {}: x-neighbour {px},{py}: {v}→{r} over {dx} (L={lip})",
                        params.contrast
                    );
                }
                if py + 1 < n {
                    let u = sdf.data[(py + 1) * n + px] as f64;
                    assert!(
                        (u - v).abs() <= dy * lip * (1.0 + 1e-4) + 1e-6,
                        "contrast {}: y-neighbour {px},{py}: {v}→{u} over {dy} (L={lip})",
                        params.contrast
                    );
                }
            }
        }
        // translation: same field, shifted bbox
        let mut sk2 = GlyphSkeleton::empty();
        sk2.add_stroke(curve.translate(0.37, -0.21));
        let sdf2 = gen.generate_from_skeleton(&sk2);
        assert!(((sdf2.bbox_min.x - sdf.bbox_min.x) as f64 - 0.37).abs() < 1e-6);
        assert!(((sdf2.bbox_min.y - sdf.bbox_min.y) as f64 + 0.21).abs() < 1e-6);
        let mut worst = 0.0f32;
        for i in 0..n * n {
            worst = worst.max((sdf2.data[i] - sdf.data[i]).abs());
        }
        assert!(
            worst < 1e-5,
            "translation changed the field by up to {worst}"
        );
    }
}

// ───────────────────────── crate boundary: Font → alice-sdf ───────────────

#[cfg(feature = "sdf")]
mod sdf_bridge_contract {
    use super::*;
    use alice_font::sdf_bridge::{glyph_skeleton_to_sdf2d, stroke_to_sdf2d};
    use alice_sdf::sdf2d::eval_2d;

    fn grid() -> Vec<(f64, f64)> {
        let mut v = Vec::new();
        for j in 0..=40 {
            for i in 0..=40 {
                v.push((-0.2 + i as f64 * 0.035, -0.2 + j as f64 * 0.035));
            }
        }
        v
    }

    /// A straight stroke handed to alice-sdf must evaluate to the capsule
    /// closed form (segment distance − thickness) — the same truth the Font
    /// rasteriser is held to above.
    #[test]
    #[ignore = "alice-sdf ≤ 3.1.0 の Sdf2dNode::Bezier は 17 点 sampling で線分間の距離が最大 L/32 ずれる、SDF 側修正 + Font の dep bump 後に ignore を外す"]
    fn stroke_to_sdf2d_line_evaluates_to_the_capsule_closed_form() {
        let (a, b) = (Point2::new(0.05, 0.1), Point2::new(0.9, 0.7));
        let thickness = 0.046f32;
        let node = stroke_to_sdf2d(&Stroke::line(a, b), thickness);
        let mut worst = 0.0f64;
        for p in grid() {
            let got = eval_2d(&node, [p.0 as f32, p.1 as f32]) as f64;
            let expected = segment_distance(p, (a.x as f64, a.y as f64), (b.x as f64, b.y as f64))
                - thickness as f64;
            worst = worst.max((got - expected).abs());
        }
        assert!(worst < 1e-4, "worst |eval_2d − capsule| = {worst}");
    }

    /// Upper bound of the *current* alice-sdf approximation (17 uniform samples
    /// ⇒ error ≤ half the sample spacing = L/32).  This is a regression guard
    /// on the known approximation, not a correctness oracle — the oracle is
    /// the ignored test above.
    #[test]
    fn stroke_to_sdf2d_line_error_never_exceeds_the_sampling_bound() {
        let (a, b) = (Point2::new(0.05, 0.1), Point2::new(0.9, 0.7));
        let len = ((b.x - a.x) as f64).hypot((b.y - a.y) as f64);
        let thickness = 0.046f32;
        let node = stroke_to_sdf2d(&Stroke::line(a, b), thickness);
        for p in grid() {
            let got = eval_2d(&node, [p.0 as f32, p.1 as f32]) as f64;
            let expected = segment_distance(p, (a.x as f64, a.y as f64), (b.x as f64, b.y as f64))
                - thickness as f64;
            // point-set distance over-estimates, never under-estimates
            assert!(got + 1e-6 >= expected, "{p:?}: {got} < capsule {expected}");
            assert!(
                got - expected <= len / 32.0 + 1e-6,
                "{p:?}: {got} − {expected} > L/32"
            );
        }
    }

    /// Bridge composition contract: the skeleton union is the pointwise min
    /// of its strokes (exact — Union is `min` in alice-sdf), and the empty
    /// skeleton is everywhere outside.
    #[test]
    fn glyph_skeleton_to_sdf2d_is_the_pointwise_min_of_its_strokes() {
        let s1 = Stroke::line(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0));
        let s2 = Stroke::new(
            Point2::new(0.0, 1.0),
            Point2::new(0.3, 0.2),
            Point2::new(0.7, 0.8),
            Point2::new(1.0, 0.0),
        );
        let mut sk = GlyphSkeleton::empty();
        sk.add_stroke(s1);
        sk.add_stroke(s2);
        let union = glyph_skeleton_to_sdf2d(&sk, 0.05);
        let (n1, n2) = (stroke_to_sdf2d(&s1, 0.05), stroke_to_sdf2d(&s2, 0.05));
        for p in grid() {
            let q = [p.0 as f32, p.1 as f32];
            let expected = eval_2d(&n1, q).min(eval_2d(&n2, q));
            assert_eq!(eval_2d(&union, q).to_bits(), expected.to_bits(), "{p:?}");
        }
        let empty = glyph_skeleton_to_sdf2d(&GlyphSkeleton::empty(), 0.05);
        for p in grid() {
            assert!(eval_2d(&empty, [p.0 as f32, p.1 as f32]) >= 0.0);
        }
    }
}
