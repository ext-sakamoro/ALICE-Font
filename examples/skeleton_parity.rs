//! What a GPU needs to check the skeleton shaders against the CPU, as JSON on stdout (read by
//! `tools/skeleton-parity/run.mjs`): the text shader as built ([`MULTI_SKELETON_FRAG_GLSL`]) and without its
//! `precision highp int` line (the same program with the int precision left to the driver), its vertex shader, a
//! fragment shader that writes [`SKELETON_DISTANCE_GLSL`]'s `skeletonDistance` into a float target, the packed
//! std140 chunks of a string, and the CPU twin's distance ([`skeleton_distance`]) at every pixel centre of a
//! `GRID` × `GRID` target per glyph.
//!
//! The grid's em coordinates are exact in f32 on both sides: pixel centre `i + 0.5`, over `GRID` (a power of two),
//! times `SPAN` (a power of two), plus `ORIGIN` (a multiple of a power of two), so a difference between the two is
//! the distance's arithmetic, not the point's.
//!
//!     cargo run --example skeleton_parity -- [string] > parity.json
//!
//! Author: Moroya Sakamoto

use alice_font::param::MetaFontParams;
use alice_font::skeleton_shader::{
    pack_skeleton_string, skeleton_distance, skeleton_lookup, MULTI_SKELETON_FRAG_GLSL,
    MULTI_SKELETON_MAX_GLYPHS, MULTI_SKELETON_VERT_GLSL, SKELETON_DISTANCE_GLSL,
};

const GRID: usize = 64;
const SPAN: f32 = 2.0;
const ORIGIN: f32 = -0.5;

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn main() {
    let text = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "株式会社エクストーリア".to_string());
    let params = MetaFontParams::sans_regular();
    let payload = pack_skeleton_string(&text, skeleton_lookup(&params)).expect("the string packs");
    assert_eq!(
        payload.glyph_count(),
        text.chars().count(),
        "every character has a skeleton"
    );
    let chunks = payload.to_std140_ubo_chunks();
    let glyphs = payload.glyph_count();
    let without_int = MULTI_SKELETON_FRAG_GLSL.replacen("precision highp int;\n", "", 1);
    assert_ne!(
        without_int, MULTI_SKELETON_FRAG_GLSL,
        "the int precision line is there to take out"
    );
    let distance_frag = format!(
        "#version 300 es\nprecision highp float;\nprecision highp int;\n{SKELETON_DISTANCE_GLSL}\n\
         uniform int uGlyph;\nout vec4 fragColor;\nvoid main() {{\n    \
         vec2 p = gl_FragCoord.xy / {GRID}.0 * {SPAN:.1} + vec2({ORIGIN:.1});\n    \
         fragColor = vec4(skeletonDistance(p, uGlyph), 0.0, 0.0, 1.0);\n}}\n"
    );
    let mut cpu = Vec::new();
    for g in 0..glyphs {
        let (chunk, slot) = (g / MULTI_SKELETON_MAX_GLYPHS, g % MULTI_SKELETON_MAX_GLYPHS);
        let mut values = Vec::with_capacity(GRID * GRID);
        for iy in 0..GRID {
            for ix in 0..GRID {
                // the same expression the distance shader computes from gl_FragCoord (exact in f32)
                let p = [
                    (ix as f32 + 0.5) / GRID as f32 * SPAN + ORIGIN,
                    (iy as f32 + 0.5) / GRID as f32 * SPAN + ORIGIN,
                ];
                values.push(skeleton_distance(&chunks[chunk], slot, p).expect("a packed chunk"));
            }
        }
        cpu.push((chunk, slot, values));
    }
    let chunk_json: Vec<String> = chunks
        .iter()
        .map(|c| {
            format!(
                "[{}]",
                c.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
            )
        })
        .collect();
    let counts: Vec<String> = (0..chunks.len())
        .map(|c| {
            (glyphs - c * MULTI_SKELETON_MAX_GLYPHS)
                .min(MULTI_SKELETON_MAX_GLYPHS)
                .to_string()
        })
        .collect();
    let cpu_json: Vec<String> = cpu
        .iter()
        .map(|(c, s, v)| {
            // f32 bit patterns, so the comparison sees every bit the CPU computed
            let bits: Vec<String> = v.iter().map(|x| x.to_bits().to_string()).collect();
            format!(
                "{{\"chunk\":{c},\"slot\":{s},\"bits\":[{}]}}",
                bits.join(",")
            )
        })
        .collect();
    println!(
        "{{\"text\":{},\"grid\":{GRID},\"vert\":{},\"frag\":{},\"frag_without_int\":{},\"distance_frag\":{},\
         \"chunks\":[{}],\"instance_counts\":[{}],\"cpu\":[{}]}}",
        json_str(&text),
        json_str(MULTI_SKELETON_VERT_GLSL),
        json_str(MULTI_SKELETON_FRAG_GLSL),
        json_str(&without_int),
        json_str(&distance_frag),
        chunk_json.join(","),
        counts.join(","),
        cpu_json.join(",")
    );
}
