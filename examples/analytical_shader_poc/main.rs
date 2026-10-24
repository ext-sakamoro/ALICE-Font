//! ALICE-Font S6.3.2b example — analytical UBO shader PoC HTML generator (v3)。
//!
//! ```sh
//! cargo run --example analytical_shader_poc --features serde -- 森 out.html
//! cargo run --example analytical_shader_poc --features serde -- 明 mei.html
//! ```
//!
//! v0.5.0 では JSON payload + uniform arrays を使っていたが、v0.5.2 (S6.3.2b) で
//! Rust `to_std140_bytes()` 出力をそのまま WebGL2 UBO に upload する方式に切替。
//! std140 layout は Rust ↔ GLSL bit-exact 一致 (byte-level test で検証済)。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use alice_font::param::MetaFontParams;
use alice_font::shader::{pack_glyph_for_shader, GLYPH_UBO_SIZE};
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_glyph.frag.glsl");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: cargo run --example analytical_shader_poc --features serde -- <char> <output.html>");
        eprintln!(
            "example: cargo run --example analytical_shader_poc --features serde -- 森 mori.html"
        );
        std::process::exit(1);
    }

    let ch: char = args[1]
        .chars()
        .next()
        .expect("first arg must be a single character");
    let out_path = &args[2];

    // v0.5.4: shader path 用に sans_hairline preset (weight=0.15) を採用。
    let params = MetaFontParams::sans_hairline();
    let payload = pack_glyph_for_shader(ch, &params).unwrap_or_else(|| {
        eprintln!(
            "error: {ch} (U+{:04X}) is not registered in KANJI_DB (or has 0 strokes)",
            ch as u32
        );
        std::process::exit(2);
    });

    let stroke_count = payload.stroke_count();
    let ubo_bytes = payload.to_std140_bytes();
    assert_eq!(
        ubo_bytes.len(),
        GLYPH_UBO_SIZE,
        "UBO must be fixed 800 byte"
    );
    // JSON array of u8 (browser: new Uint8Array(...))。
    let ubo_json = serde_json::to_string(&ubo_bytes).expect("serde_json must succeed");

    let html = TEMPLATE_HTML
        .replace("{{CHAR}}", &ch.to_string())
        .replace("{{CHAR_HEX}}", &format!("{:04X}", ch as u32))
        .replace("{{UBO_BYTES}}", &ubo_bytes.len().to_string())
        .replace("{{STROKE_COUNT}}", &stroke_count.to_string())
        .replace("{{FRAG_SHADER}}", FRAG_SHADER)
        .replace("{{UBO_BYTES_JSON}}", &ubo_json);

    let mut f = std::fs::File::create(out_path)
        .unwrap_or_else(|e| panic!("failed to create {out_path}: {e}"));
    f.write_all(html.as_bytes())
        .unwrap_or_else(|e| panic!("failed to write {out_path}: {e}"));

    println!("✓ generated {out_path}");
    println!("  char: {ch} (U+{:04X})", ch as u32);
    println!("  strokes: {stroke_count}");
    println!(
        "  UBO bytes: {} (fixed 800B, texture upload = 0)",
        ubo_bytes.len()
    );
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2-capable browser to view UBO-based analytical rendering.");
}
