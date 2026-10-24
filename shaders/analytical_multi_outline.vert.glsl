#version 300 es
// ALICE-Font S6.3.3 v0.6.7 — instanced multi-outline vertex shader
//
// gl_InstanceID → per-glyph NDC 配置。UBO 内 uGlyphMetas[i] から advance +
// x_offset を参照して text 上の位置を計算、vGlyphIdx flat varying で fragment
// に glyph index を渡す。
//
// v0.6.4: aspect-preserving fit + descender Y padding (uCanvasSize +
// uYPadding uniform)。canvas と glyph em box の aspect 乖離時 letterbox で
// 対応。fragment shader が em [0, 1] 前提を保つため、descender は vUV を
// [-uYPadding, 1+uYPadding] に expand して覆う。
//
// UBO layout は Rust `MultiOutlineGlyphPayload::to_std140_ubo_chunks` と
// bit-exact 一致 (**13,392 byte / chunk fixed** in v0.6.7、384 pt × 4 glyph × 16 contour)。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

const int MAX_GLYPHS = 4;
const int MAX_POINTS_PER_GLYPH = 384;
const int MAX_CONTOURS_PER_GLYPH = 16;

layout(std140) uniform MultiOutlineBlock {
    // Section 1: 4 glyph × 384 point / 2 per vec4 = 768 vec4 (= 12,288B)
    vec4 uPoints[MAX_GLYPHS * MAX_POINTS_PER_GLYPH / 2];
    // Section 2: 4 glyph × 16 contour info = 64 vec4 (= 1,024B、v0.6.7)
    vec4 uContourInfos[MAX_GLYPHS * MAX_CONTOURS_PER_GLYPH];
    // Section 3: per-glyph meta = (advance, contour_count_f, x_offset, 0)
    vec4 uGlyphMetas[MAX_GLYPHS];
    // Section 4: uTextMeta = (total_advance, chunk_glyph_count, 0, 0)
    vec4 uTextMeta;
};

// v0.6.4: canvas layout uniforms
uniform vec2 uCanvasSize; // pixel dimensions (W, H)
uniform float uYPadding;  // em units above/below glyph baseline for descender/ascender

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

    // v0.6.4: descender/ascender 用に vUV.y を [0, 1] → [-uYPadding, 1+uYPadding] へ expand
    // vUV.x は em [0, 1] 内 glyph 内座標のまま
    vUV = vec2(uv.x, uv.y * (1.0 + 2.0 * uYPadding) - uYPadding);

    // Text 全体の em box: 幅 totalAdv、高さ (1 + 2 × uYPadding)
    float totalEmW = totalAdv;
    float glyphEmH = 1.0 + 2.0 * uYPadding;

    // Aspect-preserving fit: canvas と em box の aspect 比較して letterbox 決定
    float canvasAspect = uCanvasSize.x / max(uCanvasSize.y, 1.0);
    float glyphAspect = totalEmW / glyphEmH;

    float sX;
    float sY;
    if (glyphAspect >= canvasAspect) {
        // em box が canvas より wide → 幅 fit、上下 letterbox
        sX = 1.0;
        sY = canvasAspect / glyphAspect;
    } else {
        // em box が canvas より tall → 高さ fit、左右 letterbox
        sX = glyphAspect / canvasAspect;
        sY = 1.0;
    }

    // Per-glyph NDC 配置 (em 内相対位置を em box 全体 [-1, +1] にマップ、
    // 後で sX で aspect-fit scale)
    float emX = ((xOff + uv.x * advance) / totalEmW) * 2.0 - 1.0;
    float emY = aQuadPos.y; // [-1, +1] は em box 全体の縦範囲
    gl_Position = vec4(emX * sX, emY * sY, 0.0, 1.0);
}
