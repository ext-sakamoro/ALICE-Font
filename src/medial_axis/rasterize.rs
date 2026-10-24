//! Polygon rasterization to bitmap via winding number test。
//!
//! Input contour points in em space `[0, 1]²`、output `bitmap[resolution²]`。
//! Pixel = true if inside polygon (per winding number rule)、false otherwise。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Polygon contour list を bitmap にラスタライズ (winding number test)。
///
/// # Returns
/// - `bitmap: Vec<bool>`、length = `resolution² `、row-major (y * res + x)
/// - `bitmap[i] = true` if pixel is inside polygon
#[must_use]
pub fn rasterize_polygon(contours: &[Vec<[f32; 2]>], resolution: usize) -> Vec<bool> {
    let mut bitmap = vec![false; resolution * resolution];
    let res_f = resolution as f32;

    for y in 0..resolution {
        for x in 0..resolution {
            // Center of pixel in em space
            let px = (x as f32 + 0.5) / res_f;
            let py = (y as f32 + 0.5) / res_f;
            let wn = winding_number([px, py], contours);
            bitmap[y * resolution + x] = wn != 0;
        }
    }

    bitmap
}

/// Winding number of point `p` w.r.t. polygon `contours` (closed via wrap-around)。
///
/// Positive winding = inside (CCW enclosing)、negative = inside (CW enclosing)、
/// zero = outside。Non-zero fill rule: inside if `wn != 0`。
fn winding_number(p: [f32; 2], contours: &[Vec<[f32; 2]>]) -> i32 {
    let mut wn = 0i32;
    for contour in contours {
        if contour.len() < 2 {
            continue;
        }
        for i in 0..contour.len() {
            let a = contour[i];
            let b = contour[(i + 1) % contour.len()];
            wn += segment_winding(p, a, b);
        }
    }
    wn
}

/// Segment `a → b` の点 `p` に対する winding contribution。
fn segment_winding(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> i32 {
    // Check if segment crosses horizontal ray from p going +x
    if a[1] <= p[1] {
        if b[1] > p[1] {
            // Upward crossing candidate
            let cross = (b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1]);
            if cross > 0.0 {
                return 1;
            }
        }
    } else if b[1] <= p[1] {
        // Downward crossing candidate
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1]);
        if cross < 0.0 {
            return -1;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rasterize_square_produces_inside_pixels() {
        let square = vec![vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]]];
        let bitmap = rasterize_polygon(&square, 16);
        // Center pixel should be inside
        let center = bitmap[8 * 16 + 8];
        assert!(center, "center of square must be inside");
        // Corner pixel should be outside
        let corner = bitmap[0];
        assert!(!corner, "top-left corner must be outside");
    }

    #[test]
    fn rasterize_empty_polygon_all_outside() {
        let bitmap = rasterize_polygon(&[], 8);
        assert!(bitmap.iter().all(|&b| !b), "no polygon → all outside");
    }

    #[test]
    fn winding_number_inside_square_is_1() {
        let square = vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]];
        assert_eq!(winding_number([0.5, 0.5], &square), 1);
    }

    #[test]
    fn winding_number_outside_square_is_0() {
        let square = vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]];
        assert_eq!(winding_number([1.5, 0.5], &square), 0);
    }
}
