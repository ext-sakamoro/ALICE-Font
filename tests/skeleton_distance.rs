//! The skeleton as a distance (`skeleton_shader::skeleton_distance` / `skeleton_stroke`, the Rust twin of
//! `shaders/skeleton_distance.glsl`): held to closed forms on chunks built by hand from the std140 layout, to the
//! packer's own chunks for a real glyph, and to the shader text it is the twin of. The agreement with the GPU is
//! measured by `examples/skeleton_parity.rs` and its browser page, not here (no GPU in a unit test).
//!
//! Author: Moroya Sakamoto

use alice_font::font_dsl::dsl_lookup;
use alice_font::skeleton_shader::{
    pack_skeleton_string, skeleton_distance, skeleton_stroke, skeleton_stroke_count,
    MULTI_SKELETON_FRAG_GLSL, MULTI_SKELETON_MAX_GLYPHS, MULTI_SKELETON_MAX_STROKES_PER_GLYPH,
    MULTI_SKELETON_STROKES_AREA_SIZE, MULTI_SKELETON_UBO_SIZE, SKELETON_DISTANCE_GLSL,
    VEC4_PER_STROKE,
};

/// One stroke: (p0, p1, p2, p3, w_start, w_end)
type StrokeRow = ([f32; 2], [f32; 2], [f32; 2], [f32; 2], f32, f32);

/// A chunk with one glyph (glyph 0) whose strokes are given as rows, default pen `pen`
fn chunk(strokes: &[StrokeRow], pen: f32) -> Vec<u8> {
    let mut c = vec![0u8; MULTI_SKELETON_UBO_SIZE];
    let put = |c: &mut Vec<u8>, vec4: usize, lane: usize, v: f32| {
        let at = vec4 * 16 + lane * 4;
        c[at..at + 4].copy_from_slice(&v.to_le_bytes());
    };
    for (s, (p0, p1, p2, p3, ws, we)) in strokes.iter().enumerate() {
        let b = s * VEC4_PER_STROKE;
        put(&mut c, b, 0, p0[0]);
        put(&mut c, b, 1, p0[1]);
        put(&mut c, b, 2, p1[0]);
        put(&mut c, b, 3, p1[1]);
        put(&mut c, b + 1, 0, p2[0]);
        put(&mut c, b + 1, 1, p2[1]);
        put(&mut c, b + 1, 2, p3[0]);
        put(&mut c, b + 1, 3, p3[1]);
        put(&mut c, b + 2, 0, *ws);
        put(&mut c, b + 2, 1, *we);
    }
    let metas = MULTI_SKELETON_STROKES_AREA_SIZE / 16;
    put(&mut c, metas, 1, strokes.len() as f32);
    put(&mut c, metas, 3, pen);
    c
}

/// A straight stroke from (0, 0) to (1, 0), its control points evenly on the line (a cubic that is the segment
/// itself, t equal to x)
fn straight(ws: f32, we: f32) -> ([f32; 2], [f32; 2], [f32; 2], [f32; 2], f32, f32) {
    (
        [0.0, 0.0],
        [1.0 / 3.0, 0.0],
        [2.0 / 3.0, 0.0],
        [1.0, 0.0],
        ws,
        we,
    )
}

/// The tolerance for the closed forms below: the polyline of a straight cubic is the segment, so the only error is
/// f32 rounding of distances of order 1 (a few ulp of 1.0 = 1.2e-7 each, over about ten operations)
const ROUND: f64 = 2e-6;

#[test]
fn a_straight_stroke_of_one_width_is_a_capsule() {
    // oracle: the distance to a capsule of half-width w about the segment (0,0)-(1,0): |y| − w over the segment,
    // the distance to the nearer end − w past it
    let w = 0.05_f32;
    let c = chunk(&[straight(w, w)], 0.0);
    let mut n = 0;
    for (x, y) in [
        (0.5, 0.2),
        (0.25, -0.03),
        (0.9, 0.0),
        (1.3, 0.4),
        (-0.2, -0.1),
        (0.5, 0.05),
    ] {
        let want = if (0.0..=1.0).contains(&x) {
            f64::abs(y) - f64::from(w)
        } else {
            let ex = if x < 0.0 { 0.0 } else { 1.0 };
            f64::hypot(x - ex, y) - f64::from(w)
        };
        let got =
            f64::from(skeleton_distance(&c, 0, [x as f32, y as f32]).expect("a well-formed chunk"));
        assert!((got - want).abs() < ROUND, "({x}, {y}): {got} vs {want}");
        n += 1;
    }
    assert_eq!(n, 6);
}

#[test]
fn the_pen_widens_along_the_stroke() {
    // oracle: at x in [0, 1] beside the straight stroke, t = x and the pen is mix(w_start, w_end, x)
    let (ws, we) = (0.02_f32, 0.10_f32);
    let c = chunk(&[straight(ws, we)], 0.0);
    for x in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let y = 0.3_f64;
        let pen = f64::from(ws) * (1.0 - x) + f64::from(we) * x;
        let got = f64::from(skeleton_distance(&c, 0, [x as f32, y as f32]).expect("chunk"));
        assert!(
            (got - (y - pen)).abs() < ROUND,
            "x {x}: {got} vs {}",
            y - pen
        );
        let [d, p] = skeleton_stroke(&c, 0, 0, [x as f32, y as f32]).expect("stroke");
        assert!(
            (f64::from(d) - y).abs() < ROUND && (f64::from(p) - pen).abs() < ROUND,
            "x {x}: {d} {p}"
        );
    }
}

#[test]
fn a_width_left_at_zero_takes_the_default_pen() {
    let c = chunk(&[straight(0.0, 0.0)], 0.07);
    let got = f64::from(skeleton_distance(&c, 0, [0.5, 0.2]).expect("chunk"));
    assert!((got - (0.2 - 0.07)).abs() < ROUND, "{got}");
}

#[test]
fn inside_a_stroke_is_negative_and_the_nearest_stroke_wins() {
    // two parallel strokes at y = 0 and y = 1: between them the nearer one answers
    let mut far = straight(0.05, 0.05);
    for p in [&mut far.0, &mut far.1, &mut far.2, &mut far.3] {
        p[1] = 1.0;
    }
    let c = chunk(&[straight(0.05, 0.05), far], 0.0);
    assert!(skeleton_distance(&c, 0, [0.5, 0.01]).expect("chunk") < 0.0);
    let near_far = f64::from(skeleton_distance(&c, 0, [0.5, 0.8]).expect("chunk"));
    assert!((near_far - (0.2 - 0.05)).abs() < ROUND, "{near_far}");
    assert_eq!(skeleton_stroke_count(&c, 0), Some(2));
}

#[test]
fn the_glyph_distance_is_the_least_over_its_strokes() {
    // a real glyph from the packer: skeleton_distance is min over strokes of (distance − pen), exactly
    let payload = pack_skeleton_string("三", dsl_lookup).expect("the string packs");
    let chunks = payload.to_std140_ubo_chunks();
    let c = &chunks[0];
    let strokes = skeleton_stroke_count(c, 0).expect("glyph 0");
    assert!(strokes > 0);
    let mut inside = 0;
    let mut compared = 0;
    for iy in 0..24 {
        for ix in 0..24 {
            let p = [ix as f32 / 23.0, iy as f32 / 23.0];
            let mut least = f32::INFINITY;
            for s in 0..strokes {
                let [d, pen] = skeleton_stroke(c, 0, s, p).expect("stroke");
                least = least.min(d - pen);
            }
            let got = skeleton_distance(c, 0, p).expect("glyph");
            assert_eq!(got.to_bits(), least.to_bits(), "{p:?}");
            inside += usize::from(got < 0.0);
            compared += 1;
        }
    }
    assert_eq!(compared, 576);
    // 三 is three horizontal strokes: some of the grid is inside them, most is not
    assert!(
        inside > 0 && inside < compared / 2,
        "{inside} of {compared} inside"
    );
}

#[test]
fn malformed_input_gives_none_not_a_read_past_the_chunk() {
    let c = chunk(&[straight(0.05, 0.05)], 0.0);
    assert_eq!(
        skeleton_distance(&c[..c.len() - 1], 0, [0.0, 0.0]),
        None,
        "a short chunk"
    );
    assert_eq!(
        skeleton_stroke_count(&c[..c.len() - 1], 0),
        None,
        "a short chunk's count"
    );
    assert_eq!(
        skeleton_stroke(&c[..c.len() - 1], 0, 0, [0.0, 0.0]),
        None,
        "a short chunk's stroke"
    );
    assert_eq!(
        skeleton_distance(&c, MULTI_SKELETON_MAX_GLYPHS, [0.0, 0.0]),
        None,
        "a glyph past the chunk"
    );
    assert_eq!(
        skeleton_stroke(&c, 0, MULTI_SKELETON_MAX_STROKES_PER_GLYPH, [0.0, 0.0]),
        None,
        "a stroke past it"
    );
    // a glyph with no strokes is far from everything, as the shader's 1e10
    assert_eq!(skeleton_distance(&c, 1, [0.0, 0.0]), Some(1e10));
    // a NaN point gives a value (the loop does not panic); the shader gives what its GPU gives
    assert!(skeleton_distance(&c, 0, [f32::NAN, 0.0]).is_some());
}

#[test]
fn a_stroke_count_past_the_limit_reads_only_the_chunk() {
    // the GPU loop stops at 16 strokes whatever the count says; so does the twin
    let mut c = chunk(&[straight(0.05, 0.05)], 0.0);
    let metas = MULTI_SKELETON_STROKES_AREA_SIZE / 16;
    c[metas * 16 + 4..metas * 16 + 8].copy_from_slice(&1000.0_f32.to_le_bytes());
    assert!(skeleton_distance(&c, 0, [0.5, 0.2]).is_some());
}

#[test]
fn the_text_shader_is_built_on_the_distance_chunk() {
    let frag = MULTI_SKELETON_FRAG_GLSL;
    assert!(
        frag.starts_with("#version 300 es\n"),
        "the version comes first"
    );
    // both precisions: an int left mediump is computed in half precision on some GPUs
    assert!(
        frag.contains("precision highp float;") && frag.contains("precision highp int;"),
        "both precisions"
    );
    assert_eq!(
        frag.matches(SKELETON_DISTANCE_GLSL).count(),
        1,
        "the chunk, once"
    );
    assert_eq!(
        frag.matches("vec2 bezierAt(").count(),
        1,
        "no second copy of the curve"
    );
    assert_eq!(frag.matches("vec2 sdBezierApproxT(").count(), 1);
    assert!(
        frag.contains("skeletonStroke(p, glyphIdx, s)"),
        "the text shader calls the chunk"
    );
    let directive = |w: &str| {
        SKELETON_DISTANCE_GLSL
            .lines()
            .any(|l| l.trim_start().starts_with(w))
    };
    assert!(
        !directive("#version"),
        "the chunk leaves the version to its includer"
    );
    assert!(!directive("precision "), "and the precision");
}

#[test]
fn the_dispatcher_provider_packs_kana_and_kanji_with_the_fonts_own_pen() {
    use alice_font::param::MetaFontParams;
    use alice_font::skeleton_shader::skeleton_lookup;
    let text = "株式会社エクストーリア";
    let params = MetaFontParams::sans_regular();
    let payload = pack_skeleton_string(text, skeleton_lookup(&params)).expect("the string packs");
    assert_eq!(
        payload.glyph_count(),
        text.chars().count(),
        "every character has a skeleton"
    );
    // dsl_lookup knows a few radicals only: the same string through it keeps none of these
    let dsl = pack_skeleton_string(text, dsl_lookup).map_or(0, |p| p.glyph_count());
    assert!(dsl < text.chars().count(), "{dsl}");
    // oracle: the pen is the half-width the weight gives (0.01 + 0.08 weight)
    let pen = 0.01 + 0.08 * params.weight;
    for g in &payload.glyphs {
        assert!(g.stroke_count > 0, "{}", g.codepoint);
        assert_eq!(g.pen_width.to_bits(), pen.to_bits(), "{}", g.codepoint);
        assert!(g.stroke_widths[..g.stroke_count]
            .iter()
            .all(|w| w.0 == pen && w.1 == pen));
    }
    // a heavier weight gives a wider pen, so the glyph's distance at a stroke's middle goes down
    let heavy = MetaFontParams {
        weight: 0.7,
        ..params
    };
    let thin = pack_skeleton_string("三", skeleton_lookup(&params))
        .expect("packs")
        .to_std140_ubo_chunks();
    let bold = pack_skeleton_string("三", skeleton_lookup(&heavy))
        .expect("packs")
        .to_std140_ubo_chunks();
    let p = [0.5, 0.5];
    assert!(
        skeleton_distance(&bold[0], 0, p).expect("glyph")
            < skeleton_distance(&thin[0], 0, p).expect("glyph")
    );
}
