//! Ridge extraction from distance transform (local maxima)。
//!
//! Distance transform の各 inside pixel について、3×3 neighborhood 内で
//! strict local maximum (≥ 全 8 neighbors) の pixel を "ridge" として抽出。
//!
//! Ridge = 書体 stroke の centerline に相当。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Distance transform → ridge bitmap (local maxima at each pixel)。
///
/// # Returns
/// - `Vec<bool>` length = `resolution² `、ridge pixels = true、他 = false
#[must_use]
pub fn extract_ridge(dist: &[f32], resolution: usize) -> Vec<bool> {
    let n = resolution * resolution;
    debug_assert_eq!(dist.len(), n);

    let mut ridge = vec![false; n];
    // Minimum distance threshold for ridge (skip very small distances = boundary noise)
    let min_dist_threshold = 1.5 / resolution as f32; // 1.5 pixel

    for y in 1..resolution.saturating_sub(1) {
        for x in 1..resolution.saturating_sub(1) {
            let idx = y * resolution + x;
            let d = dist[idx];
            if d < min_dist_threshold {
                continue;
            }
            // Check 8 neighbors
            let mut is_local_max = true;
            let mut equal_count = 0; // Allow plateau (relax strict max)
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    let nd = dist[ny * resolution + nx];
                    if nd > d {
                        is_local_max = false;
                        break;
                    }
                    if (nd - d).abs() < 1e-6 {
                        equal_count += 1;
                    }
                }
                if !is_local_max {
                    break;
                }
            }
            // Accept as ridge if strict local max OR plateau with limited equal neighbors
            // (plateau allows straight thin strokes to keep continuous ridge)
            if is_local_max && equal_count <= 5 {
                ridge[idx] = true;
            }
        }
    }

    ridge
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::medial_axis::dist_transform::chamfer_distance;
    use crate::medial_axis::rasterize::rasterize_polygon;

    #[test]
    fn ridge_of_horizontal_rectangle_along_centerline() {
        // Horizontal rectangle (y in 0.45..0.55)
        let rect = vec![vec![[0.1, 0.45], [0.9, 0.45], [0.9, 0.55], [0.1, 0.55]]];
        let bitmap = rasterize_polygon(&rect, 64);
        let dist = chamfer_distance(&bitmap, 64);
        let ridge = extract_ridge(&dist, 64);
        // Ridge should exist near y=0.5 (row 32)
        let ridge_at_center_row: usize = (0..64).filter(|&x| ridge[32 * 64 + x]).count();
        assert!(
            ridge_at_center_row > 3,
            "should have several ridge pixels along centerline"
        );
    }
}
