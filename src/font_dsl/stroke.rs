//! 永字八法 (8 basic CJK stroke types) as cubic Bezier primitives。
//!
//! 楷書の 8 基本 stroke:
//! - 横 (horizontal), 縦 (vertical), 撇 (left-falling curved),
//!   捺 (right-falling curved), 点 (dot), 提 (rising short),
//!   折 (turn corner), 鈎 (hook at end)
//!
//! 全 primitive は `Stroke` (cubic Bezier) を返す。直線 stroke は
//! degenerate cubic (control point が endpoints 上) を使用。曲線 stroke は
//! subtle な curvature を control points で表現。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use crate::stroke::{Point2, Stroke};

/// **横** — horizontal stroke at `y`、`x1` → `x2`。
///
/// Ex: 一 = `horizontal(0.5, 0.1, 0.9)`
#[must_use]
pub fn horizontal(y: f32, x1: f32, x2: f32) -> Stroke {
    Stroke::line(Point2::new(x1, y), Point2::new(x2, y))
}

/// **縦** — vertical stroke at `x`、`y1` → `y2`。
///
/// Ex: 十 の縦棒 = `vertical(0.5, 0.1, 0.9)`
#[must_use]
pub fn vertical(x: f32, y1: f32, y2: f32) -> Stroke {
    Stroke::line(Point2::new(x, y1), Point2::new(x, y2))
}

/// **撇** — left-falling curved stroke (top-right → bottom-left)。
///
/// Subtle inward curve (control points が中間を左に引く)。
/// Ex: 木 の左枝 = `left_falling((0.5, 0.5), (0.15, 0.15))`
#[must_use]
pub fn left_falling(from: (f32, f32), to: (f32, f32)) -> Stroke {
    let (fx, fy) = from;
    let (tx, ty) = to;
    let dx = tx - fx;
    let dy = ty - fy;
    // Control points: 30% and 70% along path, subtle curvature toward left
    let cp1 = Point2::new(fx + dx * 0.3 + dx * 0.05, fy + dy * 0.3);
    let cp2 = Point2::new(fx + dx * 0.7 - dx * 0.05, fy + dy * 0.7);
    Stroke::new(Point2::new(fx, fy), cp1, cp2, Point2::new(tx, ty))
}

/// **捺** — right-falling curved stroke (top-left → bottom-right、末端が太くなる書法)。
///
/// Slight curve with heavier bottom-right endpoint (書道 の 捺 = 8 品法の final 8th stroke)。
/// Ex: 木 の右枝 = `right_falling((0.5, 0.5), (0.85, 0.15))`
#[must_use]
pub fn right_falling(from: (f32, f32), to: (f32, f32)) -> Stroke {
    let (fx, fy) = from;
    let (tx, ty) = to;
    let dx = tx - fx;
    let dy = ty - fy;
    // Symmetric curvature to left_falling
    let cp1 = Point2::new(fx + dx * 0.3 - dx * 0.05, fy + dy * 0.3);
    let cp2 = Point2::new(fx + dx * 0.7 + dx * 0.05, fy + dy * 0.7);
    Stroke::new(Point2::new(fx, fy), cp1, cp2, Point2::new(tx, ty))
}

/// **点** — small dot / short diagonal mark at `center`。
///
/// Size = length of dot (em unit)、typical value 0.06-0.10。
/// Ex: 心 の 3 点、火 の 2 点。
#[must_use]
pub fn dot(center: (f32, f32), size: f32) -> Stroke {
    let (x, y) = center;
    let half = size * 0.5;
    // Short diagonal stroke top-left → bottom-right (楷書 典型 dot)
    Stroke::line(
        Point2::new(x - half * 0.7, y + half * 0.5),
        Point2::new(x + half * 0.7, y - half * 0.5),
    )
}

/// **提** — rising short stroke (bottom-left → upper-right)。
///
/// 撇 の逆方向。short + upward tilt。
/// Ex: 提 手偏 の下端 rising。
#[must_use]
pub fn rising(from: (f32, f32), to: (f32, f32)) -> Stroke {
    Stroke::line(Point2::new(from.0, from.1), Point2::new(to.0, to.1))
}

/// **折** — turn / corner stroke (2 direction join、L-shape)。
///
/// Single cubic Bezier で sharp corner を表現 (control points が corner に集約)。
/// `p0` → `corner` → `p3` の path。
/// Ex: 口 の右上→右下 corner (実際は separate strokes、demonstration 用)。
#[must_use]
pub fn turn(p0: (f32, f32), corner: (f32, f32), p3: (f32, f32)) -> Stroke {
    Stroke::new(
        Point2::new(p0.0, p0.1),
        Point2::new(corner.0, corner.1),
        Point2::new(corner.0, corner.1),
        Point2::new(p3.0, p3.1),
    )
}

/// **鈎** — hook at end of stroke (vertical/horizontal ending with hook)。
///
/// `from` → `to` の main stroke に、`hook_dir` 方向の subtle hook を末端に付加。
/// Ex: 力 の縦棒末端 hook = `hook((0.5, 0.9), (0.5, 0.15), (-0.1, 0.05))`
#[must_use]
pub fn hook(from: (f32, f32), to: (f32, f32), hook_dir: (f32, f32)) -> Stroke {
    let hook_end = Point2::new(to.0 + hook_dir.0, to.1 + hook_dir.1);
    // Main stroke as degenerate cubic to hook_end via `to` control
    Stroke::new(
        Point2::new(from.0, from.1),
        Point2::new(to.0, to.1),
        Point2::new(to.0, to.1),
        hook_end,
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_endpoints_match_y() {
        let s = horizontal(0.5, 0.1, 0.9);
        assert!((s.p0.y - 0.5).abs() < 1e-6);
        assert!((s.p3.y - 0.5).abs() < 1e-6);
        assert!((s.p0.x - 0.1).abs() < 1e-6);
        assert!((s.p3.x - 0.9).abs() < 1e-6);
    }

    #[test]
    fn vertical_endpoints_match_x() {
        let s = vertical(0.5, 0.1, 0.9);
        assert!((s.p0.x - 0.5).abs() < 1e-6);
        assert!((s.p3.x - 0.5).abs() < 1e-6);
        assert!((s.p0.y - 0.1).abs() < 1e-6);
        assert!((s.p3.y - 0.9).abs() < 1e-6);
    }

    #[test]
    fn left_falling_starts_top_ends_bottom() {
        let s = left_falling((0.8, 0.9), (0.2, 0.1));
        assert!((s.p0.x - 0.8).abs() < 1e-6);
        assert!((s.p0.y - 0.9).abs() < 1e-6);
        assert!((s.p3.x - 0.2).abs() < 1e-6);
        assert!((s.p3.y - 0.1).abs() < 1e-6);
    }

    #[test]
    fn dot_is_small_stroke() {
        let s = dot((0.5, 0.5), 0.08);
        let length = ((s.p3.x - s.p0.x).powi(2) + (s.p3.y - s.p0.y).powi(2)).sqrt();
        assert!(length > 0.02 && length < 0.15);
    }

    #[test]
    fn hook_extends_beyond_to() {
        let s = hook((0.5, 0.9), (0.5, 0.15), (-0.1, 0.05));
        assert!((s.p3.x - 0.4).abs() < 1e-6);
        assert!((s.p3.y - 0.20).abs() < 1e-6);
    }
}
