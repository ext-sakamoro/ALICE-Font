//! ALICE-Font S6.3.3 v0.6.1 — instanced multi-outline shader PoC。
//!
//! ```sh
//! cargo run --example analytical_outline_multi_poc --features serde -- "三十囗田" out.html
//! cargo run --example analytical_outline_multi_poc --features serde -- "三十三十" repeat.html
//! ```
//!
//! Synthetic outline provider で multi-glyph outline text rendering。
//! 実 BIZ UDPGothic outline は outline provider (L1) 経由で inject する design。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary; Sample outlines: synthetic, not from any copyrighted font.
//! Author: Moroya Sakamoto

use alice_font::outline::{pack_outline_string, OutlineGlyph, MULTI_OUTLINE_UBO_SIZE};
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const VERT_SHADER: &str = include_str!("../../shaders/analytical_multi_outline.vert.glsl");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_multi_outline.frag.glsl");

fn synthetic_outline(ch: char) -> Option<OutlineGlyph> {
    match ch {
        '三' => Some(OutlineGlyph {
            codepoint: '三',
            contours: vec![
                vec![[0.15, 0.75], [0.85, 0.75], [0.85, 0.85], [0.15, 0.85]],
                vec![[0.20, 0.45], [0.80, 0.45], [0.80, 0.55], [0.20, 0.55]],
                vec![[0.10, 0.15], [0.90, 0.15], [0.90, 0.25], [0.10, 0.25]],
            ],
            advance: 1.0,
        }),
        '十' => Some(OutlineGlyph {
            codepoint: '十',
            contours: vec![
                vec![[0.10, 0.45], [0.90, 0.45], [0.90, 0.55], [0.10, 0.55]],
                vec![[0.45, 0.10], [0.55, 0.10], [0.55, 0.90], [0.45, 0.90]],
            ],
            advance: 1.0,
        }),
        '囗' => Some(OutlineGlyph {
            codepoint: '囗',
            contours: vec![
                vec![[0.10, 0.10], [0.90, 0.10], [0.90, 0.90], [0.10, 0.90]],
                vec![[0.20, 0.80], [0.80, 0.80], [0.80, 0.20], [0.20, 0.20]],
            ],
            advance: 1.0,
        }),
        '田' => Some(OutlineGlyph {
            codepoint: '田',
            contours: vec![
                vec![[0.10, 0.10], [0.90, 0.10], [0.90, 0.90], [0.10, 0.90]],
                vec![[0.15, 0.85], [0.48, 0.85], [0.48, 0.52], [0.15, 0.52]],
                vec![[0.52, 0.85], [0.85, 0.85], [0.85, 0.52], [0.52, 0.52]],
                vec![[0.15, 0.48], [0.48, 0.48], [0.48, 0.15], [0.15, 0.15]],
                vec![[0.52, 0.48], [0.85, 0.48], [0.85, 0.15], [0.52, 0.15]],
            ],
            advance: 1.0,
        }),
        _ => None,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: cargo run --example analytical_outline_multi_poc --features serde -- <text> <output.html>"
        );
        eprintln!("supported chars: 三 十 囗 田");
        std::process::exit(1);
    }

    let text = &args[1];
    let out_path = &args[2];

    let payload = pack_outline_string(text, synthetic_outline).unwrap_or_else(|| {
        eprintln!("error: no supported chars in text {text:?} (supported: 三 十 囗 田)");
        std::process::exit(2);
    });

    let glyph_count = payload.glyph_count();
    let chunk_count = payload.chunk_count();
    let total_advance = payload.total_advance;

    let ubo_chunks = payload.to_std140_ubo_chunks();
    for c in &ubo_chunks {
        assert_eq!(c.len(), MULTI_OUTLINE_UBO_SIZE);
    }
    let ubo_total_bytes = ubo_chunks.len() * MULTI_OUTLINE_UBO_SIZE;

    let instance_counts: Vec<usize> = (0..chunk_count)
        .map(|ci| {
            let start = ci * alice_font::outline::MULTI_OUTLINE_MAX_GLYPHS;
            let end = (start + alice_font::outline::MULTI_OUTLINE_MAX_GLYPHS).min(glyph_count);
            end - start
        })
        .collect();

    let ubo_chunks_json = serde_json::to_string(&ubo_chunks).expect("serde_json");
    let instance_counts_json = serde_json::to_string(&instance_counts).expect("serde_json");

    // v0.6.4: canvas は em box (幅 total_advance × 高さ 1 + 2 × Y_PAD_EM) の
    // aspect に合わせて sizing。descender / ascender 分の padding を確保。
    const PX_PER_EM: f32 = 200.0;
    const Y_PAD_EM: f32 = 0.15;
    let em_h = 1.0 + 2.0 * Y_PAD_EM;
    let canvas_w = (total_advance * PX_PER_EM).clamp(512.0, 2048.0) as u32;
    let canvas_h = (em_h * PX_PER_EM).ceil() as u32;

    let html = TEMPLATE_HTML
        .replace("{{TEXT}}", text)
        .replace("{{GLYPH_COUNT}}", &glyph_count.to_string())
        .replace("{{CHUNK_COUNT}}", &chunk_count.to_string())
        .replace("{{UBO_TOTAL_BYTES}}", &ubo_total_bytes.to_string())
        .replace("{{MULTI_UBO_SIZE}}", &MULTI_OUTLINE_UBO_SIZE.to_string())
        .replace("{{TOTAL_ADVANCE}}", &format!("{total_advance:.2}"))
        .replace("{{CANVAS_W}}", &canvas_w.to_string())
        .replace("{{CANVAS_H}}", &canvas_h.to_string())
        .replace("{{Y_PAD_EM}}", &format!("{Y_PAD_EM}"))
        .replace("{{VERT_SHADER}}", VERT_SHADER)
        .replace("{{FRAG_SHADER}}", FRAG_SHADER)
        .replace("{{UBO_CHUNKS_JSON}}", &ubo_chunks_json)
        .replace("{{INSTANCE_COUNTS_JSON}}", &instance_counts_json);

    let mut f = std::fs::File::create(out_path)
        .unwrap_or_else(|e| panic!("failed to create {out_path}: {e}"));
    f.write_all(html.as_bytes())
        .unwrap_or_else(|e| panic!("failed to write {out_path}: {e}"));

    let draw_calls_before = glyph_count;
    let draw_calls_after = chunk_count;
    let reduction_pct = (draw_calls_after * 100)
        .checked_div(draw_calls_before)
        .map_or(0, |ratio| 100 - ratio);

    println!("✓ generated {out_path}");
    println!("  text: {text} ({} chars)", text.chars().count());
    println!("  glyphs packed: {glyph_count}");
    println!("  total advance: {total_advance:.2} em");
    println!("  canvas: {canvas_w}×{canvas_h} px");
    println!(
        "  chunks: {chunk_count} (each {MULTI_OUTLINE_UBO_SIZE}B fixed UBO、texture upload = 0)"
    );
    println!("  UBO total: {ubo_total_bytes} byte");
    println!(
        "  draw calls: {draw_calls_before} → {draw_calls_after} (-{reduction_pct}% via drawArraysInstanced)"
    );
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2-capable browser。");
}
