//! ALICE-Font S6.3.3 example — outline (polygon contour) shader PoC (v0.6.0)。
//!
//! ```sh
//! cargo run --example analytical_outline_poc --features serde -- three /tmp/three.html
//! cargo run --example analytical_outline_poc --features serde -- ten /tmp/ten.html
//! cargo run --example analytical_outline_poc --features serde -- box /tmp/box.html
//! ```
//!
//! v0.6.0 (S6.3.3): TTF-derived polygon outline を fragment shader が winding
//! number test で per-pixel evaluation する path。**本 example は synthetic
//! outline (三/十/囗) のみを提供**、実 BIZ UDPGothic outline data は
//! outline provider (L1) 経由で consumer に inject する design。
//!
//! # L0/L1 分離
//!
//! - L0 (本 crate): outline shader path 提供 (data 非依存)
//! - L1 (outline provider): `regular_outline(ch)` 等 data provider
//! - Consumer: L0 + L1 を組み合わせて実 BIZ UDPGothic を render
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Sample outline shapes: synthetic、not derived from any copyrighted font.
//! Author: Moroya Sakamoto

use alice_font::outline::{pack_outline_for_shader, OutlineGlyph, OUTLINE_UBO_SIZE};
use std::io::Write;

const TEMPLATE_HTML: &str = include_str!("template.html");
const VERT_SHADER: &str = include_str!("../../shaders/analytical_outline.vert.glsl");
const FRAG_SHADER: &str = include_str!("../../shaders/analytical_outline.frag.glsl");

/// Synthetic sample outlines (not from any copyrighted font)。
fn sample_outline(name: &str) -> Option<OutlineGlyph> {
    match name {
        // 三 = 3 horizontal rectangles
        "three" => Some(OutlineGlyph {
            codepoint: '三',
            contours: vec![
                vec![[0.15, 0.75], [0.85, 0.75], [0.85, 0.85], [0.15, 0.85]],
                vec![[0.20, 0.45], [0.80, 0.45], [0.80, 0.55], [0.20, 0.55]],
                vec![[0.10, 0.15], [0.90, 0.15], [0.90, 0.25], [0.10, 0.25]],
            ],
            advance: 1.0,
        }),
        // 十 = 1 vertical rectangle + 1 horizontal rectangle
        "ten" => Some(OutlineGlyph {
            codepoint: '十',
            contours: vec![
                // Horizontal bar
                vec![[0.10, 0.45], [0.90, 0.45], [0.90, 0.55], [0.10, 0.55]],
                // Vertical bar
                vec![[0.45, 0.10], [0.55, 0.10], [0.55, 0.90], [0.45, 0.90]],
            ],
            advance: 1.0,
        }),
        // 囗 = box with hole (outer contour + inner hole)
        "box" => Some(OutlineGlyph {
            codepoint: '囗',
            contours: vec![
                // Outer boundary
                vec![[0.10, 0.10], [0.90, 0.10], [0.90, 0.90], [0.10, 0.90]],
                // Inner hole (reverse winding order for hole)
                vec![[0.20, 0.80], [0.80, 0.80], [0.80, 0.20], [0.20, 0.20]],
            ],
            advance: 1.0,
        }),
        // 田 = rice paddy (grid with 4 holes)
        "rice" => Some(OutlineGlyph {
            codepoint: '田',
            contours: vec![
                // Outer boundary
                vec![[0.10, 0.10], [0.90, 0.10], [0.90, 0.90], [0.10, 0.90]],
                // Top-left hole
                vec![[0.15, 0.85], [0.48, 0.85], [0.48, 0.52], [0.15, 0.52]],
                // Top-right hole
                vec![[0.52, 0.85], [0.85, 0.85], [0.85, 0.52], [0.52, 0.52]],
                // Bottom-left hole
                vec![[0.15, 0.48], [0.48, 0.48], [0.48, 0.15], [0.15, 0.15]],
                // Bottom-right hole
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
            "usage: cargo run --example analytical_outline_poc --features serde -- <sample> <output.html>"
        );
        eprintln!("samples: three | ten | box | rice");
        std::process::exit(1);
    }

    let name = &args[1];
    let out_path = &args[2];

    let outline = sample_outline(name).unwrap_or_else(|| {
        eprintln!("error: unknown sample {name:?}、valid: three | ten | box | rice");
        std::process::exit(2);
    });

    let payload = pack_outline_for_shader(&outline).unwrap_or_else(|| {
        eprintln!(
            "error: pack_outline_for_shader failed for {}",
            outline.codepoint
        );
        std::process::exit(3);
    });

    let ubo_bytes = payload.to_std140_bytes();
    assert_eq!(ubo_bytes.len(), OUTLINE_UBO_SIZE);
    let ubo_json = serde_json::to_string(&ubo_bytes).expect("serde_json");

    let bb = payload.bounding_box;
    let html = TEMPLATE_HTML
        .replace("{{CHAR}}", &outline.codepoint.to_string())
        .replace("{{CHAR_HEX}}", &format!("{:04X}", outline.codepoint as u32))
        .replace("{{SAMPLE_NAME}}", name)
        .replace("{{TOTAL_POINTS}}", &payload.total_points().to_string())
        .replace("{{CONTOUR_COUNT}}", &payload.contour_count().to_string())
        .replace("{{UBO_BYTES}}", &ubo_bytes.len().to_string())
        .replace(
            "{{BOUNDING_BOX}}",
            &format!("[{:.3}, {:.3}, {:.3}, {:.3}]", bb[0], bb[1], bb[2], bb[3]),
        )
        .replace("{{VERT_SHADER}}", VERT_SHADER)
        .replace("{{FRAG_SHADER}}", FRAG_SHADER)
        .replace("{{UBO_BYTES_JSON}}", &ubo_json);

    let mut f = std::fs::File::create(out_path)
        .unwrap_or_else(|e| panic!("failed to create {out_path}: {e}"));
    f.write_all(html.as_bytes())
        .unwrap_or_else(|e| panic!("failed to write {out_path}: {e}"));

    println!("✓ generated {out_path}");
    println!(
        "  sample: {name} ({} U+{:04X})",
        outline.codepoint, outline.codepoint as u32
    );
    println!("  contours: {}", payload.contour_count());
    println!("  total points: {}", payload.total_points());
    println!(
        "  UBO bytes: {} (fixed 2,256B、texture upload = 0)",
        ubo_bytes.len()
    );
    println!("  html size: {} byte", html.len());
    println!();
    println!("Open {out_path} in a WebGL2-capable browser (outline shader path)。");
}
