//! `outline::rasterize` refuses input it cannot rasterise correctly or in bounded time, instead
//! of returning a plausible field.

use alice_font::outline::{
    rasterize, OutlineGlyph, RasterizeError, RASTER_MAX_CONTOURS, RASTER_MAX_PIECES,
    RASTER_MAX_POINTS,
};

fn glyph(contours: Vec<Vec<[f32; 2]>>, advance: f32) -> OutlineGlyph {
    OutlineGlyph {
        codepoint: 'x',
        contours,
        advance,
    }
}

/// The error, or a failure naming what was accepted
fn refused(g: &OutlineGlyph) -> RasterizeError {
    match rasterize(g) {
        Err(e) => e,
        Ok(_) => panic!("the outline was rasterised"),
    }
}

fn square() -> Vec<[f32; 2]> {
    vec![[0.25, 0.0], [0.75, 0.0], [0.75, 0.5], [0.25, 0.5]]
}

#[test]
fn a_nan_coordinate_is_refused_with_its_place() {
    let mut c = square();
    c[2][1] = f32::NAN;
    let err = refused(&glyph(vec![square(), c], 1.0));
    assert_eq!(
        err,
        RasterizeError::NonFinite {
            contour: 1,
            point: 2
        }
    );
}

#[test]
fn an_infinite_coordinate_is_refused() {
    let mut c = square();
    c[0][0] = f32::INFINITY;
    assert_eq!(
        refused(&glyph(vec![c], 1.0)),
        RasterizeError::NonFinite {
            contour: 0,
            point: 0
        }
    );
    let mut c = square();
    c[3][1] = f32::NEG_INFINITY;
    assert_eq!(
        refused(&glyph(vec![c], 1.0)),
        RasterizeError::NonFinite {
            contour: 0,
            point: 3
        }
    );
}

#[test]
fn a_non_finite_advance_is_refused() {
    assert_eq!(
        refused(&glyph(vec![square()], f32::NAN)),
        RasterizeError::NonFiniteAdvance
    );
    assert_eq!(
        refused(&glyph(vec![square()], f32::INFINITY)),
        RasterizeError::NonFiniteAdvance
    );
}

#[test]
fn points_up_to_the_cap_are_accepted_and_one_more_is_refused() {
    // one contour of RASTER_MAX_POINTS points on a circle
    let ring = |n: usize| -> Vec<[f32; 2]> {
        (0..n)
            .map(|i| {
                let a = i as f32 / n as f32 * core::f32::consts::TAU;
                [0.5 + 0.3 * a.cos(), 0.35 + 0.3 * a.sin()]
            })
            .collect()
    };
    assert!(rasterize(&glyph(vec![ring(RASTER_MAX_POINTS)], 1.0)).is_ok());
    assert_eq!(
        refused(&glyph(vec![ring(RASTER_MAX_POINTS + 1)], 1.0)),
        RasterizeError::TooManyPoints {
            points: RASTER_MAX_POINTS + 1,
            max: RASTER_MAX_POINTS
        }
    );
}

#[test]
fn contours_up_to_the_cap_are_accepted_and_one_more_is_refused() {
    let many = |n: usize| -> Vec<Vec<[f32; 2]>> {
        (0..n)
            .map(|i| {
                let x = (i % 16) as f32 / 16.0;
                let y = (i / 16) as f32 / 16.0 * 0.6;
                vec![[x, y], [x + 0.03, y], [x + 0.03, y + 0.03]]
            })
            .collect()
    };
    assert!(rasterize(&glyph(many(RASTER_MAX_CONTOURS), 1.0)).is_ok());
    assert_eq!(
        refused(&glyph(many(RASTER_MAX_CONTOURS + 1), 1.0)),
        RasterizeError::TooManyContours {
            contours: RASTER_MAX_CONTOURS + 1,
            max: RASTER_MAX_CONTOURS
        }
    );
}

/// Two zigzags across the tile, `n` points each: every edge of one crosses every edge of the other
fn crossing_zigzags(n: usize) -> Vec<Vec<[f32; 2]>> {
    let zig = |horizontal: bool| -> Vec<[f32; 2]> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                let w = if i % 2 == 0 { 0.0 } else { 1.0 };
                if horizontal {
                    [t, -0.2 + 1.1 * w]
                } else {
                    [w, -0.2 + 1.1 * t]
                }
            })
            .collect()
    };
    vec![zig(true), zig(false)]
}

#[test]
fn edges_that_cross_too_often_are_refused() {
    // 2 × 128 points stay under the point cap, but their edges cut into far more pieces
    let g = glyph(crossing_zigzags(128), 1.0);
    assert!(g.total_points() <= RASTER_MAX_POINTS);
    assert_eq!(
        refused(&g),
        RasterizeError::TooManyPieces {
            max: RASTER_MAX_PIECES
        }
    );
    // a few crossings are fine
    assert!(rasterize(&glyph(crossing_zigzags(16), 1.0)).is_ok());
}

#[test]
fn the_error_says_what_was_refused() {
    let msg = RasterizeError::NonFinite {
        contour: 1,
        point: 2,
    }
    .to_string();
    assert!(
        msg.contains("contour 1") && msg.contains("point 2"),
        "{msg}"
    );
}
