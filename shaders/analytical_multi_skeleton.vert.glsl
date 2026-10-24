#version 300 es
// ALICE-Font S6.4 v0.7.2-alpha — parametric skeleton vertex shader
//
// gl_InstanceID → per-glyph NDC 配置。UBO 内 uGlyphMetas[i] から advance +
// x_offset を参照、canvas layout uniform (uCanvasSize + uYPadding) で
// aspect-preserving fit + descender Y padding (v0.6.4 outline path と同 pattern)。
//
// UBO layout は Rust `MultiSkeletonGlyphPayload::to_std140_ubo_chunks` と
// bit-exact 一致 (**3,152 byte / chunk fixed** in v0.7.2-alpha、4 glyph × 16 stroke × 3 vec4)。
//
// v0.6.x outline path が polygon data を requires したのに対し、本 shader は
// **procedurally generated Bezier skeleton** を評価する zero-asset path。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

const int MAX_GLYPHS = 4;
const int MAX_STROKES_PER_GLYPH = 16;
const int VEC4_PER_STROKE = 3; // v0.7.2: 2 → 3 (+ width vec4)

layout(std140) uniform MultiSkeletonBlock {
    // Section 1: 4 glyph × 16 stroke × 3 vec4 = 192 vec4 (= 3,072B)
    // stroke i の vec4[i*3+0] = (p0.x, p0.y, p1.x, p1.y)
    // stroke i の vec4[i*3+1] = (p2.x, p2.y, p3.x, p3.y)
    // stroke i の vec4[i*3+2] = (width_start, width_end, 0, 0) v0.7.2-alpha
    vec4 uStrokes[MAX_GLYPHS * MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE];
    // Section 2: per-glyph meta = (advance, stroke_count_f, x_offset, pen_width)
    vec4 uGlyphMetas[MAX_GLYPHS];
    // Section 3: uTextMeta = (total_advance, chunk_glyph_count, 0, 0)
    vec4 uTextMeta;
};

// v0.6.4 → v0.7.0: canvas layout uniforms (aspect-fit + descender padding)
uniform vec2 uCanvasSize; // pixel dimensions (W, H)
uniform float uYPadding;  // em units above/below glyph baseline

in vec2 aQuadPos;
flat out int vGlyphIdx;
out vec2 vUV;

void main() {
    int idx = gl_InstanceID;
    vec4 meta = uGlyphMetas[idx];
    float advance = meta.x;
    float xOff = meta.z;
    float totalAdv = uTextMeta.x;

    vec2 uv = aQuadPos * 0.5 + 0.5;
    vGlyphIdx = idx;

    // Descender/ascender: expand vUV.y to [-uYPadding, 1 + uYPadding]
    vUV = vec2(uv.x, uv.y * (1.0 + 2.0 * uYPadding) - uYPadding);

    // Aspect-preserving fit (v0.6.4 と同一)
    float totalEmW = totalAdv;
    float glyphEmH = 1.0 + 2.0 * uYPadding;
    float canvasAspect = uCanvasSize.x / max(uCanvasSize.y, 1.0);
    float glyphAspect = totalEmW / glyphEmH;

    float sX;
    float sY;
    if (glyphAspect >= canvasAspect) {
        sX = 1.0;
        sY = canvasAspect / glyphAspect;
    } else {
        sX = glyphAspect / canvasAspect;
        sY = 1.0;
    }

    float emX = ((xOff + uv.x * advance) / totalEmW) * 2.0 - 1.0;
    float emY = aQuadPos.y;
    gl_Position = vec4(emX * sX, emY * sY, 0.0, 1.0);
}
