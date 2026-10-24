//! ALICE-Font S6.3.2b.4 diagnostic — single kanji + shader debug info。
//!
//! ```sh
//! cargo run --example analytical_shader_diagnostic_poc --features serde -- 三 out.html
//! cargo run --example analytical_shader_diagnostic_poc --features serde -- 森 mori_debug.html
//! ```
//!
//! Diagnostic HTML: 大 canvas 512×512、side panel に stroke count / control
//! points / pen params 表示、grid overlay + shader compile log inspector 付き。
//! Multi-glyph 描画壊れの原因究明用 (v0.5.4 flat UBO fix の validation)。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use alice_font::param::MetaFontParams;
use alice_font::shader::pack_glyph_for_shader;
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_glyph.frag.glsl");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: cargo run --example analytical_shader_diagnostic_poc --features serde -- <char> <output.html>"
        );
        std::process::exit(1);
    }

    let ch: char = args[1]
        .chars()
        .next()
        .expect("first arg must be a single character");
    let out_path = &args[2];

    let params = MetaFontParams::sans_hairline();
    let payload = pack_glyph_for_shader(ch, &params).unwrap_or_else(|| {
        eprintln!(
            "error: {ch} (U+{:04X}) is not registered in KANJI_DB",
            ch as u32
        );
        std::process::exit(2);
    });

    let ubo_bytes = payload.to_std140_bytes();
    let ubo_json = serde_json::to_string(&ubo_bytes).expect("serde_json");
    let payload_json = serde_json::to_string_pretty(&payload).expect("serde_json");

    let html = TEMPLATE_HTML
        .replace("{{CHAR}}", &ch.to_string())
        .replace("{{CHAR_HEX}}", &format!("{:04X}", ch as u32))
        .replace("{{STROKE_COUNT}}", &payload.stroke_count().to_string())
        .replace(
            "{{PEN_BASE_WIDTH}}",
            &format!("{:.4}", payload.pen.base_width),
        )
        .replace("{{PEN_CONTRAST}}", &format!("{:.3}", payload.pen.contrast))
        .replace("{{PEN_ANGLE}}", &format!("{:.3}", payload.pen.pen_angle))
        .replace(
            "{{PEN_ROUNDNESS}}",
            &format!("{:.3}", payload.pen.roundness),
        )
        .replace("{{ADVANCE}}", &format!("{:.3}", payload.advance))
        .replace("{{FRAG_SHADER}}", FRAG_SHADER)
        .replace("{{UBO_BYTES_JSON}}", &ubo_json)
        .replace("{{PAYLOAD_DEBUG_JSON}}", &payload_json);

    let mut f = std::fs::File::create(out_path)
        .unwrap_or_else(|e| panic!("failed to create {out_path}: {e}"));
    f.write_all(html.as_bytes())
        .unwrap_or_else(|e| panic!("failed to write {out_path}: {e}"));

    println!("✓ generated {out_path}");
    println!("  char: {ch} (U+{:04X})", ch as u32);
    println!("  strokes: {}", payload.stroke_count());
    println!("  pen.base_width: {:.4} em", payload.pen.base_width);
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2 browser (single-glyph debug view).");
}
