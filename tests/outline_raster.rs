//! `outline::rasterize` against closed forms and an independent brute force.
//!
//! The expected values are computed here in `f64` with `std` `sqrt`, from the shape itself (a box
//! distance, or the minimum over the edges of the union polygon written out by hand), not from
//! the crate's code.

use alice_font::glyph::{GlyphSdf, GLYPH_SDF_SIZE};
use alice_font::outline::{
    rasterize, OutlineGlyph, TILE_EM_BOTTOM, TILE_EM_LEFT, TILE_EM_RIGHT, TILE_EM_TOP,
};

const N: usize = GLYPH_SDF_SIZE;
const TOL: f64 = 1e-6;

fn glyph(contours: Vec<Vec<[f32; 2]>>) -> OutlineGlyph {
    OutlineGlyph {
        codepoint: 'x',
        contours,
        advance: 0.75,
    }
}

/// Counter-clockwise rectangle
fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<[f32; 2]> {
    vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
}

fn reversed(mut c: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    c.reverse();
    c
}

/// The em position of sample `(px, py)`
fn sample(px: usize, py: usize) -> (f64, f64) {
    let l = f64::from(TILE_EM_LEFT);
    let b = f64::from(TILE_EM_BOTTOM);
    let w = f64::from(TILE_EM_RIGHT) - l;
    let h = f64::from(TILE_EM_TOP) - b;
    (
        l + (px as f64 + 0.5) / N as f64 * w,
        b + (py as f64 + 0.5) / N as f64 * h,
    )
}

/// Signed distance to an axis-aligned box, closed form
fn box_sdf(x: f64, y: f64, r: [f32; 4]) -> f64 {
    let [x0, y0, x1, y1] = r.map(f64::from);
    let qx = (x - (x0 + x1) / 2.0).abs() - (x1 - x0) / 2.0;
    let qy = (y - (y0 + y1) / 2.0).abs() - (y1 - y0) / 2.0;
    let outside = qx.max(0.0).hypot(qy.max(0.0));
    outside + qx.max(qy).min(0.0)
}

fn seg_dist(x: f64, y: f64, a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let t = (((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    (x - (a[0] + dx * t)).hypot(y - (a[1] + dy * t))
}

/// Signed distance to one simple polygon: minimum over its edges, sign by even-odd
fn polygon_sdf(x: f64, y: f64, poly: &[[f32; 2]]) -> f64 {
    let p: Vec<[f64; 2]> = poly
        .iter()
        .map(|v| [f64::from(v[0]), f64::from(v[1])])
        .collect();
    let mut d = f64::INFINITY;
    let mut inside = false;
    for i in 0..p.len() {
        let (a, b) = (p[i], p[(i + 1) % p.len()]);
        d = d.min(seg_dist(x, y, a, b));
        if (a[1] > y) != (b[1] > y) && x < a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]) {
            inside = !inside;
        }
    }
    if inside {
        -d
    } else {
        d
    }
}

fn assert_matches(sdf: &GlyphSdf, expected: impl Fn(f64, f64) -> f64) {
    let mut worst = 0.0_f64;
    for py in 0..N {
        for px in 0..N {
            let (x, y) = sample(px, py);
            let want = expected(x, y);
            let got = f64::from(sdf.data[py * N + px]);
            assert_eq!(
                got < 0.0,
                want < 0.0,
                "sign at ({px}, {py}): got {got}, want {want}"
            );
            worst = worst.max((got - want).abs());
        }
    }
    assert!(worst <= TOL, "largest error {worst} em");
}

const BOX: [f32; 4] = [0.25, 0.0625, 0.75, 0.6875];

#[test]
fn a_box_is_its_closed_form() {
    let [x0, y0, x1, y1] = BOX;
    let sdf = rasterize(&glyph(vec![rect(x0, y0, x1, y1)]));
    assert_matches(&sdf, |x, y| box_sdf(x, y, BOX));
}

#[test]
fn the_direction_of_a_contour_does_not_matter() {
    let [x0, y0, x1, y1] = BOX;
    let ccw = rasterize(&glyph(vec![rect(x0, y0, x1, y1)]));
    let cw = rasterize(&glyph(vec![reversed(rect(x0, y0, x1, y1))]));
    assert_eq!(ccw.data, cw.data);
}

#[test]
fn an_explicit_closing_point_changes_nothing() {
    let [x0, y0, x1, y1] = BOX;
    let mut closed = rect(x0, y0, x1, y1);
    closed.push(closed[0]);
    assert_eq!(
        rasterize(&glyph(vec![rect(x0, y0, x1, y1)])).data,
        rasterize(&glyph(vec![closed])).data
    );
}

#[test]
fn a_reversed_inner_contour_is_a_hole() {
    let outer = [0.125_f32, 0.0, 0.875, 0.75];
    let inner = [0.375_f32, 0.25, 0.625, 0.5];
    let sdf = rasterize(&glyph(vec![
        rect(outer[0], outer[1], outer[2], outer[3]),
        reversed(rect(inner[0], inner[1], inner[2], inner[3])),
    ]));
    // outside the outer box: its distance; in the hole: the distance to the inner box; in the
    // ring: minus the nearer of the two boundaries
    assert_matches(&sdf, |x, y| box_sdf(x, y, outer).max(-box_sdf(x, y, inner)));
}

/// A plus sign drawn as two overlapping bars, and the same plus as one 12-gon
const HBAR: [f32; 4] = [0.125, 0.25, 0.875, 0.4375];
const VBAR: [f32; 4] = [0.40625, -0.0625, 0.59375, 0.75];
const PLUS: [[f32; 2]; 12] = [
    [0.40625, -0.0625],
    [0.59375, -0.0625],
    [0.59375, 0.25],
    [0.875, 0.25],
    [0.875, 0.4375],
    [0.59375, 0.4375],
    [0.59375, 0.75],
    [0.40625, 0.75],
    [0.40625, 0.4375],
    [0.125, 0.4375],
    [0.125, 0.25],
    [0.40625, 0.25],
];

fn plus_of_two_bars() -> OutlineGlyph {
    glyph(vec![
        rect(HBAR[0], HBAR[1], HBAR[2], HBAR[3]),
        rect(VBAR[0], VBAR[1], VBAR[2], VBAR[3]),
    ])
}

#[test]
fn overlapping_contours_measure_to_the_union_boundary() {
    let sdf = rasterize(&plus_of_two_bars());
    assert_matches(&sdf, |x, y| polygon_sdf(x, y, &PLUS));
}

#[test]
fn the_edges_inside_an_overlap_take_no_part() {
    // the same samples measured to every contour edge would differ: the oracle above has teeth
    let sdf = rasterize(&plus_of_two_bars());
    let mut differs = 0;
    for py in 0..N {
        for px in 0..N {
            let (x, y) = sample(px, py);
            let all_edges = box_sdf(x, y, HBAR).abs().min(box_sdf(x, y, VBAR).abs());
            if (f64::from(sdf.data[py * N + px]).abs() - all_edges).abs() > 1e-3 {
                differs += 1;
            }
        }
    }
    assert!(differs > 0);
}

#[test]
fn the_overlap_itself_is_filled() {
    // winding 2 there: inside under the non-zero rule (even-odd would make it a hole)
    let sdf = rasterize(&plus_of_two_bars());
    let (px, py) = (
        N / 2,
        ((0.34 - f64::from(TILE_EM_BOTTOM)) / 1.1 * N as f64) as usize,
    );
    let (x, y) = sample(px, py);
    assert!(f64::from(HBAR[1]) < y && y < f64::from(HBAR[3]) && x > 0.40625 && x < 0.59375);
    assert!(sdf.data[py * N + px] < 0.0);
}

/// A tall box with a bar to its right, as one 8-gon
const TEE: [[f32; 2]; 8] = [
    [0.2, 0.1],
    [0.5, 0.1],
    [0.5, 0.2],
    [0.8, 0.2],
    [0.8, 0.4],
    [0.5, 0.4],
    [0.5, 0.6],
    [0.2, 0.6],
];

#[test]
fn an_edge_crossed_near_its_end_is_cut_there() {
    // the bar starts just inside the box: its long edges cross the box's right edge close to
    // their own ends
    let sdf = rasterize(&glyph(vec![
        rect(0.2, 0.1, 0.5, 0.6),
        rect(0.49, 0.2, 0.8, 0.4),
    ]));
    assert_matches(&sdf, |x, y| polygon_sdf(x, y, &TEE));
}

#[test]
fn abutting_contours_share_no_boundary() {
    // the bar's left edge lies on the box's right edge
    let sdf = rasterize(&glyph(vec![
        rect(0.2, 0.1, 0.5, 0.6),
        rect(0.5, 0.2, 0.8, 0.4),
    ]));
    assert_matches(&sdf, |x, y| polygon_sdf(x, y, &TEE));
}

#[test]
fn a_contour_drawn_twice_is_the_contour() {
    let [x0, y0, x1, y1] = BOX;
    let once = rasterize(&glyph(vec![rect(x0, y0, x1, y1)]));
    let twice = rasterize(&glyph(vec![rect(x0, y0, x1, y1), rect(x0, y0, x1, y1)]));
    assert_eq!(once.data, twice.data);
}

#[test]
fn a_contour_and_its_reverse_cancel() {
    let [x0, y0, x1, y1] = BOX;
    let sdf = rasterize(&glyph(vec![
        rect(x0, y0, x1, y1),
        reversed(rect(x0, y0, x1, y1)),
    ]));
    assert!(sdf.data.iter().all(|&v| v == 1.0));
}

#[test]
fn an_empty_outline_is_the_empty_glyph() {
    let sdf = rasterize(&glyph(vec![]));
    assert_eq!(sdf.data, GlyphSdf::empty().data);
}

#[test]
fn the_glyph_carries_its_advance_and_the_tile_frame() {
    let sdf = rasterize(&plus_of_two_bars());
    assert_eq!(sdf.advance, 0.75);
    assert_eq!(
        (sdf.bbox_min.x, sdf.bbox_min.y),
        (TILE_EM_LEFT, TILE_EM_BOTTOM)
    );
    assert_eq!(
        (sdf.bbox_max.x, sdf.bbox_max.y),
        (TILE_EM_RIGHT, TILE_EM_TOP)
    );
}

/// SHA-256 is not a dependency here; a fixed 64-bit FNV-1a over the bits is enough to notice any
/// change of any sample between platforms
fn digest(sdf: &GlyphSdf) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in sdf.data {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// A star with slanted edges (a simple polygon)
const STAR: [[f32; 2]; 10] = [
    [0.5, 0.78],
    [0.58, 0.47],
    [0.9, 0.46],
    [0.63, 0.29],
    [0.74, -0.03],
    [0.5, 0.17],
    [0.26, -0.03],
    [0.37, 0.29],
    [0.1, 0.46],
    [0.42, 0.47],
];

#[test]
fn a_star_is_its_brute_force() {
    let sdf = rasterize(&glyph(vec![STAR.to_vec()]));
    assert_matches(&sdf, |x, y| polygon_sdf(x, y, &STAR));
}

#[test]
fn the_output_bits_are_the_same_on_every_platform() {
    // the star overlapping a box, so rounding in every step shows
    let sdf = rasterize(&glyph(vec![
        STAR.to_vec(),
        rect(BOX[0], BOX[1], BOX[2], BOX[3]),
    ]));
    assert_eq!(digest(&sdf), STAR_DIGEST, "{:#018x}", digest(&sdf));
}

const STAR_DIGEST: u64 = 0xa09e_8903_c20e_2b17;
