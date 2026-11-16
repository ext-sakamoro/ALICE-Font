//! Glyph outlines drawn on a deformable lattice
//!
//! A glyph's box (u across its advance, v up the em box, both `0..1`) carries an `n x n` lattice of
//! nodes; the caller moves the nodes ([`rest_lattice`] gives them at rest) and the glyph is drawn
//! deformed with them. The outline is still the bezier outline evaluated per pixel (no texture): each
//! pixel is mapped back to the rest glyph through the triangle of the deformed mesh it lies in, and the
//! plain shader's evaluation runs at that rest point ([`LATTICE_OUTLINE_VERT_GLSL`] /
//! [`LATTICE_OUTLINE_FRAG_GLSL`], same uniform block as [`super::MULTI_OUTLINE_VERT_GLSL`]).
//!
//! The mesh ([`lattice_triangles`]): every lattice cell is split `k x k`. A sub-vertex lies at its rest
//! point plus the Catmull-Rom (bicubic, through the nodes) interpolation of the nodes' displacements
//! from rest; past the lattice's border a ghost node continues the border quadratically (`d(-1) =
//! 3 d(0) - 3 d(1) + d(2)`; linearly, `2 d(0) - d(1)`, when `n = 2`). Each sub-quad is cut along its
//! diagonal from `(a, b)` to `(a + 1, b + 1)` into the triangles `(a,b) (a+1,b) (a+1,b+1)` and
//! `(a,b) (a+1,b+1) (a,b+1)`, counter-clockwise in `(u, v)`. Hence:
//! - at rest every displacement is exactly 0 and every deformed vertex equals its rest vertex bit for
//!   bit, so the draw is the plain draw of the same text;
//! - Catmull-Rom reproduces quadratics and the ghosts continue them, so an affine move of the nodes moves
//!   every sub-vertex by the same affine map, and a move quadratic in u and v (with `n >= 3`) puts every
//!   sub-vertex on it (up to rounding);
//! - a node is a vertex of the mesh at exactly the position given;
//! - `k = 1` gives one piecewise-affine map per triangle of the lattice itself (creases along the cell
//!   edges and diagonals under a bend); a larger `k` follows the smooth interpolant more closely, at
//!   `2 (n - 1)^2 k^2` triangles per glyph.
//!
//! Author: Moroya Sakamoto

use alloc::vec::Vec;
use core::fmt;

/// The vertex shader (GLSL ES 3.00, WebGL2) of the lattice draw: one non-instanced draw of the vertices
/// [`lattice_vertices`] writes, with the uniform block of [`super::MultiOutlineGlyphPayload::to_std140_ubo_chunks`]
///
/// Attributes: `aSlot` (glyph slot in the chunk, which vertex of its triangle), `aD01`, `aD2R0`, `aR12`,
/// `aO01`, `aO2` (the triangle's deformed and rest vertices and their outward directions, see
/// [`LATTICE_VERTEX_FLOATS`]); uniforms `uCanvasSize` and `uYPadding` as for the plain vertex shader
pub const LATTICE_OUTLINE_VERT_GLSL: &str =
    include_str!("../../shaders/analytical_lattice_outline.vert.glsl");

/// The fragment shader (GLSL ES 3.00, WebGL2) paired with [`LATTICE_OUTLINE_VERT_GLSL`]: the pixel's
/// glyph-box point as the plain fragment shader computes it, moved by the deformation back to the rest
/// glyph, then the plain shader's evaluation (same uniforms as [`super::MULTI_OUTLINE_FRAG_GLSL`])
pub const LATTICE_OUTLINE_FRAG_GLSL: &str =
    include_str!("../../shaders/analytical_lattice_outline.frag.glsl");

/// Floats per vertex written by [`lattice_vertices`], in order: glyph slot, which vertex of the triangle
/// (0..2), `D0 D1 D2` (deformed vertices), `R0 R1 R2` (rest vertices), `O0 O1 O2` (outward directions),
/// each point two floats `(u, v)`
pub const LATTICE_VERTEX_FLOATS: usize = 20;

/// The most nodes along one side of a glyph's lattice
pub const LATTICE_MAX_NODES_PER_SIDE: usize = 64;

/// The largest subdivision of a lattice cell
pub const LATTICE_MAX_SUBDIVISION: usize = 16;

/// Why a lattice was refused
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LatticeError {
    /// `n` is not in `2..=LATTICE_MAX_NODES_PER_SIDE`
    NodesPerSide(usize),
    /// `k` is not in `1..=LATTICE_MAX_SUBDIVISION`
    Subdivision(usize),
    /// `positions` does not hold `n * n` nodes
    PositionCount { expected: usize, got: usize },
    /// A node position is not finite
    NonFinite { node: usize },
    /// The glyph slot is not in `0..4`
    Slot(usize),
}

impl fmt::Display for LatticeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodesPerSide(n) => write!(
                f,
                "lattice of {n} nodes per side (2..={LATTICE_MAX_NODES_PER_SIDE} expected)"
            ),
            Self::Subdivision(k) => write!(
                f,
                "subdivision {k} (1..={LATTICE_MAX_SUBDIVISION} expected)"
            ),
            Self::PositionCount { expected, got } => {
                write!(f, "{got} node positions ({expected} expected)")
            }
            Self::NonFinite { node } => write!(f, "node {node} is not finite"),
            Self::Slot(s) => write!(f, "glyph slot {s} (0..4 expected)"),
        }
    }
}

/// One triangle of a glyph's lattice mesh, in the glyph's box coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatticeTriangle {
    /// The vertices where the deformation put them
    pub deformed: [[f32; 2]; 3],
    /// The same vertices at rest
    pub rest: [[f32; 2]; 3],
    /// Each vertex's outward direction on the lattice's border (`-1`, `0` or `1` per axis; `0` inside)
    pub outward: [[f32; 2]; 3],
}

fn check_n(n: usize) -> Result<(), LatticeError> {
    if (2..=LATTICE_MAX_NODES_PER_SIDE).contains(&n) {
        Ok(())
    } else {
        Err(LatticeError::NodesPerSide(n))
    }
}

/// The Catmull-Rom weights of the nodes `-1, 0, 1, 2` around a cell at the parameter `s` in `0..=1`
/// (exact `0, 1, 0, 0` at `s = 0` and `0, 0, 1, 0` at `s = 1`)
fn catmull_rom(s: f32) -> [f32; 4] {
    let s2 = s * s;
    let s3 = s2 * s;
    [
        (-s + 2.0 * s2 - s3) * 0.5,
        (2.0 - 5.0 * s2 + 3.0 * s3) * 0.5,
        (s + 4.0 * s2 - 3.0 * s3) * 0.5,
        (s3 - s2) * 0.5,
    ]
}

/// The displacements of a lattice's nodes from rest, read with ghosts past the border (see the module
/// doc); a ghost is only ever one node past the border
struct Displacements {
    n: usize,
    d: Vec<[f32; 2]>,
}

impl Displacements {
    /// the ghost past `a` from the border nodes `a, b, c` (inward): quadratic, or linear without `c`
    fn ghost(a: [f32; 2], b: [f32; 2], c: Option<[f32; 2]>) -> [f32; 2] {
        match c {
            Some(c) => [
                3.0 * a[0] - 3.0 * b[0] + c[0],
                3.0 * a[1] - 3.0 * b[1] + c[1],
            ],
            None => [2.0 * a[0] - b[0], 2.0 * a[1] - b[1]],
        }
    }

    fn at(&self, i: isize, j: isize) -> [f32; 2] {
        let n = self.n as isize;
        let third = |k: isize| if n >= 3 { Some(k) } else { None };
        if j < 0 {
            return Self::ghost(
                self.at(i, 0),
                self.at(i, 1),
                third(2).map(|k| self.at(i, k)),
            );
        }
        if j >= n {
            return Self::ghost(
                self.at(i, n - 1),
                self.at(i, n - 2),
                third(n - 3).map(|k| self.at(i, k)),
            );
        }
        if i < 0 {
            return Self::ghost(
                self.at(0, j),
                self.at(1, j),
                third(2).map(|k| self.at(k, j)),
            );
        }
        if i >= n {
            return Self::ghost(
                self.at(n - 1, j),
                self.at(n - 2, j),
                third(n - 3).map(|k| self.at(k, j)),
            );
        }
        self.d[j as usize * self.n + i as usize]
    }
}

/// The nodes of an `n x n` lattice at rest, rows from `v = 0` up, each row from `u = 0`: node `(i, j)`
/// is at `(i / (n - 1), j / (n - 1))`
///
/// # Errors
/// [`LatticeError::NodesPerSide`] when `n` is out of range
pub fn rest_lattice(n: usize) -> Result<Vec<[f32; 2]>, LatticeError> {
    check_n(n)?;
    let last = (n - 1) as f32;
    let mut out = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            out.push([i as f32 / last, j as f32 / last]);
        }
    }
    Ok(out)
}

/// The triangles of one glyph's lattice mesh (see the module doc) for the nodes `positions` (`n * n`,
/// in the order of [`rest_lattice`]) and the subdivision `k`
///
/// # Errors
/// A [`LatticeError`] when `n`, `k` or the positions are out of range
pub fn lattice_triangles(
    n: usize,
    k: usize,
    positions: &[[f32; 2]],
) -> Result<Vec<LatticeTriangle>, LatticeError> {
    let rest = rest_lattice(n)?;
    if !(1..=LATTICE_MAX_SUBDIVISION).contains(&k) {
        return Err(LatticeError::Subdivision(k));
    }
    if positions.len() != n * n {
        return Err(LatticeError::PositionCount {
            expected: n * n,
            got: positions.len(),
        });
    }
    if let Some(node) = positions
        .iter()
        .position(|p| !(p[0].is_finite() && p[1].is_finite()))
    {
        return Err(LatticeError::NonFinite { node });
    }
    let disp = Displacements {
        n,
        d: positions
            .iter()
            .zip(&rest)
            .map(|(p, r)| [p[0] - r[0], p[1] - r[1]])
            .collect(),
    };
    // the sub-vertex grid: m x m points, (m - 1) = (n - 1) k intervals per side
    let m = (n - 1) * k + 1;
    let last = (m - 1) as f32;
    let kf = k as f32;
    let mut grid = Vec::with_capacity(m * m);
    for b in 0..m {
        let cj = (b / k).min(n - 2);
        let t = (b - cj * k) as f32 / kf;
        let wt = catmull_rom(t);
        for a in 0..m {
            let ci = (a / k).min(n - 2);
            let s = (a - ci * k) as f32 / kf;
            let r = [a as f32 / last, b as f32 / last];
            let on_node_u = a % k == 0;
            let on_node_v = b % k == 0;
            let deformed = if on_node_u && on_node_v {
                // a node: the position as given
                positions[(b / k) * n + a / k]
            } else {
                let ws = catmull_rom(s);
                let mut acc = [0.0f32; 2];
                for (y, wy) in wt.iter().enumerate() {
                    for (x, wx) in ws.iter().enumerate() {
                        let dd =
                            disp.at(ci as isize + x as isize - 1, cj as isize + y as isize - 1);
                        let w = wx * wy;
                        acc[0] += w * dd[0];
                        acc[1] += w * dd[1];
                    }
                }
                [r[0] + acc[0], r[1] + acc[1]]
            };
            let out = |c: usize| -> f32 {
                if c == 0 {
                    -1.0
                } else if c == m - 1 {
                    1.0
                } else {
                    0.0
                }
            };
            grid.push((deformed, r, [out(a), out(b)]));
        }
    }
    let mut tris = Vec::with_capacity(2 * (m - 1) * (m - 1));
    let tri = |p: [usize; 3]| LatticeTriangle {
        deformed: p.map(|i| grid[i].0),
        rest: p.map(|i| grid[i].1),
        outward: p.map(|i| grid[i].2),
    };
    for b in 0..m - 1 {
        for a in 0..m - 1 {
            let v00 = b * m + a;
            let v10 = v00 + 1;
            let v01 = v00 + m;
            let v11 = v01 + 1;
            tris.push(tri([v00, v10, v11]));
            tris.push(tri([v00, v11, v01]));
        }
    }
    Ok(tris)
}

/// Append the vertices of `triangles` for the glyph in `slot` of its chunk to `out`
/// ([`LATTICE_VERTEX_FLOATS`] floats per vertex, three vertices per triangle)
///
/// # Errors
/// [`LatticeError::Slot`] when `slot` is not in `0..4`
pub fn lattice_vertices(
    slot: usize,
    triangles: &[LatticeTriangle],
    out: &mut Vec<f32>,
) -> Result<(), LatticeError> {
    if slot >= super::MULTI_OUTLINE_MAX_GLYPHS {
        return Err(LatticeError::Slot(slot));
    }
    out.reserve(triangles.len() * 3 * LATTICE_VERTEX_FLOATS);
    for t in triangles {
        for which in 0..3 {
            out.push(slot as f32);
            out.push(which as f32);
            for p in t.deformed.iter().chain(&t.rest).chain(&t.outward) {
                out.extend_from_slice(p);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outline::{MULTI_OUTLINE_FRAG_GLSL, MULTI_OUTLINE_VERT_GLSL};
    use alloc::vec;

    fn area(t: &[[f32; 2]; 3]) -> f64 {
        let [a, b, c] = t.map(|p| [f64::from(p[0]), f64::from(p[1])]);
        ((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])) / 2.0
    }

    // oracle: node (i, j) of a rest lattice is (i / (n - 1), j / (n - 1)), each a correctly rounded
    // quotient of two small integers
    #[test]
    fn rest_lattice_places_node_i_j_at_i_and_j_over_n_minus_1() {
        let r = rest_lattice(4).unwrap();
        assert_eq!(r.len(), 16);
        assert_eq!(r[0], [0.0, 0.0]);
        assert_eq!(r[3], [1.0, 0.0]);
        assert_eq!(r[4], [0.0, 1.0 / 3.0]);
        assert_eq!(r[15], [1.0, 1.0]);
        assert_eq!(r[9], [1.0 / 3.0, 2.0 / 3.0]);
    }

    // oracle: at rest every displacement is 0, so the mesh is the rest mesh bit for bit; the rest
    // triangles tile the unit box (areas add to 1, all counter-clockwise)
    #[test]
    fn at_rest_every_deformed_vertex_is_its_rest_vertex_bit_for_bit() {
        for (n, k) in [(2, 1), (3, 1), (4, 3), (8, 4), (5, 16)] {
            let tris = lattice_triangles(n, k, &rest_lattice(n).unwrap()).unwrap();
            assert_eq!(tris.len(), 2 * (n - 1) * (n - 1) * k * k, "n {n} k {k}");
            let mut sum = 0.0;
            for t in &tris {
                for v in 0..3 {
                    assert_eq!(t.deformed[v][0].to_bits(), t.rest[v][0].to_bits());
                    assert_eq!(t.deformed[v][1].to_bits(), t.rest[v][1].to_bits());
                }
                let a = area(&t.rest);
                assert!(
                    a > 0.0,
                    "n {n} k {k}: a rest triangle is not counter-clockwise"
                );
                sum += a;
            }
            assert!(
                (sum - 1.0).abs() < 1e-9,
                "n {n} k {k}: rest triangles cover {sum}"
            );
        }
    }

    // oracle: an affine move x -> A x + b of the nodes moves every sub-vertex to A r + b (Catmull-Rom
    // with linear ghosts reproduces affine maps); tolerance a few f32 ulps of the unit box
    #[test]
    fn an_affine_move_of_the_nodes_moves_every_vertex_by_the_same_affine_map() {
        let a = [[0.9_f32, -0.3], [0.2, 1.1]];
        let b = [0.07_f32, -0.05];
        let f = |p: [f32; 2]| {
            [
                a[0][0] * p[0] + a[0][1] * p[1] + b[0],
                a[1][0] * p[0] + a[1][1] * p[1] + b[1],
            ]
        };
        for (n, k) in [(2, 1), (3, 3), (6, 4)] {
            let pos: Vec<[f32; 2]> = rest_lattice(n).unwrap().into_iter().map(f).collect();
            for t in lattice_triangles(n, k, &pos).unwrap() {
                for v in 0..3 {
                    let want = f(t.rest[v]);
                    for c in 0..2 {
                        assert!(
                            (t.deformed[v][c] - want[c]).abs() < 4e-6,
                            "n {n} k {k}: {:?} vs {want:?}",
                            t.deformed[v]
                        );
                    }
                }
            }
        }
    }

    // oracle: the mesh passes through the nodes: a sub-vertex at a node is the given position exactly
    #[test]
    fn every_node_is_a_mesh_vertex_at_exactly_the_given_position() {
        let n = 4;
        let k = 3;
        let mut pos = rest_lattice(n).unwrap();
        for (i, p) in pos.iter_mut().enumerate() {
            p[0] += 0.013 * (i as f32).sin_cos_free_wobble();
            p[1] -= 0.007 * (i % 5) as f32;
        }
        let tris = lattice_triangles(n, k, &pos).unwrap();
        let rest = rest_lattice(n).unwrap();
        for (node, (&want, &r)) in pos.iter().zip(&rest).enumerate() {
            let hit = tris
                .iter()
                .any(|t| (0..3).any(|v| t.rest[v] == r && t.deformed[v] == want));
            assert!(hit, "node {node} at {want:?} is not a vertex of the mesh");
        }
    }

    trait Wobble {
        fn sin_cos_free_wobble(self) -> f32;
    }
    impl Wobble for f32 {
        // a deterministic bounded sequence without libm (the crate bans the platform transcendentals)
        fn sin_cos_free_wobble(self) -> f32 {
            let x = self * 0.37;
            (x - (x + 0.5).floor()) * 2.0
        }
    }

    // oracle: the outward direction is -1 / +1 on the border's axis and 0 inside
    #[test]
    fn border_vertices_point_outward_and_inner_ones_do_not() {
        let n = 3;
        for t in lattice_triangles(n, 2, &rest_lattice(n).unwrap()).unwrap() {
            for v in 0..3 {
                for c in 0..2 {
                    let r = t.rest[v][c];
                    let want = if r == 0.0 {
                        -1.0
                    } else if r == 1.0 {
                        1.0
                    } else {
                        0.0
                    };
                    assert_eq!(t.outward[v][c], want, "{:?}", t.rest[v]);
                }
            }
        }
    }

    // oracle: each sub-quad is cut along (a, b)-(a + 1, b + 1): with n = 2, k = 1 the two triangles are
    // (0,0) (1,0) (1,1) and (0,0) (1,1) (0,1)
    #[test]
    fn the_cut_runs_from_a_b_to_a_plus_1_b_plus_1() {
        let tris = lattice_triangles(2, 1, &rest_lattice(2).unwrap()).unwrap();
        assert_eq!(
            tris.iter().map(|t| t.rest).collect::<Vec<_>>(),
            vec![
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
                [[0.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            ]
        );
    }

    #[test]
    fn out_of_range_inputs_are_refused_with_their_reason() {
        let r3 = rest_lattice(3).unwrap();
        assert_eq!(rest_lattice(1), Err(LatticeError::NodesPerSide(1)));
        assert_eq!(rest_lattice(0), Err(LatticeError::NodesPerSide(0)));
        assert_eq!(
            rest_lattice(LATTICE_MAX_NODES_PER_SIDE + 1),
            Err(LatticeError::NodesPerSide(LATTICE_MAX_NODES_PER_SIDE + 1))
        );
        assert_eq!(
            lattice_triangles(3, 0, &r3),
            Err(LatticeError::Subdivision(0))
        );
        assert_eq!(
            lattice_triangles(3, LATTICE_MAX_SUBDIVISION + 1, &r3),
            Err(LatticeError::Subdivision(LATTICE_MAX_SUBDIVISION + 1))
        );
        assert_eq!(
            lattice_triangles(3, 1, &r3[..8]),
            Err(LatticeError::PositionCount {
                expected: 9,
                got: 8
            })
        );
        let mut bad = r3.clone();
        bad[4][1] = f32::NAN;
        assert_eq!(
            lattice_triangles(3, 1, &bad),
            Err(LatticeError::NonFinite { node: 4 })
        );
        bad[4][1] = f32::INFINITY;
        assert_eq!(
            lattice_triangles(3, 1, &bad),
            Err(LatticeError::NonFinite { node: 4 })
        );
        let tris = lattice_triangles(3, 1, &r3).unwrap();
        let mut out = Vec::new();
        assert_eq!(
            lattice_vertices(4, &tris, &mut out),
            Err(LatticeError::Slot(4))
        );
        assert!(out.is_empty(), "a refused slot writes nothing");
    }

    // oracle: the vertex layout of LATTICE_VERTEX_FLOATS, written out by hand for one triangle
    #[test]
    fn vertices_carry_the_whole_triangle_in_the_documented_order() {
        let t = LatticeTriangle {
            deformed: [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]],
            rest: [[7.0, 8.0], [9.0, 10.0], [11.0, 12.0]],
            outward: [[-1.0, 0.0], [1.0, 1.0], [0.0, -1.0]],
        };
        let mut out = vec![99.0];
        lattice_vertices(2, &[t], &mut out).unwrap();
        assert_eq!(out.len(), 1 + 3 * LATTICE_VERTEX_FLOATS);
        let body = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, -1.0, 0.0, 1.0, 1.0,
            0.0, -1.0,
        ];
        for which in 0..3 {
            let v =
                &out[1 + which * LATTICE_VERTEX_FLOATS..1 + (which + 1) * LATTICE_VERTEX_FLOATS];
            assert_eq!(v[0], 2.0, "slot");
            assert_eq!(v[1], which as f32, "which vertex");
            assert_eq!(&v[2..], &body[..]);
        }
    }

    /// The image of the rest vertical line `u = u0` under the mesh's piecewise-affine map: in each
    /// triangle the line crosses, the image of the direction `(0, 1)`; returns the largest jump between
    /// consecutive directions, as the sine of the angle (`|a x b| / (|a| |b|)`, no transcendental),
    /// split into jumps across a sub-quad's diagonal and jumps across its horizontal edge
    fn crease_sines(tris: &[LatticeTriangle], u0: f64) -> (f64, f64) {
        let mut hits: Vec<(f64, bool, [f64; 2])> = Vec::new(); // (v of the sub-quad, lower triangle, direction)
        for t in tris {
            let r = t.rest.map(|p| [f64::from(p[0]), f64::from(p[1])]);
            let d = t.deformed.map(|p| [f64::from(p[0]), f64::from(p[1])]);
            let umin = r.iter().map(|p| p[0]).fold(f64::MAX, f64::min);
            let umax = r.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
            if !(umin < u0 && u0 < umax) {
                continue;
            }
            let e1 = [r[1][0] - r[0][0], r[1][1] - r[0][1]];
            let e2 = [r[2][0] - r[0][0], r[2][1] - r[0][1]];
            let f1 = [d[1][0] - d[0][0], d[1][1] - d[0][1]];
            let f2 = [d[2][0] - d[0][0], d[2][1] - d[0][1]];
            let det = e1[0] * e2[1] - e1[1] * e2[0];
            // (0, 1) = a e1 + b e2
            let a = -e2[0] / det;
            let b = e1[0] / det;
            let img = [a * f1[0] + b * f2[0], a * f1[1] + b * f2[1]];
            // the lower triangle of a sub-quad is (a,b) (a+1,b) (a+1,b+1): its first two rest vertices share v
            let lower = r[1][1] == r[0][1];
            let vmin = r.iter().map(|p| p[1]).fold(f64::MAX, f64::min);
            hits.push((vmin, lower, img));
        }
        // up the line: sub-quad by sub-quad, the lower triangle first
        hits.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap().then(y.1.cmp(&x.1)));
        assert!(hits.len() >= 2, "the line crosses {} triangles", hits.len());
        let (mut diag, mut edge) = (0.0_f64, 0.0_f64);
        for w in hits.windows(2) {
            let (a, b) = (w[0].2, w[1].2);
            let sine = (a[0] * b[1] - a[1] * b[0]).abs()
                / ((a[0] * a[0] + a[1] * a[1]).sqrt() * (b[0] * b[0] + b[1] * b[1]).sqrt());
            if w[0].1 && !w[1].1 {
                diag = diag.max(sine);
            } else {
                edge = edge.max(sine);
            }
        }
        (diag, edge)
    }

    /// A smooth bend of known shape: node (u, v) moves to (u + 0.5 (v - 0.5)^2, v), a parabola in v; the
    /// image of a vertical line turns its direction smoothly, by atan(0.5) ≈ 26.6° from bottom to top
    fn bent(n: usize) -> Vec<[f32; 2]> {
        rest_lattice(n)
            .unwrap()
            .into_iter()
            .map(|[u, v]| {
                let s = v - 0.5;
                [u + 0.5 * s * s, v]
            })
            .collect()
    }

    // oracle (closed form): the bend moves x by 0.5 (v - 0.5)^2, a quadratic, which the mesh reproduces;
    // x depends on v alone, so both triangles of a sub-quad row map (0, 1) to (s, 1) with s the parabola's
    // secant slope over the row (no jump across the diagonal), and consecutive rows differ by exactly
    // h = 1 / (3 k) in slope: across the edge between rows of slopes s1, s2 the jump's sine is
    // h / (sqrt(1 + s1^2) sqrt(1 + s2^2)); the largest over the edges is the expected crease
    #[test]
    fn the_creases_of_a_quadratic_bend_are_the_closed_form_secant_jumps() {
        let n = 4;
        for k in [1, 2, 4, 8] {
            let tris = lattice_triangles(n, k, &bent(n)).unwrap();
            let (diag, edge) = crease_sines(&tris, 0.45);
            let rows = 3 * k;
            let h = 1.0 / rows as f64;
            let slope = |row: usize| (row as f64 + 0.5) * h - 0.5;
            let want = (0..rows - 1)
                .map(|r| {
                    let (s1, s2) = (slope(r), slope(r + 1));
                    h / ((1.0 + s1 * s1).sqrt() * (1.0 + s2 * s2).sqrt())
                })
                .fold(0.0_f64, f64::max);
            #[cfg(feature = "std")]
            std::eprintln!(
                "crease n {n} k {k}: triangles {} diagonal sine {diag:.6} edge sine {edge:.6} (closed form {want:.6})",
                tris.len()
            );
            assert!(diag < 1e-5, "k {k}: diagonal jump {diag}");
            assert!(
                (edge - want).abs() < 1e-4,
                "k {k}: edge jump {edge} vs closed form {want}"
            );
        }
    }

    // oracle: with a linear ghost (the rule for n = 2) a quadratic is not reproduced; with n = 2 the mesh
    // is still exact at rest and affine (covered above), and a bend gives a single straight column
    #[test]
    fn a_two_node_lattice_is_bilinear_in_the_nodes() {
        // n = 2, every Catmull-Rom neighbour is a node or a linear ghost: the mesh is the bilinear map of
        // the four corners; with the top corners moved right by 0.2, a point at height v moves by 0.2 v
        let pos = [[0.0, 0.0], [1.0, 0.0], [0.2, 1.0], [1.2, 1.0]];
        for t in lattice_triangles(2, 4, &pos).unwrap() {
            for v in 0..3 {
                let r = t.rest[v];
                assert!(
                    (t.deformed[v][0] - (r[0] + 0.2 * r[1])).abs() < 1e-6,
                    "{r:?}"
                );
                assert!((t.deformed[v][1] - r[1]).abs() < 1e-6, "{r:?}");
            }
        }
    }

    // the plain shader's outline evaluation is copied into the lattice fragment shader byte for byte:
    // the text from `int contourCount` to the end must be equal
    #[test]
    fn the_lattice_shader_evaluates_the_outline_with_the_plain_shaders_text() {
        let tail = |src: &str| -> alloc::string::String {
            let at = src
                .find("    int contourCount = int(meta.y);")
                .expect("evaluation start");
            src[at..].into()
        };
        assert_eq!(
            tail(LATTICE_OUTLINE_FRAG_GLSL),
            tail(MULTI_OUTLINE_FRAG_GLSL)
        );
        // and the pixel's glyph-box point is the plain shader's expressions
        for line in [
            "    vec2 at = gl_FragCoord.xy - uViewOrigin;",
            "    float nx = at.x / w * 2.0 - 1.0;",
            "    float ny = 1.0 - (h - at.y) / h * 2.0;",
            "    float xEm = (nx / sX + 1.0) * 0.5 * totalAdv;",
        ] {
            assert!(MULTI_OUTLINE_FRAG_GLSL.contains(line), "plain: {line}");
            assert!(LATTICE_OUTLINE_FRAG_GLSL.contains(line), "lattice: {line}");
        }
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains("vec2 rest = q + (fromD - fromR);"));
        for src in [LATTICE_OUTLINE_VERT_GLSL, LATTICE_OUTLINE_FRAG_GLSL] {
            assert!(src.starts_with("#version 300 es"));
            assert!(src.contains("layout(std140) uniform MultiOutlineBlock"));
        }
        // the block is declared exactly as the plain shaders declare it
        let block = |src: &str| -> alloc::string::String {
            let s = src
                .find("layout(std140) uniform MultiOutlineBlock")
                .unwrap();
            let e = s + src[s..].find("};").unwrap();
            src[s..e]
                .lines()
                .map(|l| l.split("//").next().unwrap().trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(
            block(LATTICE_OUTLINE_VERT_GLSL),
            block(MULTI_OUTLINE_VERT_GLSL)
        );
        assert_eq!(
            block(LATTICE_OUTLINE_FRAG_GLSL),
            block(MULTI_OUTLINE_FRAG_GLSL)
        );
        for attr in ["aSlot", "aD01", "aD2R0", "aR12", "aO01", "aO2"] {
            assert!(
                LATTICE_OUTLINE_VERT_GLSL.contains(&alloc::format!(" {attr};")),
                "{attr}"
            );
        }
    }
}
