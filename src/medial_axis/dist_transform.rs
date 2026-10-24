//! Chamfer distance transform for inside pixels of a bitmap。
//!
//! 各 inside pixel から nearest outside (boundary) pixel までの距離を計算。
//! Chamfer 5-7-11 approximation を使用 (Euclidean 距離の近似、fast)。
//!
//! Output = distance array of `f32` (em unit)、outside pixels = 0.0。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

/// Chamfer 5-7-11 weight (approximation of Euclidean distance)。
///
/// - Direct neighbor (edge): weight 5
/// - Diagonal neighbor: weight 7
/// - Knight-move (2,1) neighbor: weight 11
///
/// After 2 forward + backward passes、divide by 5.0 で em-unit conversion。
const CHAMFER_EDGE: u32 = 5;
const CHAMFER_DIAG: u32 = 7;
// CHAMFER_KNIGHT: u32 = 11 (5-7-11 の 3rd weight、MVP では edge+diag のみ使用)

/// Bitmap → distance transform (inside distance to nearest boundary)。
///
/// # Returns
/// - `Vec<f32>` length = `resolution² `、outside pixels = 0.0、
///   inside pixels = distance in em unit (0..~0.5)
#[must_use]
pub fn chamfer_distance(bitmap: &[bool], resolution: usize) -> Vec<f32> {
    let n = resolution * resolution;
    debug_assert_eq!(bitmap.len(), n);

    // Initialize: outside = 0, inside = MAX
    let mut dist_u32 = vec![u32::MAX; n];
    for (i, &b) in bitmap.iter().enumerate() {
        if !b {
            dist_u32[i] = 0;
        }
    }

    // Forward pass (top-left → bottom-right)
    for y in 0..resolution {
        for x in 0..resolution {
            let idx = y * resolution + x;
            if !bitmap[idx] {
                continue; // outside stays 0
            }
            let mut d = dist_u32[idx];
            // Check 5-7-11 neighbors already visited (top-left quadrant)
            d = min_with_offset(d, &dist_u32, x, y, -1, -1, resolution, CHAMFER_DIAG);
            d = min_with_offset(d, &dist_u32, x, y, 0, -1, resolution, CHAMFER_EDGE);
            d = min_with_offset(d, &dist_u32, x, y, 1, -1, resolution, CHAMFER_DIAG);
            d = min_with_offset(d, &dist_u32, x, y, -1, 0, resolution, CHAMFER_EDGE);
            // Knight moves (skip for MVP、edge+diag sufficient)
            dist_u32[idx] = d;
        }
    }

    // Backward pass (bottom-right → top-left)
    for y in (0..resolution).rev() {
        for x in (0..resolution).rev() {
            let idx = y * resolution + x;
            if !bitmap[idx] {
                continue;
            }
            let mut d = dist_u32[idx];
            d = min_with_offset(d, &dist_u32, x, y, 1, 1, resolution, CHAMFER_DIAG);
            d = min_with_offset(d, &dist_u32, x, y, 0, 1, resolution, CHAMFER_EDGE);
            d = min_with_offset(d, &dist_u32, x, y, -1, 1, resolution, CHAMFER_DIAG);
            d = min_with_offset(d, &dist_u32, x, y, 1, 0, resolution, CHAMFER_EDGE);
            dist_u32[idx] = d;
        }
    }

    // Convert u32 (chamfer units) → f32 (em unit)
    // Chamfer edge = 5 = 1 pixel = 1/resolution em
    let scale = 1.0f32 / (5.0 * resolution as f32);
    dist_u32.iter().map(|&d| d as f32 * scale).collect()
}

/// Neighbor pixel の distance + weight を候補として最小値取る。
#[allow(clippy::too_many_arguments)] // Internal helper、5 bbox coords + weight = 意図的
fn min_with_offset(
    current: u32,
    dist: &[u32],
    x: usize,
    y: usize,
    dx: i32,
    dy: i32,
    res: usize,
    weight: u32,
) -> u32 {
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= res as i32 || ny >= res as i32 {
        return current;
    }
    let nidx = (ny as usize) * res + (nx as usize);
    let neighbor_dist = dist[nidx];
    if neighbor_dist == u32::MAX {
        return current;
    }
    let candidate = neighbor_dist.saturating_add(weight);
    current.min(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::medial_axis::rasterize::rasterize_polygon;

    #[test]
    fn distance_transform_outside_is_zero() {
        let bitmap = vec![false; 64]; // 8×8 all outside
        let dist = chamfer_distance(&bitmap, 8);
        assert!(dist.iter().all(|&d| d == 0.0));
    }

    #[test]
    fn distance_transform_center_of_square_has_max_distance() {
        let square = vec![vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]]];
        let bitmap = rasterize_polygon(&square, 32);
        let dist = chamfer_distance(&bitmap, 32);
        // Center pixel index
        let center = dist[16 * 32 + 16];
        // Edge pixel inside
        let edge_inside = dist[16 * 32 + 8]; // still inside (x=8, y=16 = em (0.25, 0.5), inside square 0.2-0.8)
        assert!(center > edge_inside, "center should have max distance");
        assert!(center > 0.1, "center distance should be substantial");
    }
}
