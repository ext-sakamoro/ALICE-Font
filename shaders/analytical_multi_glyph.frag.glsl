#version 300 es
// ALICE-Font S6.3.2b.4 — instanced multi-glyph fragment shader (v0.5.4 flat UBO)
//
// v0.5.3 nested `uGlyphs[vGlyphIdx].strokes[i]` の 2 段 dynamic indexing が
// 一部 GPU で不動作 (multi-glyph 描画壊れ)、v0.5.4 で flat `uStrokes[glyphIdx *
// MAX_STROKES + i]` に置換。indexing は 1 段 arithmetic のみ。
//
// UBO layout は Rust `MultiGlyphPayload::to_std140_ubo_chunks` (v0.5.4) と
// bit-exact 一致 (12,816 byte / chunk fixed、内部 layout のみ変更)。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

precision highp float;

const int MAX_STROKES = 16;
const int POLYLINE_STEPS = 16;
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
    vec4 uTextMeta;                              // 16 B
};

// Non-UBO uniforms (色 / AA、全 glyph 共通)。
uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

flat in int vGlyphIdx;
in vec2 vUV;
out vec4 fragColor;

// ===========================================================================
// SDF primitives (single-glyph shader と同一 logic)
// ===========================================================================

vec2 sdSegmentWithT(vec2 p, vec2 a, vec2 b) {
    vec2 ba = b - a;
    vec2 pa = p - a;
    float baLen2 = dot(ba, ba);
    if (baLen2 < 1e-12) {
        return vec2(length(pa), 0.0);
    }
    float t = clamp(dot(pa, ba) / baLen2, 0.0, 1.0);
    return vec2(length(pa - ba * t), t);
}

vec2 bezierPos(vec2 A, vec2 B, vec2 C, vec2 D, float t) {
    float mt = 1.0 - t;
    float mt2 = mt * mt;
    float mt3 = mt2 * mt;
    float t2 = t * t;
    float t3 = t2 * t;
    return mt3 * A + 3.0 * mt2 * t * B + 3.0 * mt * t2 * C + t3 * D;
}

vec2 sdBezierPolylineWithT(vec2 p, vec2 A, vec2 B, vec2 C, vec2 D) {
    float minDist = 1e10;
    float minT = 0.0;
    vec2 prev = A;
    for (int i = 1; i <= POLYLINE_STEPS; i++) {
        float tHi = float(i) / float(POLYLINE_STEPS);
        float tLo = float(i - 1) / float(POLYLINE_STEPS);
        vec2 curr = bezierPos(A, B, C, D, tHi);
        vec2 segRes = sdSegmentWithT(p, prev, curr);
        if (segRes.x < minDist) {
            minDist = segRes.x;
            minT = mix(tLo, tHi, segRes.y);
        }
        prev = curr;
    }
    return vec2(minDist, minT);
}

float sdStroke(vec2 p, Stroke s) {
    vec2 A = s.p01.xy;
    vec2 B = s.p01.zw;
    vec2 C = s.p23.xy;
    vec2 D = s.p23.zw;
    float w0 = s.widths.x;
    float w1 = s.widths.y;
    vec2 res = sdBezierPolylineWithT(p, A, B, C, D);
    float d = res.x;
    float t = res.y;
    float w = mix(w0, w1, t);
    return d - w;
}

// ===========================================================================
// Main
// ===========================================================================

void main() {
    // uMetas[vGlyphIdx].meta.y = stroke_count as float
    int strokeCount = int(uMetas[vGlyphIdx].meta.y);

    // Flat stroke array indexing: base offset = vGlyphIdx * MAX_STROKES
    int strokeBase = vGlyphIdx * MAX_STROKES;

    float minSdf = 1e10;
    for (int i = 0; i < MAX_STROKES; i++) {
        if (i >= strokeCount) {
            break;
        }
        // 1 段 dynamic index (arithmetic sum の runtime index)。
        minSdf = min(minSdf, sdStroke(vUV, uStrokes[strokeBase + i]));
    }

    float coverage = 1.0 - smoothstep(-uAntiAlias, uAntiAlias, minSdf);
    fragColor = mix(uBgColor, uFgColor, coverage);
}
