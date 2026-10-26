//! Law minimum / maximum / clamp for the library's float comparisons (ALICE-Law G2)
//!
//! The sign of an equal pair of zeros is unspecified for `f32::min` / `f32::max` and differs between
//! targets (`min(+0, -0)` is `-0` on one and `+0` on another), so the library does not call them
//! (`.github/clippy-lib/clippy.toml` forbids them); it routes every float minimum and maximum through
//! alice-det-math, which fixes the order `-0 < +0` and gives the canonical quiet NaN for a both-NaN
//! pair (ALICE-Law G4). These delegate to it, so there is no copy to drift.
//!
//! # Clamp and the clamp generation
//!
//! alice-det-math's main does not yet expose a `clamp`; its clamp generation is unlanded. Until it
//! lands, [`law_clamp_f32`] / [`law_clamp_f64`] compose `min(max(x, lo), hi)` from the det-math
//! minimum and maximum, which fixes the sign of a zero at the bounds the same way. The sites that use
//! them will move again when det-math's native clamp replaces the composition (it may round the
//! degenerate `lo > hi` and NaN cases differently); the ALICE-Law conformance note records which.
//!
//! # NaN
//!
//! At the helper level `law_clamp(NaN, lo, hi)` returns `lo`: the det-math minimum and maximum take
//! the number over a NaN, so `max(NaN, lo) = lo` then `min(lo, hi) = lo`. That differs from
//! `f32::clamp`'s NaN → NaN, but no call site can feed a NaN that survives the clamp: the outline
//! rasteriser's `segments()` drops zero-length edges (so `len2 > 0` in `dist2`), the glyph generator
//! maps a degenerate stroke length to `inv_len2 = 0` (so the product is `0`, not `0/0`), and a NaN
//! coordinate makes the distance NaN through the other terms regardless of the clamp.
//!
//! Tests keep the platform `f32::min` / `f32::max` for their independent oracles, so these helpers
//! and the ban live on the library only.

use alice_det_math::ops;

/// Law maximum of two `f32` (alice-det-math's `max`): the platform value except the sign of an equal
/// pair of zeros, which is fixed to the order `-0 < +0`; a both-NaN pair gives the canonical NaN.
#[inline(always)]
pub(crate) fn law_max_f32(a: f32, b: f32) -> f32 {
    ops::max(a, b)
}

/// Law minimum of two `f32` ([`law_max_f32`]'s rule; alice-det-math's `min`).
#[inline(always)]
pub(crate) fn law_min_f32(a: f32, b: f32) -> f32 {
    ops::min(a, b)
}

/// Law maximum of two `f64` (alice-det-math's `max64`).
#[inline(always)]
pub(crate) fn law_max_f64(a: f64, b: f64) -> f64 {
    ops::max64(a, b)
}

/// Law minimum of two `f64` (alice-det-math's `min64`).
#[inline(always)]
pub(crate) fn law_min_f64(a: f64, b: f64) -> f64 {
    ops::min64(a, b)
}

/// Law clamp of an `f32` to `[lo, hi]`, composed from the det-math minimum and maximum (see the
/// [module docs](self) on the clamp generation). For `lo <= hi` it is `min(max(x, lo), hi)`.
#[inline(always)]
pub(crate) fn law_clamp_f32(x: f32, lo: f32, hi: f32) -> f32 {
    law_min_f32(law_max_f32(x, lo), hi)
}

/// Law clamp of an `f64` to `[lo, hi]` ([`law_clamp_f32`] for `f64`).
#[inline(always)]
pub(crate) fn law_clamp_f64(x: f64, lo: f64, hi: f64) -> f64 {
    law_min_f64(law_max_f64(x, lo), hi)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Per site class (ALICE-Law G2): the det-math minimum / maximum fix the sign of an equal pair of
    // zeros to the order -0 < +0, which the platform f32::min / f32::max leave unspecified. These are
    // the ties the routing exists for; a build that reverted to the platform functions would differ
    // here on some target. The NaN cases are pinned too (the helper's behaviour, which no call site
    // reaches with a surviving NaN -- see the module docs); the G6 goldens are unchanged by the
    // routing because the recorded scenarios feed no ±0 pair.

    #[test]
    fn law_min_max_order_the_sign_of_a_zero_pair() {
        assert_eq!(law_min_f32(0.0, -0.0).to_bits(), (-0.0f32).to_bits());
        assert_eq!(law_min_f32(-0.0, 0.0).to_bits(), (-0.0f32).to_bits());
        assert_eq!(law_max_f32(0.0, -0.0).to_bits(), (0.0f32).to_bits());
        assert_eq!(law_max_f32(-0.0, 0.0).to_bits(), (0.0f32).to_bits());
        assert_eq!(law_min_f64(0.0, -0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(law_max_f64(-0.0, 0.0).to_bits(), (0.0f64).to_bits());
    }

    #[test]
    fn law_clamp_orders_the_sign_of_a_zero_at_a_bound() {
        // clamp(-0, +0, 1) = min(max(-0, +0), 1) = min(+0, 1) = +0
        assert_eq!(law_clamp_f32(-0.0, 0.0, 1.0).to_bits(), (0.0f32).to_bits());
        // clamp(+0, -1, -0) = min(max(+0, -1), -0) = min(+0, -0) = -0
        assert_eq!(
            law_clamp_f32(0.0, -1.0, -0.0).to_bits(),
            (-0.0f32).to_bits()
        );
        assert_eq!(law_clamp_f64(-0.0, 0.0, 1.0).to_bits(), (0.0f64).to_bits());
    }

    #[test]
    fn law_min_max_nan_policy() {
        // a NaN with a number gives the number
        assert_eq!(law_min_f32(f32::NAN, 1.0), 1.0);
        assert_eq!(law_max_f32(2.0, f32::NAN), 2.0);
        assert_eq!(law_min_f64(f64::NAN, 3.0), 3.0);
        assert_eq!(law_max_f64(4.0, f64::NAN), 4.0);
        // a both-NaN pair is a NaN (its exact bits are det-math's contract, checked below)
        assert!(law_min_f32(f32::NAN, f32::NAN).is_nan());
        assert!(law_max_f64(f64::NAN, f64::NAN).is_nan());
        // the helper-level clamp of a NaN returns lo (max(NaN, lo) = lo, then min(lo, hi) = lo),
        // which differs from f32::clamp's NaN -> NaN; no call site reaches this (see the module docs)
        assert_eq!(law_clamp_f32(f32::NAN, 0.0, 1.0), 0.0);
        assert_eq!(law_clamp_f64(f64::NAN, 0.0, 1.0), 0.0);
    }

    #[test]
    fn a_nonfinite_input_does_not_panic() {
        let _ = law_clamp_f32(f32::INFINITY, 0.0, 1.0);
        let _ = law_clamp_f32(f32::NEG_INFINITY, 0.0, 1.0);
        let _ = law_min_f64(f64::INFINITY, f64::NEG_INFINITY);
    }
}
