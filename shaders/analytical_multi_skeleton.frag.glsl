// ALICE-Font S6.4 v0.7.2-alpha — parametric skeleton fragment shader (variable-width pen)
//
// v0.7.2-alpha 変更: per-stroke variable pen width (書道 taper 表現)。
// 各 stroke に (width_start, width_end) を UBO 3rd vec4 で受取り、
// Bezier t (closest point) で `mix(w_start, w_end, t)` interpolate。
//
// vGlyphIdx (flat varying from vertex) で glyph 特定、UBO から 16 strokes を
// 読取り、per-pixel で each stroke の cubic Bezier に対する variable-width
// pen SDF を評価。coverage は max across strokes (union)。
//
// UBO layout (Rust `MultiSkeletonGlyphPayload::to_std140_ubo_chunks` v0.7.2):
//   - uStrokes[192]: 4 glyph × 16 stroke × 3 vec4 = flat packed
//     stroke i の vec4[i*3+0] = (p0.x, p0.y, p1.x, p1.y)
//     stroke i の vec4[i*3+1] = (p2.x, p2.y, p3.x, p3.y)
//     stroke i の vec4[i*3+2] = (width_start, width_end, 0, 0) ← v0.7.2 追加
//   - uGlyphMetas[4]: (advance, stroke_count_f, x_offset, default_pen)
//   - uTextMeta: (total_advance, chunk_glyph_count, 0, 0)
//
// Per-fragment cost: 16 stroke × 12 polyline sample × (dist + t track) =
// ~250 op 上限 / fragment (v0.7.0-v0.7.1 の 192 op から +30%、taper 表現分)。
//
// Zero-asset property: uStrokes は Rust が radical + IDS 法則から
// procedurally 生成した Bezier control points + variable widths、
// texture upload 0 byte。
//
// `#version 300 es`, `precision highp float;` and shaders/skeleton_distance.glsl (the UBO block, the stroke
// distance) come before this file: `skeleton_shader::MULTI_SKELETON_FRAG_GLSL` puts them together.
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

flat in int vGlyphIdx;
in vec2 vUV;
out vec4 fragColor;

void main() {
    vec2 p = vUV;
    int glyphIdx = vGlyphIdx;
    int strokeCount = skeletonStrokeCount(glyphIdx);

    float maxCoverage = 0.0;
    float aa = uAntiAlias;

    for (int s = 0; s < MAX_STROKES_PER_GLYPH; s++) {
        if (s >= strokeCount) {
            break;
        }
        // (the distance to the stroke's centre line, its pen there): shaders/skeleton_distance.glsl
        vec2 dw = skeletonStroke(p, glyphIdx, s);
        float d = dw.x;
        float penW = dw.y;

        float cov = 1.0 - smoothstep(penW - aa, penW + aa, d);
        maxCoverage = max(maxCoverage, cov);
    }

    fragColor = mix(uBgColor, uFgColor, maxCoverage);
}
