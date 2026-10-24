// MIT License
// Copyright (c) 2026 Moroya Sakamoto
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Shared helper functions for glyph construction.
//!
//! Provides quarter-arc, ellipse, half-arc, and serif primitives built on top
//! of cubic Bezier strokes. All helpers accept a `slant` parameter so that
//! italic variants can reuse the same construction logic.

use crate::stroke::{Point2, Stroke};

/// Bezier approximation constant for a quarter circle.
///
/// For a unit circle the optimal control-point offset is `4/3 * tan(pi/8)`,
/// which is approximately 0.5523.
pub const KAPPA: f32 = 0.5523;

/// Adds a quarter-circle arc to the skeleton as a single cubic Bezier stroke.
///
/// # Quadrants
///
/// - `0` top-right: (cx, cy+ry) to (cx+rx, cy)
/// - `1` bottom-right: (cx+rx, cy) to (cx, cy-ry)
/// - `2` bottom-left: (cx, cy-ry) to (cx-rx, cy)
/// - `3` top-left: (cx-rx, cy) to (cx, cy+ry)
///
/// The resulting stroke has `apply_slant(slant)` called before being added.
pub fn add_quarter_arc(
    skel: &mut super::GlyphSkeleton,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    quadrant: u8,
    slant: f32,
) {
    let kx = rx * KAPPA;
    let ky = ry * KAPPA;

    let (p0, p1, p2, p3) = match quadrant {
        // top-right: start at top (cx, cy+ry), end at right (cx+rx, cy)
        0 => (
            Point2::new(cx, cy + ry),
            Point2::new(cx + kx, cy + ry),
            Point2::new(cx + rx, cy + ky),
            Point2::new(cx + rx, cy),
        ),
        // bottom-right: start at right (cx+rx, cy), end at bottom (cx, cy-ry)
        1 => (
            Point2::new(cx + rx, cy),
            Point2::new(cx + rx, cy - ky),
            Point2::new(cx + kx, cy - ry),
            Point2::new(cx, cy - ry),
        ),
        // bottom-left: start at bottom (cx, cy-ry), end at left (cx-rx, cy)
        2 => (
            Point2::new(cx, cy - ry),
            Point2::new(cx - kx, cy - ry),
            Point2::new(cx - rx, cy - ky),
            Point2::new(cx - rx, cy),
        ),
        // top-left: start at left (cx-rx, cy), end at top (cx, cy+ry)
        3 => (
            Point2::new(cx - rx, cy),
            Point2::new(cx - rx, cy + ky),
            Point2::new(cx - kx, cy + ry),
            Point2::new(cx, cy + ry),
        ),
        _ => return,
    };

    let stroke = Stroke::new(p0, p1, p2, p3).apply_slant(slant);
    skel.add_stroke(stroke);
}

/// Adds a full ellipse to the skeleton as four quarter-arc Bezier strokes.
///
/// The ellipse is centered at `(cx, cy)` with horizontal radius `rx` and
/// vertical radius `ry`. Drawing order: top-right, bottom-right, bottom-left,
/// top-left (clockwise from the top).
pub fn add_ellipse(
    skel: &mut super::GlyphSkeleton,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    slant: f32,
) {
    add_quarter_arc(skel, cx, cy, rx, ry, 0, slant);
    add_quarter_arc(skel, cx, cy, rx, ry, 1, slant);
    add_quarter_arc(skel, cx, cy, rx, ry, 2, slant);
    add_quarter_arc(skel, cx, cy, rx, ry, 3, slant);
}

/// Adds a half-circle arc to the skeleton as two quarter-arc Bezier strokes.
///
/// # Sides
///
/// - `0` right half: top to bottom (quadrants 0 then 1). Used for the right
///   bowl of letters like B, D, P, R.
/// - `1` left half: bottom to top (quadrants 2 then 3). Used for reversed
///   bowls.
pub fn add_half_arc(
    skel: &mut super::GlyphSkeleton,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    side: u8,
    slant: f32,
) {
    match side {
        // right half: top-right then bottom-right
        0 => {
            add_quarter_arc(skel, cx, cy, rx, ry, 0, slant);
            add_quarter_arc(skel, cx, cy, rx, ry, 1, slant);
        }
        // left half: bottom-left then top-left
        1 => {
            add_quarter_arc(skel, cx, cy, rx, ry, 2, slant);
            add_quarter_arc(skel, cx, cy, rx, ry, 3, slant);
        }
        _ => {}
    }
}

/// Adds a serif bracket stroke at `base` extending in `direction`.
///
/// The serif is a short cubic Bezier curve of the given `length` that fans out
/// from the stem. The bracket shape uses control points placed at 1/3 and 2/3
/// of the length along the direction vector, with a slight vertical offset to
/// create the characteristic curved bracket.
///
/// `direction` should be a unit-length vector pointing along the baseline
/// (typically left or right from the stem).
pub fn add_serif_at(
    skel: &mut super::GlyphSkeleton,
    base: Point2,
    direction: Point2,
    length: f32,
    slant: f32,
) {
    // Perpendicular offset for the bracket curve (upward from baseline).
    // A small fraction of the serif length gives a subtle bracket.
    let bracket_height = length * 0.25;

    let perp = Point2::new(-direction.y, direction.x);

    let p0 = base;
    let p1 = Point2::new(
        base.x + direction.x * length * 0.33 + perp.x * bracket_height,
        base.y + direction.y * length * 0.33 + perp.y * bracket_height,
    );
    let p2 = Point2::new(
        base.x + direction.x * length * 0.66,
        base.y + direction.y * length * 0.66,
    );
    let p3 = Point2::new(base.x + direction.x * length, base.y + direction.y * length);

    let stroke = Stroke::new(p0, p1, p2, p3).apply_slant(slant);
    skel.add_stroke(stroke);
}
