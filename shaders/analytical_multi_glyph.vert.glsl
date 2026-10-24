#version 300 es
// ALICE-Font S6.3.2b.4 — instanced multi-glyph vertex shader (v0.5.4 flat UBO)
//
// v0.5.3 nested `uGlyphs[i].strokes[j]` の 2 段 dynamic indexing が一部 GPU で
// 不動作 (multi-glyph 描画壊れ、user report 2026-07-30)。v0.5.4 で flat 化:
// `uMetas[i]` は 1 段 dynamic index、fragment shader 側 `uStrokes[glyphIdx * 16
// + strokeIdx]` も flat arithmetic index に。
//
// UBO layout は Rust `MultiGlyphPayload::to_std140_ubo_chunks` (v0.5.4) と
// bit-exact 一致 (12,816 byte / chunk fixed、内部 layout のみ変更)。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

const int MAX_STROKES = 16;
const int MAX_GLYPHS = 16;

// Flat UBO layout (v0.5.4)。
struct Stroke {
    vec4 p01;
    vec4 p23;
    vec4 widths;
};
struct GlyphMeta {
    vec4 pen;    // (base_width, contrast, pen_angle, roundness)
    vec4 meta;   // (advance, stroke_count as float, x_offset, 0)
};

layout(std140) uniform MultiGlyphBlock {
    Stroke uStrokes[MAX_GLYPHS * MAX_STROKES];  // 256 slots = 12,288 B
    GlyphMeta uMetas[MAX_GLYPHS];                // 16 × 32 = 512 B
    vec4 uTextMeta;                              // 16 B: (total_advance, chunk_glyph_count, 0, 0)
};

in vec2 aQuadPos;   // [-1, 1] × [-1, 1] 基準 quad
flat out int vGlyphIdx;
out vec2 vUV;

void main() {
    int idx = gl_InstanceID;
    // uMetas[idx].meta = (advance, stroke_count_f, x_offset, 0)
    vec4 meta = uMetas[idx].meta;
    float advance = meta.x;
    float xOff = meta.z;
    float totalAdv = uTextMeta.x;

    vec2 uv = aQuadPos * 0.5 + 0.5;
    vUV = uv;
    vGlyphIdx = idx;

    // NDC.x: text 全体 [0, totalAdv] を [-1, 1] に mapping。
    float ndcX = ((xOff + uv.x * advance) / totalAdv) * 2.0 - 1.0;
    gl_Position = vec4(ndcX, aQuadPos.y, 0.0, 1.0);
}
