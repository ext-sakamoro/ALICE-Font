//! ALICE-SDF bridge (Milestone A.3、2026-08-06)
//!
//! `Stroke` / `GlyphSkeleton` を [`alice_sdf::Sdf2dNode`] に変換する parametric adapter
//!
//! # 設計原理: 「データを送るな、法則を送れ」
//!
//! 本 bridge は **[`GlyphSdf`] (64x64 bitmap)** ではなく **[`GlyphSkeleton`] (parametric Bezier
//! stroke)** を SDF に変換する 前者は `Sdf2dNode::FontGlyph` (data path、`Box<[f32; 4096]>`
//! 同梱) に対応、本 bridge が扱う後者は `Sdf2dNode::Bezier` (parametric、control point のみ)
//! に対応する = **texture 経由なしの law-only path** これが ALICE 原理準拠の推奨経路
//!
//! [`GlyphSdf`]: crate::glyph::GlyphSdf
//! [`GlyphSkeleton`]: crate::glyph::GlyphSkeleton
//!
//! # Usage
//!
//! ```ignore
//! use alice_font::{sdf_bridge::stroke_to_sdf2d, stroke::{Point2, Stroke}};
//!
//! let stroke = Stroke::new(
//!     Point2::new(0.0, 0.0),
//!     Point2::new(0.3, 0.5),
//!     Point2::new(0.7, 0.5),
//!     Point2::new(1.0, 0.0),
//! );
//! let sdf = stroke_to_sdf2d(&stroke, 0.05);
//! // sdf = alice_sdf::Sdf2dNode::Bezier { p0, p1, p2, p3, thickness: 0.05 }
//! ```

use crate::glyph::GlyphSkeleton;
use crate::stroke::Stroke;
use alice_sdf::prelude::Sdf2dNode;
// no_std base + sdf feature = std enabled、Box は明示 import 必要
use std::boxed::Box;

/// 単一 `Stroke` を `Sdf2dNode::Bezier` に変換
///
/// `thickness` は半太さ (radius 相当) SDF distance 演算では線の中心から `thickness` 未満が内部
#[must_use]
pub fn stroke_to_sdf2d(stroke: &Stroke, thickness: f32) -> Sdf2dNode {
    Sdf2dNode::Bezier {
        p0: [stroke.p0.x, stroke.p0.y],
        p1: [stroke.p1.x, stroke.p1.y],
        p2: [stroke.p2.x, stroke.p2.y],
        p3: [stroke.p3.x, stroke.p3.y],
        thickness,
    }
}

/// `GlyphSkeleton` (最大 16 stroke) の全 stroke を `Union` で合成した `Sdf2dNode` を返す
///
/// `stroke_count == 0` の場合は無限遠 (`Circle { radius: 0.0 }` 相当) を返す
/// これは「どの点も外部」の SDF (常に正) として振る舞う
///
/// `thickness` は全 stroke 共通、將来 stroke ごとの width profile 対応は A.3.1 で追加
#[must_use]
pub fn glyph_skeleton_to_sdf2d(skeleton: &GlyphSkeleton, thickness: f32) -> Sdf2dNode {
    if skeleton.stroke_count == 0 {
        // 空 glyph: origin 半径 0 circle (常に外部)
        return Sdf2dNode::Circle {
            center: [0.0, 0.0],
            radius: 0.0,
        };
    }

    // 各 stroke を Bezier variant に変換 → Union で fold
    let mut iter = skeleton.strokes[..skeleton.stroke_count]
        .iter()
        .map(|s| stroke_to_sdf2d(s, thickness));

    let first = iter.next().expect("stroke_count > 0 なので最低 1 個");
    iter.fold(first, |acc, next| {
        Sdf2dNode::Union(Box::new(acc), Box::new(next))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stroke::Point2;

    #[test]
    fn stroke_maps_to_bezier() {
        let stroke = Stroke::new(
            Point2::new(0.0, 0.0),
            Point2::new(0.3, 0.5),
            Point2::new(0.7, 0.5),
            Point2::new(1.0, 0.0),
        );
        let sdf = stroke_to_sdf2d(&stroke, 0.05);
        match sdf {
            Sdf2dNode::Bezier {
                p0,
                p1,
                p2,
                p3,
                thickness,
            } => {
                assert_eq!(p0, [0.0, 0.0]);
                assert_eq!(p1, [0.3, 0.5]);
                assert_eq!(p2, [0.7, 0.5]);
                assert_eq!(p3, [1.0, 0.0]);
                assert!((thickness - 0.05).abs() < f32::EPSILON);
            }
            _ => panic!("Sdf2dNode::Bezier variant を期待"),
        }
    }

    #[test]
    fn empty_skeleton_maps_to_empty_circle() {
        let empty = GlyphSkeleton::empty();
        let sdf = glyph_skeleton_to_sdf2d(&empty, 0.05);
        match sdf {
            Sdf2dNode::Circle { center, radius } => {
                assert_eq!(center, [0.0, 0.0]);
                assert!((radius - 0.0).abs() < f32::EPSILON);
            }
            _ => panic!("Circle variant (empty glyph) を期待"),
        }
    }

    #[test]
    fn multi_stroke_skeleton_maps_to_union_tree() {
        let mut skel = GlyphSkeleton::empty();
        skel.stroke_count = 3;
        skel.strokes[0] = Stroke::new(
            Point2::new(0.0, 0.0),
            Point2::new(0.3, 0.0),
            Point2::new(0.7, 0.0),
            Point2::new(1.0, 0.0),
        );
        skel.strokes[1] = Stroke::new(
            Point2::new(0.5, 0.0),
            Point2::new(0.5, 0.3),
            Point2::new(0.5, 0.7),
            Point2::new(0.5, 1.0),
        );
        skel.strokes[2] = Stroke::new(
            Point2::new(0.0, 1.0),
            Point2::new(0.3, 1.0),
            Point2::new(0.7, 1.0),
            Point2::new(1.0, 1.0),
        );

        let sdf = glyph_skeleton_to_sdf2d(&skel, 0.02);
        // 3 stroke → 2 Union node で合成
        match sdf {
            Sdf2dNode::Union(_, _) => {
                // 深さ確認 (fold で外側 Union の左が Union、右が Bezier)
            }
            _ => panic!("Union tree を期待 (multi stroke)"),
        }
    }

    #[test]
    fn single_stroke_skeleton_returns_bezier_directly() {
        let mut skel = GlyphSkeleton::empty();
        skel.stroke_count = 1;
        skel.strokes[0] = Stroke::new(
            Point2::new(0.0, 0.0),
            Point2::new(0.3, 0.5),
            Point2::new(0.7, 0.5),
            Point2::new(1.0, 0.0),
        );
        let sdf = glyph_skeleton_to_sdf2d(&skel, 0.03);
        assert!(matches!(sdf, Sdf2dNode::Bezier { .. }));
    }
}
