//! ALICE-Font S6.3.2b.3 example — instanced multi-kanji UBO shader PoC (v0.5.3)。
//!
//! ```sh
//! cargo run --example analytical_shader_multi_poc --features serde -- "森明林" text.html
//! cargo run --example analytical_shader_multi_poc --features serde -- "森羅万象" text2.html
//! cargo run --example analytical_shader_multi_poc --features serde -- "日月火水木金土" week.html
//! cargo run --example analytical_shader_multi_poc --features serde -- "一二三四五六七八九十百千万木火水土" chunk_split.html
//! ```
//!
//! v0.5.3 (S6.3.2b.3): per-glyph draw loop を廃止、`drawArraysInstanced`
//! 経由で 1 chunk = 1 draw call。16 glyph 超過は N/16 chunk に自動分割。
//! bitmap / atlas / texture upload = 0 byte 継続。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use alice_font::param::MetaFontParams;
use alice_font::shader::{pack_string_for_shader, MAX_GLYPHS_PER_DRAW, MULTI_GLYPH_UBO_SIZE};
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const VERT_SHADER: &str = include_str!("../../shaders/analytical_multi_glyph.vert.glsl");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_multi_glyph.frag.glsl");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: cargo run --example analytical_shader_multi_poc --features serde -- <text> <output.html>"
        );
        eprintln!(
            "example: cargo run --example analytical_shader_multi_poc --features serde -- 森明林 text.html"
        );
        std::process::exit(1);
    }

    let text = &args[1];
    let out_path = &args[2];

    // v0.5.4: shader path 用に sans_hairline preset (weight=0.15) を採用。
    // sans_regular (weight=0.45) では pen 幅が大きすぎて 三 等の複数線が
    // 視覚的に merge する (user report 2026-07-30)。
    let params = MetaFontParams::sans_hairline();
    let payload = pack_string_for_shader(text, &params).unwrap_or_else(|| {
        eprintln!("error: no registered kanji in text {text:?} (or all glyphs failed to pack)");
        std::process::exit(2);
    });

    let glyph_count = payload.glyph_count();
    let total_advance = payload.total_advance;
    let chunk_count = payload.chunk_count();

    // Multi-glyph UBO chunks (each = 12,816 byte fixed)。
    let ubo_chunks = payload.to_std140_ubo_chunks();
    assert_eq!(ubo_chunks.len(), chunk_count);
    for c in &ubo_chunks {
        assert_eq!(c.len(), MULTI_GLYPH_UBO_SIZE);
    }
    let ubo_total_bytes = ubo_chunks.len() * MULTI_GLYPH_UBO_SIZE;

    // Per-chunk instance count (= drawArraysInstanced の 2nd arg)。
    // 最終 chunk は端数、他は MAX_GLYPHS_PER_DRAW。
    let instance_counts: Vec<usize> = (0..chunk_count)
        .map(|ci| {
            let start = ci * MAX_GLYPHS_PER_DRAW;
            let end = (start + MAX_GLYPHS_PER_DRAW).min(glyph_count);
            end - start
        })
        .collect();

    let ubo_chunks_json = serde_json::to_string(&ubo_chunks).expect("serialize UBO chunks");
    let instance_counts_json =
        serde_json::to_string(&instance_counts).expect("serialize instance counts");

    let canvas_w = (total_advance * 128.0).clamp(512.0, 2048.0) as u32;
    let canvas_h: u32 = 512;

    let html = TEMPLATE_HTML
        .replace("{{TEXT}}", text)
        .replace("{{GLYPH_COUNT}}", &glyph_count.to_string())
        .replace("{{CHUNK_COUNT}}", &chunk_count.to_string())
        .replace("{{UBO_TOTAL_BYTES}}", &ubo_total_bytes.to_string())
        .replace("{{TOTAL_ADVANCE}}", &format!("{total_advance:.2}"))
        .replace("{{CANVAS_W}}", &canvas_w.to_string())
        .replace("{{CANVAS_H}}", &canvas_h.to_string())
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
    println!("  glyphs packed: {glyph_count} (skip 未登録)");
    println!("  total advance: {total_advance:.2} em");
    println!("  canvas: {canvas_w}×{canvas_h} px");
    println!("  chunks: {chunk_count} (each 12,816B fixed UBO、texture upload = 0)");
    println!("  UBO total: {ubo_total_bytes} byte");
    println!(
        "  draw calls: {draw_calls_before} → {draw_calls_after} (-{reduction_pct}% via drawArraysInstanced)"
    );
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2-capable browser (instanced rendering path).");
}
