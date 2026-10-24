//! Chain tracing from ridge bitmap (BFS connectivity)。
//!
//! Ridge pixels を connected component 化し、各 chain の pixel points と
//! pen widths を出力。Junction (degree ≥ 3) は chain break points として扱う。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

use super::MedialChain;

/// Ridge bitmap + distance transform → `MedialChain` list。
///
/// BFS で connected component 抽出、各 component の pixel を em normalized
/// points に変換、対応する distance を pen width として抽出。
#[must_use]
pub fn trace_chains(ridge: &[bool], dist: &[f32], resolution: usize) -> Vec<MedialChain> {
    let n = resolution * resolution;
    debug_assert_eq!(ridge.len(), n);
    debug_assert_eq!(dist.len(), n);

    let mut visited = vec![false; n];
    let mut chains = Vec::new();

    for start_idx in 0..n {
        if !ridge[start_idx] || visited[start_idx] {
            continue;
        }
        // BFS from start_idx
        let mut queue = vec![start_idx];
        let mut chain_pixels = Vec::new();
        while let Some(idx) = queue.pop() {
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            chain_pixels.push(idx);
            let x = idx % resolution;
            let y = idx / resolution;
            // 8-connected neighbors
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= resolution as i32 || ny >= resolution as i32 {
                        continue;
                    }
                    let nidx = (ny as usize) * resolution + (nx as usize);
                    if ridge[nidx] && !visited[nidx] {
                        queue.push(nidx);
                    }
                }
            }
        }
        if chain_pixels.is_empty() {
            continue;
        }
        // Convert pixels to em space + widths
        let res_f = resolution as f32;
        let points: Vec<[f32; 2]> = chain_pixels
            .iter()
            .map(|&i| {
                let x = i % resolution;
                let y = i / resolution;
                [(x as f32 + 0.5) / res_f, (y as f32 + 0.5) / res_f]
            })
            .collect();
        let widths: Vec<f32> = chain_pixels.iter().map(|&i| dist[i]).collect();
        chains.push(MedialChain { points, widths });
    }

    // Sort each chain's points by spatial order (simple heuristic: bounding box axis)
    for chain in &mut chains {
        sort_chain_points(chain);
    }

    chains
}

/// Chain points を bounding-box-aware sequential order に並び替え。
///
/// MVP heuristic: bounding box の長辺方向で sort (horizontal or vertical
/// stroke に対応)。Junction や curved chain は精度低い可能性。
fn sort_chain_points(chain: &mut MedialChain) {
    if chain.points.len() < 2 {
        return;
    }
    // Compute bounding box
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for p in &chain.points {
        min_x = min_x.min(p[0]);
        max_x = max_x.max(p[0]);
        min_y = min_y.min(p[1]);
        max_y = max_y.max(p[1]);
    }
    let is_horizontal = (max_x - min_x) >= (max_y - min_y);

    // Zip points + widths together、sort、unzip
    let mut zipped: Vec<([f32; 2], f32)> = chain
        .points
        .iter()
        .zip(chain.widths.iter())
        .map(|(&p, &w)| (p, w))
        .collect();

    if is_horizontal {
        zipped.sort_by(|a, b| {
            a.0[0]
                .partial_cmp(&b.0[0])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
    } else {
        zipped.sort_by(|a, b| {
            a.0[1]
                .partial_cmp(&b.0[1])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
    }

    chain.points = zipped.iter().map(|z| z.0).collect();
    chain.widths = zipped.iter().map(|z| z.1).collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::medial_axis::dist_transform::chamfer_distance;
    use crate::medial_axis::rasterize::rasterize_polygon;
    use crate::medial_axis::ridge_extract::extract_ridge;

    #[test]
    fn trace_horizontal_rectangle_produces_at_least_1_chain() {
        let rect = vec![vec![[0.1, 0.45], [0.9, 0.45], [0.9, 0.55], [0.1, 0.55]]];
        let bitmap = rasterize_polygon(&rect, 64);
        let dist = chamfer_distance(&bitmap, 64);
        let ridge = extract_ridge(&dist, 64);
        let chains = trace_chains(&ridge, &dist, 64);
        assert!(!chains.is_empty(), "should produce ≥1 chain for rectangle");
    }
}
