//! ALICE-Font S6.4 v0.7.0 — zero-asset browser demo (parametric skeleton path)。
//!
//! ```sh
//! cargo run --example s64_zero_asset_demo --features serde -- out.html
//! cargo run --example s64_zero_asset_demo --features serde -- out.html "一二三感議護"
//! ```
//!
//! Rust の `kanji::build_skeleton()` (radical + IDS 法則から Bezier stroke を
//! 手続き的に生成) が出力する `GlyphSkeleton` を UBO に pack、fragment
//! shader が per-pixel で cubic Bezier 距離 + variable-width pen SDF を評価。
//!
//! # Zero-asset property
//!
//! - **Ship 内容**: shader source (5 KB) + landing HTML (10 KB) + skeleton
//!   UBO (2,128 B / chunk 、動的生成) = 合計 数十 KB
//! - **Runtime texture upload**: 0 byte
//! - **Data provider**: L0 単独 (radicals + IDS DB + composition = 法則 shipping)
//! - **Coverage**: 教育漢字 grade 1-6 の 1,018 kanji + 追加 35 = **1,053 kanji**
//!
//! v0.6.x outline path が L1 (outline provider) の 20 MB BIZ UDPGothic outline
//! data を requires したのに対し、本 demo は L0 のみで完結。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use alice_font::glyph::{kanji, MAX_GLYPH_STROKES};
use alice_font::skeleton_shader::{
    pack_skeleton_string, SkeletonGlyph, MULTI_SKELETON_MAX_GLYPHS, MULTI_SKELETON_UBO_SIZE,
};
use alice_font::stroke::Stroke;
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const VERT_SHADER: &str = include_str!("../../shaders/analytical_multi_skeleton.vert.glsl");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_multi_skeleton.frag.glsl");

const PX_PER_EM: f32 = 200.0;
const Y_PAD_EM: f32 = 0.15;
/// Default pen radius (em unit)。BIZ UDPGothic Regular に近い weight。
const DEFAULT_PEN_WIDTH: f32 = 0.045;

fn parametric_skeleton(ch: char) -> Option<SkeletonGlyph> {
    let skel = kanji::build_skeleton(ch)?;
    // Copy from GlyphSkeleton (16 stroke array) to SkeletonGlyph
    let mut strokes = [Stroke::line(
        alice_font::stroke::Point2::ZERO,
        alice_font::stroke::Point2::ZERO,
    ); MAX_GLYPH_STROKES];
    strokes[..skel.stroke_count.min(MAX_GLYPH_STROKES)]
        .copy_from_slice(&skel.strokes[..skel.stroke_count.min(MAX_GLYPH_STROKES)]);
    Some(SkeletonGlyph {
        codepoint: ch,
        strokes,
        stroke_widths: [(DEFAULT_PEN_WIDTH, DEFAULT_PEN_WIDTH); MAX_GLYPH_STROKES],
        stroke_count: skel.stroke_count.min(MAX_GLYPH_STROKES),
        advance: skel.advance,
        pen_width: DEFAULT_PEN_WIDTH,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "usage: cargo run --example s64_zero_asset_demo --features serde -- <output.html> [text]"
        );
        eprintln!("no text arg → uses default showcase");
        std::process::exit(1);
    }

    let out_path = &args[1];
    let text = if args.len() >= 3 {
        args[2].clone()
    } else {
        // v0.7.0-alpha: L0 parametric subset で real Bezier stroke render される
        // 73 kanji のうち variety を代表する 27 kanji をキュレート。
        // Excluded 例: 四 五 六 七 九 千 万 億 川 火 王 小 足 愛 頭 母 父 兄 弟 姉 家
        //   友 算 科 楽 図 育 朝 昼 夜 春 夏 秋 冬 雨 風 雪 雲 空 海 町 書 聞 食
        //   飲 買 下 右 左 東 南 北 近 遠 低 短 → 現状 placeholder rectangle。
        // 完全 rendering 対応は radicals.rs + ids_db.rs の未実装 leaf decomposition
        // 拡充 (future v0.7.x-v0.8.x) を待つ。
        "一二三十百山木目日月田水土大心明森林相想思考知感学校国語会".to_string()
    };

    let payload = pack_skeleton_string(&text, parametric_skeleton).unwrap_or_else(|| {
        eprintln!("error: no chars in text {text:?} are in KANJI_DB (1,053 kanji)");
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
        .replace("{{PEN_WIDTH}}", &format!("{DEFAULT_PEN_WIDTH}"))
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
    println!("  glyphs packed: {glyph_count}");
    println!("  per-glyph stats (parametric skeleton):");
    for ch in text.chars() {
        if let Some(g) = parametric_skeleton(ch) {
            println!(
                "    {ch}: {} strokes (advance={:.2}, pen={:.3})",
                g.stroke_count, g.advance, g.pen_width
            );
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
    println!("Zero-asset property: skeleton data is generated at build/runtime");
    println!("from radicals + IDS DB (法則), not from any embedded outline table。");
}
