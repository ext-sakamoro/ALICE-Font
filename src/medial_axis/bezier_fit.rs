//! Cubic Bezier fit for medial chain points。
//!
//! Simple 1/3, 2/3 sample fit: p0 = chain start、p3 = chain end、
//! p1 = chain[1/3]、p2 = chain[2/3]。Pen widths = chain start / end widths。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;

use super::MedialChain;
use crate::stroke::{Point2, Stroke};

#[cfg(test)]
use alloc::vec;

/// Chain → cubic Bezier `Stroke` + `(width_start, width_end)`。
///
/// # Returns
/// `(Stroke, w_start, w_end)`:
/// - `Stroke` = fitted cubic Bezier (p0, p1, p2, p3)
/// - `w_start` = pen radius at chain start (em unit)
/// - `w_end` = pen radius at chain end (em unit)
///
/// Chain point 数が < 4 の場合、control points が endpoints に collapse
/// (degenerate line 相当)。
#[must_use]
pub fn fit_bezier(chain: &MedialChain) -> (Stroke, f32, f32) {
    let n = chain.points.len();
    if n < 2 {
        let p = if n == 1 {
            Point2::new(chain.points[0][0], chain.points[0][1])
        } else {
            Point2::ZERO
        };
        return (Stroke::new(p, p, p, p), 0.05, 0.05);
    }

    let start = Point2::new(chain.points[0][0], chain.points[0][1]);
    let end = Point2::new(chain.points[n - 1][0], chain.points[n - 1][1]);

    // Control points at 1/3, 2/3 samples for cubic Bezier fit
    let idx_1_3 = n / 3;
    let idx_2_3 = (2 * n) / 3;
    let cp1_sample = chain.points[idx_1_3];
    let cp2_sample = chain.points[idx_2_3];

    // For pure cubic Bezier passing through 4 points, control points are:
    // B(1/3) = 8/27 * p0 + 12/27 * p1 + 6/27 * p2 + 1/27 * p3
    // B(2/3) = 1/27 * p0 + 6/27 * p1 + 12/27 * p2 + 8/27 * p3
    // Solving for p1, p2 given B(1/3), B(2/3), p0, p3:
    // Simplified via matrix inversion:
    let p0x = chain.points[0][0];
    let p0y = chain.points[0][1];
    let p3x = chain.points[n - 1][0];
    let p3y = chain.points[n - 1][1];
    let b1x = cp1_sample[0];
    let b1y = cp1_sample[1];
    let b2x = cp2_sample[0];
    let b2y = cp2_sample[1];

    // Inversion of 2×2 system:
    // 12/27 * p1x + 6/27 * p2x = b1x - 8/27 * p0x - 1/27 * p3x
    //  6/27 * p1x + 12/27 * p2x = b2x - 1/27 * p0x - 8/27 * p3x
    let r1x = b1x - (8.0 / 27.0) * p0x - (1.0 / 27.0) * p3x;
    let r2x = b2x - (1.0 / 27.0) * p0x - (8.0 / 27.0) * p3x;
    let r1y = b1y - (8.0 / 27.0) * p0y - (1.0 / 27.0) * p3y;
    let r2y = b2y - (1.0 / 27.0) * p0y - (8.0 / 27.0) * p3y;
    // det = (12 × 12 - 6 × 6) / 27² = 108 / 729
    // Inverse: [12 -6; -6 12] / 108 (scale factor from denominators)
    // Simplified: p1 = (12 × r1 - 6 × r2) / (108/27 = 4)... simpler formula:
    // p1 = (2 * r1 - r2) * 3 / 2  (via elimination)
    // p2 = (-r1 + 2 * r2) * 3 / 2
    let p1x = (2.0 * r1x - r2x) * 27.0 / 18.0;
    let p1y = (2.0 * r1y - r2y) * 27.0 / 18.0;
    let p2x = (-r1x + 2.0 * r2x) * 27.0 / 18.0;
    let p2y = (-r1y + 2.0 * r2y) * 27.0 / 18.0;

    let p1 = Point2::new(p1x, p1y);
    let p2 = Point2::new(p2x, p2y);

    let w_start = chain.widths[0];
    let w_end = chain.widths[n - 1];

    (Stroke::new(start, p1, p2, end), w_start, w_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_horizontal_straight_chain_produces_horizontal_bezier() {
        let chain = MedialChain {
            points: vec![[0.1, 0.5], [0.5, 0.5], [0.9, 0.5]],
            widths: vec![0.03, 0.05, 0.03],
        };
        let (stroke, w_start, w_end) = fit_bezier(&chain);
        assert!((stroke.p0.x - 0.1).abs() < 1e-4);
        assert!((stroke.p3.x - 0.9).abs() < 1e-4);
        assert!((stroke.p0.y - 0.5).abs() < 1e-4);
        assert!((stroke.p3.y - 0.5).abs() < 1e-4);
        assert!((w_start - 0.03).abs() < 1e-6);
        assert!((w_end - 0.03).abs() < 1e-6);
    }

    #[test]
    fn fit_empty_chain_produces_zero_stroke() {
        let chain = MedialChain {
            points: vec![],
            widths: vec![],
        };
        let (stroke, _, _) = fit_bezier(&chain);
        assert!((stroke.p0.x).abs() < 1e-6);
    }
}
