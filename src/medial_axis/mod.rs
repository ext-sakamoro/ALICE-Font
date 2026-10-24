//! Approach 6: Medial Axis Extraction from polygon outline (v0.7.3-alpha)。
//!
//! v0.6.7 outline path の polygon data から build-time で medial axis
//! (書体 stroke centerline) を自動抽出、v0.7.2 skeleton shader で
//! **書体品質 + zero-asset runtime** を両立する。
//!
//! # Algorithm (MVP、distance transform based)
//!
//! 1. **Rasterize**: outline contours を bitmap (128×128 default) に描画
//!    (winding number test で inside/outside 判定)
//! 2. **Distance transform**: bitmap の各 inside pixel から nearest boundary
//!    までの距離を Chamfer 5-7-11 approximation で計算
//! 3. **Ridge extraction**: distance が local maximum (3×3 neighborhood で
//!    strict max) の pixel を "ridge" として抽出
//! 4. **Chain tracing**: ridge pixel を BFS で connected chain 化、
//!    endpoints (degree=1) と junctions (degree≥3) を分離
//! 5. **Bezier fit**: 各 chain を cubic Bezier で近似 (p0=start、p3=end、
//!    p1/p2=1/3, 2/3 samples)、pen width = distance at chain midpoint
//!
//! # ALICE 原理準拠
//!
//! Outline data (BIZ UDPGothic) は **build-time のみ使用**、runtime は
//! 抽出済 skeleton のみ ship = 「法則相当」扱い (書体設計 = 数百時間の
//! 職人 authoring を数式抽出化)。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

#[cfg(test)]
use alloc::vec;

pub mod bezier_fit;
pub mod chain_trace;
pub mod dist_transform;
pub mod rasterize;
pub mod ridge_extract;

use crate::stroke::Stroke;

/// Medial axis MVP resolution (bitmap size = `MEDIAL_AXIS_RESOLUTION`²)。
///
/// v0.7.3-alpha MVP: 128×128 = 16,384 pixel、fast build 用途。
/// Higher resolution (256, 512) は quality improvement 用途。
pub const MEDIAL_AXIS_RESOLUTION: usize = 128;

/// Extracted skeleton chain (2D pixel path in em box)。
#[derive(Debug, Clone)]
pub struct MedialChain {
    /// Chain の pixel points (em normalized、`[0, 1]²`)。
    pub points: Vec<[f32; 2]>,
    /// Chain 沿いの pen width (em unit、distance transform 由来)。
    pub widths: Vec<f32>,
}

impl MedialChain {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.points.len()
    }
}

/// Polygon contour list → medial axis chains。
///
/// Input: `contours` = closed polygon list (em normalized `[0, 1]²`)、
///         各 contour は Vec of `[x, y]` points。
///
/// Output: extracted `MedialChain` list、各 chain は cubic Bezier に fit 可能。
///
/// # 制限 (MVP)
///
/// - Resolution: [`MEDIAL_AXIS_RESOLUTION`] × 同 (128×128)
/// - Chain length filter: min 3 points (単一 pixel noise 除外)
/// - Junction handling: simple split (branching kanji で精度低い可能性)
///
/// # 使用例
///
/// ```
/// use alice_font::medial_axis::extract_medial_axis;
///
/// // 3 = horizontal stroke rectangle outline
/// let contours = vec![vec![
///     [0.1, 0.45], [0.9, 0.45], [0.9, 0.55], [0.1, 0.55], [0.1, 0.45],
/// ]];
/// let chains = extract_medial_axis(&contours);
/// assert!(!chains.is_empty(), "should extract at least 1 centerline");
/// ```
#[must_use]
pub fn extract_medial_axis(contours: &[Vec<[f32; 2]>]) -> Vec<MedialChain> {
    // Step 1: Rasterize polygon → bitmap (inside=true, outside=false)
    let bitmap = rasterize::rasterize_polygon(contours, MEDIAL_AXIS_RESOLUTION);

    // Step 2: Chamfer distance transform
    let dist = dist_transform::chamfer_distance(&bitmap, MEDIAL_AXIS_RESOLUTION);

    // Step 3: Ridge extraction (local maxima)
    let ridge = ridge_extract::extract_ridge(&dist, MEDIAL_AXIS_RESOLUTION);

    // Step 4: Trace chains
    let chains = chain_trace::trace_chains(&ridge, &dist, MEDIAL_AXIS_RESOLUTION);

    // Filter: chains with len >= 3
    chains.into_iter().filter(|c| c.points.len() >= 3).collect()
}

/// `MedialChain` list → Bezier `Stroke` list (v0.7.2 skeleton shader 直結)。
///
/// 各 chain を cubic Bezier で fit、pen width は chain midpoint の distance 使用。
#[must_use]
pub fn chains_to_strokes(chains: &[MedialChain]) -> Vec<(Stroke, f32, f32)> {
    chains.iter().map(bezier_fit::fit_bezier).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_horizontal_rectangle_returns_centerline() {
        // 一 = horizontal thin rectangle
        let contours = vec![vec![
            [0.1, 0.45],
            [0.9, 0.45],
            [0.9, 0.55],
            [0.1, 0.55],
            [0.1, 0.45],
        ]];
        let chains = extract_medial_axis(&contours);
        assert!(!chains.is_empty(), "should extract at least 1 chain");
        // At least one chain should span roughly horizontally at y≈0.5
        let has_horizontal = chains.iter().any(|c| {
            let first = c.points.first();
            let last = c.points.last();
            match (first, last) {
                (Some(f), Some(l)) => (f[1] - 0.5).abs() < 0.1 && (l[0] - f[0]).abs() > 0.3,
                _ => false,
            }
        });
        assert!(has_horizontal, "should have a roughly horizontal chain");
    }

    #[test]
    fn extract_empty_polygon_returns_empty_chains() {
        let contours: Vec<Vec<[f32; 2]>> = vec![];
        let chains = extract_medial_axis(&contours);
        assert!(chains.is_empty());
    }

    #[test]
    fn chains_to_strokes_produces_bezier_for_each_chain() {
        let contours = vec![vec![
            [0.1, 0.45],
            [0.9, 0.45],
            [0.9, 0.55],
            [0.1, 0.55],
            [0.1, 0.45],
        ]];
        let chains = extract_medial_axis(&contours);
        let strokes = chains_to_strokes(&chains);
        assert_eq!(strokes.len(), chains.len());
        for (s, w_start, w_end) in &strokes {
            assert!(*w_start > 0.0 && *w_end > 0.0, "pen width must be positive");
            assert!(s.p0.x.is_finite() && s.p3.x.is_finite());
        }
    }
}
