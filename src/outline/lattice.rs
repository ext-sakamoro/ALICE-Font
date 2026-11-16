//! Glyph outlines drawn on a deformable lattice
//!
//! A glyph's box (u across its advance, v up the em box, both `0..1`) carries an `n x n` lattice of
//! nodes; the caller moves the nodes ([`rest_lattice`] gives them at rest) and the glyph is drawn
//! deformed with them. The outline is still the bezier outline evaluated per pixel (no texture)
//! ([`LATTICE_OUTLINE_VERT_GLSL`] / [`LATTICE_OUTLINE_FRAG_GLSL`], same uniform block as
//! [`super::MULTI_OUTLINE_VERT_GLSL`]; [`GlyphLattice::rest_point`] is the per-pixel computation on the CPU).
//!
//! The map: the deformation is the Catmull-Rom (bicubic, through the nodes) interpolation of the nodes'
//! displacements from rest, `x = r + disp(r)`; past the lattice's border a ghost node continues the
//! border quadratically (`d(-1) = 3 d(0) - 3 d(1) + d(2)`; linearly, `2 d(0) - d(1)`, when `n = 2`). It is
//! C1 across cells, reproduces affine maps, and reproduces quadratics when `n >= 3`.
//!
//! A pixel: its glyph-box point `q` is computed as the plain shader computes it; its rest point solves
//! `r + disp(r) = q` in the cell of the triangle that covers the pixel, by [`LATTICE_NEWTON_STEPS`] Newton
//! steps (a literal; the triangle's barycentric map gives the first guess). The result does not depend on
//! which triangle a rasterizer gave the pixel to (the two cells at a shared edge agree to rounding), and:
//! - at rest the displacement is exactly 0 and the rest point is `q` bit for bit: the draw is the plain
//!   draw of the same text;
//! - an affine move is solved exactly in one step;
//! - a pixel is not drawn when its residual after the steps is above [`LATTICE_RESIDUAL_PX`] pixels, or
//!   when the solution lies outside the triangle's cell by more than half a sub-cell (the solve went to
//!   another layer of a folded sheet): a deformation the solve cannot follow shows a hole, not a wrong
//!   colour;
//! - where the sheet overlaps itself (or glyphs overlap), each layer is drawn from its own cells and the
//!   pixel shows the most ink of all layers: the fragment shader writes depth `1 - coverage` for a
//!   `LEQUAL` depth test (the draw needs a depth buffer), so the operator is the maximum of coverage, free
//!   of draw order; an inverted cell draws the mirror image. The colour is a function of coverage alone,
//!   which is what makes equal coverages give equal bytes: a per-layer or per-glyph colour would break
//!   that tie rule (two layers of equal coverage and different colour), so adding colour means choosing
//!   another operator on purpose.
//!
//! [`GlyphLattice::regularity`] reports the Jacobian of the map (`det J <= 0`: folded or inverted
//! somewhere), the caller's measure of how hard the lattice is deformed.
//!
//! The mesh ([`GlyphLattice::triangles`]): every lattice cell is split `k x k`, a sub-vertex at its rest
//! point plus the displacement there (a node at exactly the position given), each sub-quad cut along its
//! diagonal from `(a, b)` to `(a + 1, b + 1)` into `(a,b) (a+1,b) (a+1,b+1)` and `(a,b) (a+1,b+1) (a,b+1)`.
//! The mesh decides which pixels are covered and gives the first guess; `k` trades `2 (n - 1)^2 k^2`
//! triangles per glyph against how closely the coverage follows the sheet and how good the first guess
//! is. Measured on recorded soft-body states (a hard fast drag, a sheet folded and stuck): with `k = 4`
//! at most 0.04 % of the covered pixels are left undrawn, with `k = 1` up to 3.3 %.
//!
//! Author: Moroya Sakamoto

use alloc::vec;
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
/// (0..2), the cell `i, j`, `D0 D1 D2` (deformed vertices), `R0 R1 R2` (rest vertices), `O0 O1 O2`
/// (outward directions), each point two floats `(u, v)`
pub const LATTICE_VERTEX_FLOATS: usize = 22;

/// Newton steps per pixel, a literal (the same in [`LATTICE_OUTLINE_FRAG_GLSL`]; a test keeps them equal)
pub const LATTICE_NEWTON_STEPS: usize = 4;

/// The largest residual, in pixels, a pixel's solve may leave and still be drawn
pub const LATTICE_RESIDUAL_PX: f32 = 1.0 / 64.0;

/// Intervals per cell side at which [`GlyphLattice::regularity`] samples the Jacobian (a property of the
/// lattice, not of the mesh's subdivision)
pub const LATTICE_REGULARITY_SAMPLES: usize = 8;

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

impl core::error::Error for LatticeError {}

/// One triangle of a glyph's lattice mesh, in the glyph's box coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatticeTriangle {
    /// The vertices where the deformation put them
    pub deformed: [[f32; 2]; 3],
    /// The same vertices at rest
    pub rest: [[f32; 2]; 3],
    /// Each vertex's outward direction on the lattice's border (`-1`, `0` or `1` per axis; `0` inside)
    pub outward: [[f32; 2]; 3],
    /// The lattice cell `(i, j)` the triangle lies in (its Catmull-Rom patch is the one solved)
    pub cell: [u16; 2],
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

/// The derivatives of [`catmull_rom`]'s weights in `s`
fn catmull_rom_d(s: f32) -> [f32; 4] {
    let s2 = s * s;
    [
        (-1.0 + 4.0 * s - 3.0 * s2) * 0.5,
        (-10.0 * s + 9.0 * s2) * 0.5,
        (1.0 + 8.0 * s - 9.0 * s2) * 0.5,
        (3.0 * s2 - 2.0 * s) * 0.5,
    ]
}

/// The ghost past `a` from the border nodes `a, b, c` (inward): quadratic, or linear without `c`
fn ghost(a: [f32; 2], b: [f32; 2], c: Option<[f32; 2]>) -> [f32; 2] {
    match c {
        Some(c) => [
            3.0 * a[0] - 3.0 * b[0] + c[0],
            3.0 * a[1] - 3.0 * b[1] + c[1],
        ],
        None => [2.0 * a[0] - b[0], 2.0 * a[1] - b[1]],
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

/// How regular a lattice's map is: the Jacobian `J = I + d(displacement)/d(rest)` sampled on a fixed
/// grid of [`LATTICE_REGULARITY_SAMPLES`] intervals per cell side (see [`GlyphLattice::regularity`])
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatticeRegularity {
    /// The smallest `det J`: at or below 0 the sheet is folded or a cell is inverted somewhere
    pub min_jacobian: f32,
    /// The largest stretch (largest singular value of `J`)
    pub max_stretch: f32,
    /// The smallest stretch (smallest singular value of `J`)
    pub min_stretch: f32,
}

impl LatticeRegularity {
    /// Whether the map folds or inverts somewhere (`det J <= 0` at a sample): where it does, the draw is
    /// the defined result of the module doc (later triangle covers earlier; a pixel whose solve does not
    /// converge in its own cell is not drawn), not a smooth inverse
    #[must_use]
    pub fn folded(&self) -> bool {
        self.min_jacobian <= 0.0
    }
}

/// One glyph's lattice: the displacements of its nodes padded with one ghost node on every side, and
/// the mesh (see the module doc)
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphLattice {
    n: usize,
    k: usize,
    padded: Vec<[f32; 2]>,
    triangles: Vec<LatticeTriangle>,
}

impl GlyphLattice {
    /// The lattice for the nodes `positions` (`n * n`, in the order of [`rest_lattice`]) and the
    /// subdivision `k`
    ///
    /// # Errors
    /// A [`LatticeError`] when `n`, `k` or the positions are out of range
    pub fn new(n: usize, k: usize, positions: &[[f32; 2]]) -> Result<Self, LatticeError> {
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
        let d: Vec<[f32; 2]> = positions
            .iter()
            .zip(&rest)
            .map(|(p, r)| [p[0] - r[0], p[1] - r[1]])
            .collect();
        let padded = pad(n, &d);
        let mut lattice = Self {
            n,
            k,
            padded,
            triangles: Vec::new(),
        };
        lattice.triangles = lattice.mesh(positions);
        Ok(lattice)
    }

    /// Nodes per side
    #[must_use]
    pub fn n(&self) -> usize {
        self.n
    }

    /// Cell subdivision
    #[must_use]
    pub fn k(&self) -> usize {
        self.k
    }

    /// The mesh, sub-quad by sub-quad from `v = 0` up, each row from `u = 0`, the lower triangle first
    /// (the order the GPU draws them in, which decides which layer covers where the sheet overlaps)
    #[must_use]
    pub fn triangles(&self) -> &[LatticeTriangle] {
        &self.triangles
    }

    /// The node displacements from rest with their ghosts: `(n + 2) x (n + 2)` values, row `j + 1` and
    /// column `i + 1` holding node `(i, j)` (rows from `v = 0` up), the border rows and columns the ghosts;
    /// the texture the fragment shader reads (`uDisp`)
    #[must_use]
    pub fn padded_displacements(&self) -> &[[f32; 2]] {
        &self.padded
    }

    fn node(&self, pi: usize, pj: usize) -> [f32; 2] {
        self.padded[pj * (self.n + 2) + pi]
    }

    /// The displacement in cell `(ci, cj)` at the cell parameters `(s, t)` and its derivatives in `s` and
    /// `t`, summed in the order the fragment shader sums them
    fn patch(&self, ci: usize, cj: usize, s: f32, t: f32) -> ([f32; 2], [f32; 2], [f32; 2]) {
        let (ws, wt) = (catmull_rom(s), catmull_rom(t));
        let (dws, dwt) = (catmull_rom_d(s), catmull_rom_d(t));
        let mut d = [0.0f32; 2];
        let mut ds = [0.0f32; 2];
        let mut dt = [0.0f32; 2];
        for y in 0..4 {
            for x in 0..4 {
                let p = self.node(ci + x, cj + y);
                let w = ws[x] * wt[y];
                let wu = dws[x] * wt[y];
                let wv = ws[x] * dwt[y];
                for c in 0..2 {
                    d[c] += w * p[c];
                    ds[c] += wu * p[c];
                    dt[c] += wv * p[c];
                }
            }
        }
        (d, ds, dt)
    }

    fn mesh(&self, positions: &[[f32; 2]]) -> Vec<LatticeTriangle> {
        let (n, k) = (self.n, self.k);
        // the sub-vertex grid: m x m points, (m - 1) = (n - 1) k intervals per side
        let m = (n - 1) * k + 1;
        let last = (m - 1) as f32;
        let kf = k as f32;
        let mut grid = Vec::with_capacity(m * m);
        for b in 0..m {
            let cj = (b / k).min(n - 2);
            let t = (b - cj * k) as f32 / kf;
            for a in 0..m {
                let ci = (a / k).min(n - 2);
                let s = (a - ci * k) as f32 / kf;
                let r = [a as f32 / last, b as f32 / last];
                let deformed = if a % k == 0 && b % k == 0 {
                    // a node: the position as given
                    positions[(b / k) * n + a / k]
                } else {
                    let (d, _, _) = self.patch(ci, cj, s, t);
                    [r[0] + d[0], r[1] + d[1]]
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
        for b in 0..m - 1 {
            for a in 0..m - 1 {
                let cell = [(a / k) as u16, (b / k) as u16];
                let tri = |p: [usize; 3]| LatticeTriangle {
                    deformed: p.map(|i| grid[i].0),
                    rest: p.map(|i| grid[i].1),
                    outward: p.map(|i| grid[i].2),
                    cell,
                };
                let v00 = b * m + a;
                let v10 = v00 + 1;
                let v01 = v00 + m;
                let v11 = v01 + 1;
                tris.push(tri([v00, v10, v11]));
                tris.push(tri([v00, v11, v01]));
            }
        }
        tris
    }

    /// The rest point of the glyph-box point `q` for a pixel `tri` covers, as the fragment shader computes
    /// it (see the module doc): the triangle's barycentric map gives the first guess, then
    /// [`LATTICE_NEWTON_STEPS`] Newton steps solve `c + disp(q + c) = 0` with the cell's Catmull-Rom patch;
    /// `None` (the pixel is not drawn) when the triangle has no area, the residual is above
    /// [`LATTICE_RESIDUAL_PX`] pixels (`px`: one pixel in the glyph's box coordinates), or the solution lies
    /// outside the triangle's cell by more than half a sub-cell. At rest the result is `q` bit for bit
    #[must_use]
    pub fn rest_point(&self, tri: &LatticeTriangle, q: [f32; 2], px: [f32; 2]) -> Option<[f32; 2]> {
        let (r, g) = self.newton(tri, q, LATTICE_NEWTON_STEPS)?;
        if !(g[0].abs() <= LATTICE_RESIDUAL_PX * px[0] && g[1].abs() <= LATTICE_RESIDUAL_PX * px[1])
        {
            return None;
        }
        let h = 1.0 / (self.n - 1) as f32;
        let o = [f32::from(tri.cell[0]) * h, f32::from(tri.cell[1]) * h];
        let m = h / (2.0 * self.k as f32);
        let inside = |x: f32, lo: f32| x >= lo - m && x <= lo + h + m;
        (inside(r[0], o[0]) && inside(r[1], o[1])).then_some(r)
    }

    /// `steps` Newton steps of [`GlyphLattice::rest_point`] from the triangle's guess: the point and the
    /// residual `c + disp(q + c)` there; `None` when the triangle has no area
    fn newton(
        &self,
        tri: &LatticeTriangle,
        q: [f32; 2],
        steps: usize,
    ) -> Option<([f32; 2], [f32; 2])> {
        let from_d = carry(q, &tri.deformed, &tri.rest)?;
        let from_r = carry(q, &tri.rest, &tri.rest)?;
        let mut c = [from_d[0] - from_r[0], from_d[1] - from_r[1]];
        let (ci, cj) = (usize::from(tri.cell[0]), usize::from(tri.cell[1]));
        let h = 1.0 / (self.n - 1) as f32;
        let o = [ci as f32 * h, cj as f32 * h];
        for _ in 0..steps {
            let r = [q[0] + c[0], q[1] + c[1]];
            let (d, ds, dt) = self.patch(ci, cj, (r[0] - o[0]) / h, (r[1] - o[1]) / h);
            let g = [c[0] + d[0], c[1] + d[1]];
            // J = I + [d disp / du, d disp / dv], du = h ds
            let a = 1.0 + ds[0] / h;
            let b = dt[0] / h;
            let e = ds[1] / h;
            let f = 1.0 + dt[1] / h;
            let det = a * f - b * e;
            if det.abs() > 0.0 {
                c[0] -= (f * g[0] - b * g[1]) / det;
                c[1] -= (a * g[1] - e * g[0]) / det;
            }
        }
        let r = [q[0] + c[0], q[1] + c[1]];
        let (d, _, _) = self.patch(ci, cj, (r[0] - o[0]) / h, (r[1] - o[1]) / h);
        Some((r, [c[0] + d[0], c[1] + d[1]]))
    }

    /// The Jacobian of the map sampled on [`LATTICE_REGULARITY_SAMPLES`] intervals per cell side, the
    /// same for every `k` (see [`LatticeRegularity`])
    #[must_use]
    pub fn regularity(&self) -> LatticeRegularity {
        let h = 1.0 / (self.n - 1) as f32;
        let kf = LATTICE_REGULARITY_SAMPLES as f32;
        let mut out = LatticeRegularity {
            min_jacobian: f32::INFINITY,
            max_stretch: 0.0,
            min_stretch: f32::INFINITY,
        };
        for cj in 0..self.n - 1 {
            for ci in 0..self.n - 1 {
                for b in 0..=LATTICE_REGULARITY_SAMPLES {
                    for a in 0..=LATTICE_REGULARITY_SAMPLES {
                        let (_, ds, dt) = self.patch(ci, cj, a as f32 / kf, b as f32 / kf);
                        let j = [[1.0 + ds[0] / h, dt[0] / h], [ds[1] / h, 1.0 + dt[1] / h]];
                        let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
                        // singular values of a 2x2: sqrt((t +- sqrt(t^2 - 4 det^2)) / 2), t = |J|_F^2
                        let fro = j[0][0] * j[0][0]
                            + j[0][1] * j[0][1]
                            + j[1][0] * j[1][0]
                            + j[1][1] * j[1][1];
                        let disc = fro * fro - 4.0 * det * det;
                        let root = alice_det_math::ops::sqrt(if disc > 0.0 { disc } else { 0.0 });
                        let hi = alice_det_math::ops::sqrt((fro + root) * 0.5);
                        let low = fro - root;
                        let lo = alice_det_math::ops::sqrt(if low > 0.0 { low * 0.5 } else { 0.0 });
                        if det < out.min_jacobian {
                            out.min_jacobian = det;
                        }
                        if hi > out.max_stretch {
                            out.max_stretch = hi;
                        }
                        if lo < out.min_stretch {
                            out.min_stretch = lo;
                        }
                    }
                }
            }
        }
        out
    }
}

fn cross2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// The shader's `carry`: the point with the barycentric coordinates of `q` in `d`, taken in `r`; `None`
/// when `d` has no area
fn carry(q: [f32; 2], d: &[[f32; 2]; 3], r: &[[f32; 2]; 3]) -> Option<[f32; 2]> {
    let [a, b, c] = *d;
    let sub = |x: [f32; 2], y: [f32; 2]| [x[0] - y[0], x[1] - y[1]];
    let den = cross2(sub(b, a), sub(c, a));
    // no area, or not a number (both refuse the pixel, as the shader's `abs(den) > 0.0` does)
    if den.abs() <= 0.0 || den.is_nan() {
        return None;
    }
    let l1 = cross2(sub(q, a), sub(c, a)) / den;
    let l2 = cross2(sub(b, a), sub(q, a)) / den;
    let l0 = 1.0 - l1 - l2;
    Some([
        r[0][0] * l0 + r[1][0] * l1 + r[2][0] * l2,
        r[0][1] * l0 + r[1][1] * l1 + r[2][1] * l2,
    ])
}

/// The displacements `d` (`n x n`) with one ghost node on every side, `(n + 2)^2` values
fn pad(n: usize, d: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let w = n + 2;
    let mut p = vec![[0.0f32; 2]; w * w];
    for j in 0..n {
        for i in 0..n {
            p[(j + 1) * w + i + 1] = d[j * n + i];
        }
    }
    let third = n >= 3;
    // ghost columns of the node rows, then ghost rows of every column (the corners from the ghost columns)
    for j in 1..=n {
        let row = |i: usize| p[j * w + i];
        let (l, r) = (
            ghost(row(1), row(2), third.then(|| row(3))),
            ghost(row(n), row(n - 1), third.then(|| row(n - 2))),
        );
        p[j * w] = l;
        p[j * w + n + 1] = r;
    }
    for i in 0..w {
        let col = |j: usize| p[j * w + i];
        let (b, t) = (
            ghost(col(1), col(2), third.then(|| col(3))),
            ghost(col(n), col(n - 1), third.then(|| col(n - 2))),
        );
        p[i] = b;
        p[(n + 1) * w + i] = t;
    }
    p
}

/// The triangles of one glyph's lattice mesh ([`GlyphLattice::new`] then [`GlyphLattice::triangles`])
///
/// # Errors
/// A [`LatticeError`] when `n`, `k` or the positions are out of range
pub fn lattice_triangles(
    n: usize,
    k: usize,
    positions: &[[f32; 2]],
) -> Result<Vec<LatticeTriangle>, LatticeError> {
    Ok(GlyphLattice::new(n, k, positions)?.triangles)
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
            out.push(f32::from(t.cell[0]));
            out.push(f32::from(t.cell[1]));
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
            cell: [3, 5],
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
            assert_eq!((v[2], v[3]), (3.0, 5.0), "cell");
            assert_eq!(&v[4..], &body[..]);
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
            let end = "    fragColor = mix(uBgColor, uFgColor, coverage);\n";
            let to = src.find(end).expect("evaluation end") + end.len();
            src[at..to].into()
        };
        // the most ink wins where layers overlap (see the module doc)
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains("gl_FragDepth = 1.0 - coverage;"));
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
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains("vec2 rest = q + c;"));
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains(&alloc::format!(
            "const int NEWTON_STEPS = {LATTICE_NEWTON_STEPS};"
        )));
        assert_eq!(LATTICE_RESIDUAL_PX, 0.015_625);
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains("const float RESIDUAL_PX = 0.015625;"));
        // the loop's trip count is the literal, not data
        assert!(LATTICE_OUTLINE_FRAG_GLSL.contains("for (int it = 0; it < NEWTON_STEPS; it++)"));
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
        assert!(LATTICE_OUTLINE_VERT_GLSL.contains("in vec4 aSlot;"));
        for attr in ["aSlot", "aD01", "aD2R0", "aR12", "aO01", "aO2"] {
            assert!(
                LATTICE_OUTLINE_VERT_GLSL.contains(&alloc::format!(" {attr};")),
                "{attr}"
            );
        }
    }

    const PX: [f32; 2] = [0.01, 0.01];

    fn lattice_of(n: usize, k: usize, f: impl Fn([f32; 2]) -> [f32; 2]) -> GlyphLattice {
        let pos: Vec<[f32; 2]> = rest_lattice(n).unwrap().into_iter().map(f).collect();
        GlyphLattice::new(n, k, &pos).unwrap()
    }

    /// points inside the deformed triangle (fixed barycentric mixes, the vertices included)
    fn points_in(t: &LatticeTriangle) -> Vec<[f32; 2]> {
        let mix = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            [0.2, 0.3, 0.5],
            [0.6, 0.2, 0.2],
            [0.1, 0.1, 0.8],
            [0.5, 0.5, 0.0],
        ];
        mix.iter()
            .map(|l: &[f32; 3]| {
                [0, 1].map(|c| {
                    l[0] * t.deformed[0][c] + l[1] * t.deformed[1][c] + l[2] * t.deformed[2][c]
                })
            })
            .collect()
    }

    // oracle: at rest every displacement is 0, so the solve leaves c = 0 and the rest point is q bit for bit
    #[test]
    fn at_rest_the_rest_point_is_the_pixel_point_bit_for_bit() {
        let mut compared = 0;
        for (n, k) in [(2, 1), (4, 3), (5, 2)] {
            let l = lattice_of(n, k, |p| p);
            for t in l.triangles() {
                for q in points_in(t) {
                    let r = l.rest_point(t, q, PX).expect("drawn at rest");
                    assert_eq!(
                        r.map(f32::to_bits),
                        q.map(f32::to_bits),
                        "n {n} k {k} q {q:?}"
                    );
                    compared += 1;
                }
            }
        }
        assert!(compared > 500, "{compared}");
    }

    fn affine(p: [f32; 2]) -> [f32; 2] {
        [
            0.8 * p[0] - 0.3 * p[1] + 0.12,
            0.25 * p[0] + 0.9 * p[1] + 0.04,
        ]
    }

    /// the closed-form inverse of `affine`
    fn affine_inv(q: [f32; 2]) -> [f64; 2] {
        let (a, b, c, d) = (0.8_f64, -0.3, 0.25, 0.9);
        let det = a * d - b * c;
        let x = f64::from(q[0]) - 0.12;
        let y = f64::from(q[1]) - 0.04;
        [(d * x - b * y) / det, (-c * x + a * y) / det]
    }

    // oracle (closed form): an affine move makes the equation linear, so one Newton step from any guess
    // solves it; the result is the inverse map
    #[test]
    fn an_affine_move_is_solved_in_one_newton_step() {
        for (n, k) in [(2, 1), (4, 2)] {
            let l = lattice_of(n, k, affine);
            for t in l.triangles() {
                for q in points_in(t) {
                    let want = affine_inv(q);
                    let (r, _) = l.newton(t, q, 1).unwrap();
                    for c in 0..2 {
                        assert!(
                            (f64::from(r[c]) - want[c]).abs() < 2e-6,
                            "n {n} k {k}: {r:?} vs {want:?}"
                        );
                    }
                    assert!(l.rest_point(t, q, PX).is_some());
                }
            }
        }
    }

    fn quad_bend(p: [f32; 2]) -> [f32; 2] {
        let s = p[1] - 0.5;
        [p[0] + 0.5 * s * s, p[1]]
    }

    // oracle (closed form): the quadratic bend is reproduced by the patches, so its inverse is
    // (x - 0.5 (y - 0.5)^2, y); every triangle that accepts q (not only the one that holds it) returns it,
    // so the value does not depend on which triangle a rasterizer gave the pixel to
    #[test]
    fn a_bend_is_inverted_exactly_by_any_triangle_that_accepts_the_pixel() {
        for (n, k) in [(4, 1), (4, 3), (6, 2)] {
            let l = lattice_of(n, k, quad_bend);
            let mut accepted = 0;
            for t in l.triangles() {
                for q in points_in(t) {
                    let y = f64::from(q[1]);
                    let want = [f64::from(q[0]) - 0.5 * (y - 0.5) * (y - 0.5), y];
                    for other in l.triangles() {
                        if let Some(r) = l.rest_point(other, q, PX) {
                            for c in 0..2 {
                                assert!(
                                    (f64::from(r[c]) - want[c]).abs() < 2e-6,
                                    "n {n} k {k} q {q:?}: {r:?} vs {want:?}"
                                );
                            }
                            accepted += 1;
                        }
                    }
                    assert!(
                        l.rest_point(t, q, PX).is_some(),
                        "n {n} k {k} q {q:?}: its own triangle refused"
                    );
                }
            }
            assert!(accepted > 100, "{accepted}");
        }
    }

    fn cubic(p: [f32; 2]) -> [f32; 2] {
        let (u, v) = (p[0] - 0.5, p[1] - 0.5);
        [
            p[0] + 0.4 * v * v * v + 0.2 * u * u * u,
            p[1] + 0.1 * p[0] * p[0] * p[0],
        ]
    }

    // condition across a cell boundary: under a cubic (not reproduced, so the two cells' patches are
    // different polynomials), at a point of a shared cell edge both patches have the same value and slope
    // (Catmull-Rom is C1), so the pixel there, and a pixel a distance e off it, is solved by a triangle of
    // either cell to the same rest point within 1e-6 for e = 0 and within 4 e^2 for e > 0 (the patches
    // differ from second order). A rasterizer only gives a pixel to the other cell's triangle within its
    // edge precision of the edge (about 1e-5 of the glyph box at 100 px per em)
    #[test]
    fn the_two_cells_at_a_shared_edge_give_the_same_rest_point() {
        let (n, k) = (4, 2);
        let l = lattice_of(n, k, cubic);
        let h = 1.0 / (n - 1) as f32;
        let mut compared = 0;
        let mut table = Vec::new();
        for e in [0.0_f32, 1e-5, 1e-4, 1e-3] {
            let mut worst = 0.0f32;
            for t in l.triangles() {
                for v in 0..3 {
                    let r = t.rest[v];
                    // a vertex on a vertical cell edge inside the lattice
                    let edge = (r[0] / h).round();
                    if (r[0] - edge * h).abs() > 1e-6 || edge < 1.0 || edge > (n - 2) as f32 {
                        continue;
                    }
                    let q = [t.deformed[v][0] + e, t.deformed[v][1]];
                    // the triangles touching this vertex, on both sides of the edge
                    let around: Vec<&LatticeTriangle> = l
                        .triangles()
                        .iter()
                        .filter(|o| o.rest.contains(&r))
                        .collect();
                    let sols: Vec<[f32; 2]> = around
                        .iter()
                        .filter_map(|o| l.rest_point(o, q, PX))
                        .collect();
                    let cells: Vec<u16> = around.iter().map(|o| o.cell[0]).collect();
                    assert!(
                        cells.iter().any(|&c| f32::from(c) < edge)
                            && cells.iter().any(|&c| f32::from(c) >= edge)
                    );
                    for a in &sols {
                        for b in &sols {
                            worst = worst.max((a[0] - b[0]).abs().max((a[1] - b[1]).abs()));
                            compared += 1;
                        }
                    }
                }
            }
            #[cfg(feature = "std")]
            std::eprintln!("cell boundary: offset {e:e}, largest difference {worst:e}");
            table.push((e, worst));
        }
        assert!(compared > 100, "pairs {compared}");
        for (e, worst) in table {
            assert!(
                worst <= 1e-6 + 4.0 * e * e,
                "offset {e}: difference {worst}"
            );
        }
    }

    /// the Newton steps a point needs to bring its residual to 1e-6 of the glyph box
    fn steps_needed(l: &GlyphLattice, t: &LatticeTriangle, q: [f32; 2]) -> usize {
        (0..=12)
            .find(|&s| {
                l.newton(t, q, s)
                    .is_some_and(|(_, g)| g[0].abs() <= 1e-6 && g[1].abs() <= 1e-6)
            })
            .unwrap_or(99)
    }

    // the measured table of steps against strain; every regular case (det J > 0) converges within the
    // literal step count
    #[test]
    fn newton_converges_within_the_step_count_for_regular_lattices() {
        type Case = (&'static str, fn([f32; 2]) -> [f32; 2]);
        let cases: [Case; 9] = [
            ("rest", |p| p),
            ("stretch 1.5", |p| [1.5 * p[0], p[1]]),
            ("stretch 4", |p| [4.0 * p[0], p[1]]),
            ("squeeze 0.25", |p| [0.25 * p[0], p[1]]),
            ("shear 1", |p| [p[0] + p[1], p[1]]),
            ("bend 0.5", quad_bend),
            ("bend 1.5", |p| {
                [p[0] + 1.5 * (p[1] - 0.5) * (p[1] - 0.5), p[1]]
            }),
            ("cubic", cubic),
            ("bulge", |p| {
                let (u, v) = (p[0] - 0.5, p[1] - 0.5);
                let s = 1.0 + 0.6 * (0.25 - u * u - v * v);
                [0.5 + u * s, 0.5 + v * s]
            }),
        ];
        for (name, f) in cases {
            for k in [1, 4] {
                let l = lattice_of(5, k, f);
                let reg = l.regularity();
                let mut worst = 0;
                for t in l.triangles() {
                    for q in points_in(t) {
                        worst = worst.max(steps_needed(&l, t, q));
                    }
                }
                #[cfg(feature = "std")]
                std::eprintln!(
                    "steps | {name} k {k} | min det {:.3} | stretch {:.3}..{:.3} | worst steps {worst}",
                    reg.min_jacobian, reg.min_stretch, reg.max_stretch
                );
                assert!(!reg.folded(), "{name}: regular");
                assert!(worst <= LATTICE_NEWTON_STEPS, "{name} k {k}: {worst} steps");
            }
        }
    }

    // a folded sheet (the top row put back below the middle row): the regularity says so, and every pixel
    // the solve draws is a preimage in its own cell; where the two layers overlap, a triangle of each layer
    // draws its own layer (the later one covers the earlier on the GPU, by draw order)
    #[test]
    fn a_folded_sheet_draws_each_layer_from_its_own_cells() {
        let n = 3;
        let pos: Vec<[f32; 2]> = rest_lattice(n)
            .unwrap()
            .into_iter()
            .map(|[u, v]| {
                [
                    u,
                    if v > 0.75 {
                        0.2
                    } else if v > 0.25 {
                        0.6
                    } else {
                        0.0
                    },
                ]
            })
            .collect();
        let l = GlyphLattice::new(n, 4, &pos).unwrap();
        assert!(l.regularity().folded(), "{:?}", l.regularity());
        let mut two_layers = 0;
        for t in l.triangles() {
            for q in points_in(t) {
                let mut rows = Vec::new();
                for o in l.triangles() {
                    if let Some(r) = l.rest_point(o, q, PX) {
                        // within its cell (+- half a sub-cell)
                        let h = 0.5;
                        let lo = f32::from(o.cell[1]) * h - h / 8.0;
                        assert!(
                            r[1] >= lo && r[1] <= lo + h + h / 4.0,
                            "{r:?} outside cell {:?}",
                            o.cell
                        );
                        rows.push(o.cell[1]);
                    }
                }
                if rows.contains(&0) && rows.contains(&1) {
                    two_layers += 1;
                }
            }
        }
        assert!(two_layers > 0, "no point where both layers draw");
    }

    // oracle (closed form): a mirrored lattice (x -> 1 - x) is inverted everywhere (det J = -1): it is
    // reported as folded, and the solve draws the mirror image, (1 - x, y)
    #[test]
    fn a_mirrored_lattice_is_reported_and_draws_the_mirror_image() {
        let l = lattice_of(4, 2, |p| [1.0 - p[0], p[1]]);
        let reg = l.regularity();
        assert!(reg.folded());
        assert!((reg.min_jacobian + 1.0).abs() < 1e-5, "{reg:?}");
        for t in l.triangles() {
            for q in points_in(t) {
                let r = l.rest_point(t, q, PX).expect("drawn");
                assert!(
                    (r[0] - (1.0 - q[0])).abs() < 2e-6 && (r[1] - q[1]).abs() < 2e-6,
                    "{q:?} {r:?}"
                );
            }
        }
    }

    // oracle: the identity has J = I everywhere; an affine map has J = A everywhere
    #[test]
    fn regularity_is_the_jacobian_of_the_map() {
        let rest = lattice_of(4, 3, |p| p).regularity();
        assert_eq!(
            (rest.min_jacobian, rest.min_stretch, rest.max_stretch),
            (1.0, 1.0, 1.0)
        );
        let a = lattice_of(4, 3, affine).regularity();
        assert!(
            (a.min_jacobian - (0.8 * 0.9 + 0.3 * 0.25)).abs() < 1e-5,
            "{a:?}"
        );
        assert!(!a.folded());
    }

    // a triangle with no area draws nothing; a solve that is not given the steps it needs leaves a residual
    // above the threshold (the check is what refuses an unconverged pixel)
    #[test]
    fn a_degenerate_triangle_or_an_unconverged_solve_is_not_drawn() {
        let l = lattice_of(3, 1, |p| p);
        let mut t = l.triangles()[0];
        t.deformed = [[0.2, 0.2]; 3];
        assert_eq!(l.rest_point(&t, [0.2, 0.2], PX), None);
        let bent = lattice_of(4, 1, |p| [p[0] + 1.5 * (p[1] - 0.5) * (p[1] - 0.5), p[1]]);
        let mut over = 0;
        for t in bent.triangles() {
            for q in points_in(t) {
                let (_, g) = bent.newton(t, q, 0).unwrap();
                if g[0].abs() > LATTICE_RESIDUAL_PX * PX[0]
                    || g[1].abs() > LATTICE_RESIDUAL_PX * PX[1]
                {
                    over += 1;
                }
            }
        }
        assert!(
            over > 0,
            "the first guess alone is within the threshold everywhere"
        );
    }

    // the residual threshold is in pixels: with a pixel of 1e-12 of the glyph box nothing but an exact
    // solve is drawn, so a bent lattice refuses a pixel its f32 solve cannot hit exactly, and the rest
    // lattice (residual exactly 0) still draws every pixel
    #[test]
    fn the_residual_threshold_is_measured_in_pixels() {
        let tiny = [1e-12_f32, 1e-12];
        let rest = lattice_of(4, 2, |p| p);
        let bent = lattice_of(4, 2, cubic);
        let mut refused = 0;
        for (tr, tb) in rest.triangles().iter().zip(bent.triangles()) {
            for q in points_in(tr) {
                assert!(rest.rest_point(tr, q, tiny).is_some(), "rest refused {q:?}");
            }
            for q in points_in(tb) {
                if bent.rest_point(tb, q, tiny).is_none() {
                    refused += 1;
                }
                assert!(
                    bent.rest_point(tb, q, PX).is_some(),
                    "a pixel of 0.01 refused {q:?}"
                );
            }
        }
        assert!(refused > 0, "no pixel refused at a threshold of 1e-12");
    }

    // regularity is a property of the lattice, not of the subdivision: a recorded soft-body state (a sheet
    // dragged and released until it folded, 4 x 4 nodes) is reported folded for every k, and with the same
    // numbers (k = 1 sampled at the sub-vertices alone saw det J 0.04 where k = 4 saw -1.17)
    #[test]
    fn regularity_does_not_depend_on_the_subdivision() {
        let v: [f32; 32] = [
            0.7853771,
            0.39657566,
            0.99272215,
            0.6486059,
            0.926314,
            0.3099659,
            1.2446197,
            0.32582003,
            0.5301565,
            0.5996639,
            0.72532004,
            0.8606071,
            0.8194969,
            0.5484488,
            1.151309,
            0.635547,
            0.15493701,
            0.6861848,
            0.45086133,
            0.8175874,
            0.7414651,
            0.8828341,
            1.0681843,
            0.9606719,
            0.020235322,
            0.99092907,
            0.3364653,
            1.1229175,
            0.6639094,
            1.2036088,
            0.9873873,
            1.2840649,
        ];
        let pos: Vec<[f32; 2]> = v.chunks_exact(2).map(|p| [p[0], p[1]]).collect();
        let regs: Vec<LatticeRegularity> = [1, 2, 4, 8]
            .iter()
            .map(|&k| GlyphLattice::new(4, k, &pos).unwrap().regularity())
            .collect();
        assert!(regs[0].folded(), "{:?}", regs[0]);
        for r in &regs[1..] {
            assert_eq!(r, &regs[0]);
        }
    }
}
