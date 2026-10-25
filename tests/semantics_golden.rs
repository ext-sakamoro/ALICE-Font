//! Recorded-bit goldens of what the crate computes for fixed inputs (ALICE-Law G4, the pins of
//! [`FONT_SEMANTICS_ID`], G6).
//!
//! Each scenario runs one glyph computation with fixed inputs and records the SHA-256 of every byte it
//! produced, in a stable little-endian layout. The analytic oracles of `tests/outline_raster.rs` and
//! `tests/analytic_oracle.rs` are the independent checks of the values themselves; these goldens pin that the
//! bytes do not move. Every CI leg (debug and release, x86_64 and aarch64, macOS, Windows) must reproduce them
//! bit for bit.

use alice_font::font_dsl::dsl_lookup;
use alice_font::game::{style_glyph, Color4, EffectStack, GameTextStyle, TextEffect};
use alice_font::glyph::{GlyphGenerator, GlyphSdf};
use alice_font::medial_axis::extract_medial_axis;
use alice_font::outline::{pack_outline_for_shader, rasterize, OutlineGlyph};
use alice_font::shader::pack_glyph_for_shader;
use alice_font::skeleton_shader::pack_skeleton_string;
use alice_font::{MetaFontParams, PenModel};

use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// The SHA-256 of `bytes` when it is not `expected`
fn golden_mismatch(bytes: &[u8], expected: &str) -> Option<String> {
    let got = hex(&sha256(bytes));
    (got != expected).then_some(got)
}

/// A scenario that produced nothing compared nothing
fn produced(name: &str, bytes: &[u8]) {
    assert!(!bytes.is_empty(), "{name}: the scenario produced no bytes");
}

fn assert_golden(name: &str, bytes: &[u8], expected: &str) {
    produced(name, bytes);
    if let Some(got) = golden_mismatch(bytes, expected) {
        panic!("{name} golden moved: {got}, recorded {expected}");
    }
}

#[test]
fn checker_rejects_a_wrong_digest() {
    let recorded = hex(&sha256(b"recorded bits"));
    assert_eq!(golden_mismatch(b"recorded bits", &recorded), None);
    assert!(golden_mismatch(b"moved bits", &recorded).is_some());
}

#[test]
#[should_panic(expected = "produced no bytes")]
fn checker_rejects_a_scenario_that_produced_nothing() {
    produced("empty", b"");
}

// ─── serialisation helpers ────────────────────────────────────────────────

fn push_f32(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn sdf_bytes(sdf: &GlyphSdf) -> Vec<u8> {
    let mut out = Vec::with_capacity(sdf.data.len() * 4 + 4);
    for &v in &sdf.data {
        push_f32(&mut out, v);
    }
    push_f32(&mut out, sdf.advance);
    out
}

/// A star with slanted edges and a box it overlaps; rounding at every step shows in the bits.
fn star_over_box() -> OutlineGlyph {
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
    const BOX: [f32; 4] = [0.25, 0.0625, 0.75, 0.6875];
    let rect = vec![
        [BOX[0], BOX[1]],
        [BOX[2], BOX[1]],
        [BOX[2], BOX[3]],
        [BOX[0], BOX[3]],
    ];
    OutlineGlyph {
        codepoint: 'x',
        contours: vec![STAR.to_vec(), rect],
        advance: 0.75,
    }
}

// ─── scenarios ────────────────────────────────────────────────────────────

/// Pin 1 — the outline rasteriser's signed distance field for a star over a box.
#[test]
fn raster_sdf_of_a_star_over_a_box() {
    let sdf = rasterize(&star_over_box()).expect("rasterises");
    assert_golden(
        "raster sdf of a star over a box",
        &sdf_bytes(&sdf),
        "ca784b9d637a6a132271d51ccb5ca4f30e0643ee5ca613222aeb49a9813f6dc7",
    );
}

/// Pin 2 — the parametric glyph generator's field for a fixed character and parameters.
#[test]
fn parametric_glyph_sdf() {
    let gen = GlyphGenerator::new(&MetaFontParams::sans_regular());
    let sdf = gen.generate(b'A');
    assert_golden(
        "parametric glyph sdf",
        &sdf_bytes(&sdf),
        "58b52eb6783ade233599dbfa6553430b05614d34230c11d61fc096f2530b5a01",
    );
}

/// Pin 3 — the pen half-width over a sweep of stroke angles (the broad-nib model on det-math trig).
#[test]
fn pen_half_width_over_stroke_angles() {
    let pen = PenModel::from_params(&MetaFontParams::sans_regular());
    let mut out = Vec::new();
    for i in 0..=64u32 {
        let angle = (i as f32) / 64.0 * core::f32::consts::TAU;
        push_f32(&mut out, pen.half_width_at_angle(angle));
    }
    assert_golden(
        "pen half width over stroke angles",
        &out,
        "2d080df3a5952ee5c7d6aefc4cff34dc0fb42f5249a5a0b21315ae5ccae11360",
    );
}

/// Pin 4 — the medial axis of a star over a box: each chain's point count then its points.
#[test]
fn medial_axis_of_a_star_over_a_box() {
    let g = star_over_box();
    let chains = extract_medial_axis(&g.contours);
    let mut out = Vec::new();
    out.extend_from_slice(&u32::try_from(chains.len()).unwrap().to_le_bytes());
    for chain in &chains {
        out.extend_from_slice(&u32::try_from(chain.points.len()).unwrap().to_le_bytes());
        for p in &chain.points {
            push_f32(&mut out, p[0]);
            push_f32(&mut out, p[1]);
        }
    }
    assert_golden(
        "medial axis of a star over a box",
        &out,
        "329c51132db22ce2869eb4088b4e616ffee2730db001d2bf47d930c29184d743",
    );
}

/// Pin 5 — the analytical shader payload in its std140 layout for a fixed kanji.
#[test]
fn analytical_shader_payload_std140() {
    let payload = pack_glyph_for_shader('森', &MetaFontParams::sans_regular())
        .expect("the glyph packs for the shader");
    assert_golden(
        "analytical shader payload std140",
        &payload.to_std140_bytes(),
        "1ab0f729b9234d9ababb18e8f51426febf208d155b45a2189f074602fec47f5a",
    );
}

/// Pin 6 — the outline shader payload in its std140 layout for a star over a box.
#[test]
fn outline_shader_payload_std140() {
    let payload =
        pack_outline_for_shader(&star_over_box()).expect("the outline packs for the shader");
    assert_golden(
        "outline shader payload std140",
        &payload.to_std140_bytes(),
        "89aceadec0312cda35ba3cd4ba8887b1adeed1e501e10b57db49b9127be7c6bc",
    );
}

/// Pin 7 — the multi-skeleton string payload in its std140 UBO chunks for a fixed string.
#[test]
fn skeleton_string_payload_std140() {
    let payload = pack_skeleton_string("三", dsl_lookup).expect("the string packs");
    let mut out = Vec::new();
    for chunk in payload.to_std140_ubo_chunks() {
        out.extend_from_slice(&chunk);
    }
    assert_golden(
        "skeleton string payload std140",
        &out,
        "12bc9dac20827a3c9a86d1548d401f977ca844af2da73b3c73550d9a27770134",
    );
}

/// Pin 9 — the styled raster (outline, shadow, glow and gradient) of the pin-1 field: every pixel's RGBA.
///
/// The style is an input; the mapping from the field and the style to the pixels (the smoothstep, the gradient
/// and the Porter-Duff compositing) is this crate's computation, so moving it moves these bits.
#[test]
fn styled_glyph_pixels() {
    let sdf = rasterize(&star_over_box()).expect("rasterises");
    let mut effects = EffectStack::new();
    effects.push(TextEffect::Outline {
        color: Color4::BLACK,
        width: 0.05,
    });
    effects.push(TextEffect::Shadow {
        color: Color4::new(0.0, 0.0, 0.0, 0.6),
        offset_x: 0.03,
        offset_y: -0.03,
        softness: 0.04,
    });
    effects.push(TextEffect::Glow {
        color: Color4::new(0.0, 1.0, 0.5, 0.8),
        radius: 0.15,
        falloff: 2.0,
    });
    effects.push(TextEffect::Gradient {
        color_top: Color4::new(1.0, 0.9, 0.2, 1.0),
        color_bottom: Color4::new(0.8, 0.1, 0.1, 1.0),
        radial_mix: 0.3,
    });
    let style = GameTextStyle {
        base_color: Color4::WHITE,
        threshold: 0.0,
        aa_width: 0.03,
        effects,
    };
    let styled = style_glyph(&sdf, &style);
    let mut out = Vec::with_capacity(styled.pixels.len() * 16);
    for p in &styled.pixels {
        push_f32(&mut out, p.r);
        push_f32(&mut out, p.g);
        push_f32(&mut out, p.b);
        push_f32(&mut out, p.a);
    }
    assert_golden(
        "styled glyph pixels",
        &out,
        "3eed7265a5fb213b46caa7c3f0fb4e0b5f4ea9beeff5f931ffa1d7e420329aec",
    );
}
