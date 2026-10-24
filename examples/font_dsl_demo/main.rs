//! ALICE-Font Font DSL demo (v0.7.1-alpha)。
//!
//! ```sh
//! cargo run --example font_dsl_demo --features serde -- out.html
//! cargo run --example font_dsl_demo --features serde -- out.html "木林森明相"
//! ```
//!
//! Font DSL (`alice_font::font_dsl`) で手書き定義した MVP 20 char
//! (一二三十木日月田口目山水火土心大人力女子 + 明林森相) を skeleton
//! shader path で render する。
//!
//! v0.7.0-alpha の `kanji::build_skeleton` が `radicals.rs + ids_db.rs` に
//! 依存し 73 / 1053 char しか real render しなかったのに対し、本 demo は
//! DSL 経由で **20 char 全部が real Bezier stroke** で描画される。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use alice_font::font_dsl::dsl_lookup;
use alice_font::skeleton_shader::{
    pack_skeleton_string, MULTI_SKELETON_MAX_GLYPHS, MULTI_SKELETON_UBO_SIZE,
};
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const VERT_SHADER: &str = include_str!("../../shaders/analytical_multi_skeleton.vert.glsl");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_multi_skeleton.frag.glsl");

const PX_PER_EM: f32 = 200.0;
const Y_PAD_EM: f32 = 0.15;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "usage: cargo run --example font_dsl_demo --features serde -- <output.html> [text]"
        );
        eprintln!("no text arg → uses default 20-char showcase");
        std::process::exit(1);
    }

    let out_path = &args[1];
    let text = if args.len() >= 3 {
        args[2].clone()
    } else {
        // v0.7.4-alpha Font DSL: 55 radical + 15 composed = 70 char 100% real render
        // Curated 40-char showcase for readability
        "一二三四五六七八九十百千万上下左右中木日月田口目山川石天空花草雨白青心大人力女子手足耳見立米糸車明林森相男好岩早音想意半電".to_string()
    };

    let payload = pack_skeleton_string(&text, dsl_lookup).unwrap_or_else(|| {
        eprintln!("error: no chars in text {text:?} are in Font DSL library (MVP 20)");
        std::process::exit(2);
    });

    let glyph_count = payload.glyph_count();
    let chunk_count = payload.chunk_count();
    let total_advance = payload.total_advance;

    let ubo_chunks = payload.to_std140_ubo_chunks();
    for c in &ubo_chunks {
        assert_eq!(c.len(), MULTI_SKELETON_UBO_SIZE);
    }
    let ubo_total_bytes = ubo_chunks.len() * MULTI_SKELETON_UBO_SIZE;

    let instance_counts: Vec<usize> = (0..chunk_count)
        .map(|ci| {
            let start = ci * MULTI_SKELETON_MAX_GLYPHS;
            let end = (start + MULTI_SKELETON_MAX_GLYPHS).min(glyph_count);
            end - start
        })
        .collect();

    let ubo_chunks_json = serde_json::to_string(&ubo_chunks).expect("serde_json");
    let instance_counts_json = serde_json::to_string(&instance_counts).expect("serde_json");

    let em_h = 1.0 + 2.0 * Y_PAD_EM;
    let canvas_w = (total_advance * PX_PER_EM).clamp(512.0, 2048.0) as u32;
    let canvas_h = (em_h * PX_PER_EM).ceil() as u32;

    let html = TEMPLATE_HTML
        .replace("{{TEXT}}", &text)
        .replace("{{GLYPH_COUNT}}", &glyph_count.to_string())
        .replace("{{CHUNK_COUNT}}", &chunk_count.to_string())
        .replace("{{UBO_TOTAL_BYTES}}", &ubo_total_bytes.to_string())
        .replace("{{MULTI_UBO_SIZE}}", &MULTI_SKELETON_UBO_SIZE.to_string())
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

    let input_chars = text.chars().count();
    let skipped = input_chars - glyph_count;

    println!("✓ generated {out_path}");
    println!("  text: {text} ({input_chars} chars input, {skipped} skipped as unregistered)");
    println!("  glyphs packed (DSL-defined): {glyph_count}");
    println!("  per-glyph stats:");
    for ch in text.chars() {
        if let Some(g) = dsl_lookup(ch) {
            println!(
                "    {ch}: {} strokes (DSL, pen={:.3})",
                g.stroke_count, g.pen_width
            );
        } else {
            println!("    {ch}: (not in DSL library)");
        }
    }
    println!("  total advance: {total_advance:.2} em");
    println!("  canvas: {canvas_w}×{canvas_h} px");
    println!(
        "  chunks: {chunk_count} draw call(s) (each {MULTI_SKELETON_UBO_SIZE}B UBO、texture upload = 0)"
    );
    println!("  UBO total: {ubo_total_bytes} byte");
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2 browser。");
    println!("Font DSL v0.7.1-alpha: 手書き stroke definitions で 20 char 100% real render");
}
