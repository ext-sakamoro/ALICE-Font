#version 300 es
// ALICE-Font S6.3.2b — analytical cubic Bezier glyph fragment shader (v3 UBO)
//
// 「データを送るな、法則を送れ」ROADMAP core value 直接実証 (v0.5.2)。
// v0.5.0 (uniform arrays) → v0.5.1 (pen t-nearest) → v0.5.2 (UBO 完全統合)。
//
// UBO layout は Rust `alice_font::shader::AnalyticalGlyphPayload::to_std140_bytes`
// と bit-exact 一致 (800 byte fixed size):
//   - Stroke uStrokes[16] (48B × 16 = 768B)
//   - vec4 uPen (base_width, contrast, pen_angle, roundness、16B)
//   - vec4 uMeta (advance, stroke_count as float, 0, 0、16B)
//
// Reference:
// - Inigo Quilez "Distance to Bezier": https://iquilezles.org/articles/distfunctions2d/
//   polyline 近似 (16 subdivision) + pen 半幅 t-nearest 補間。
// - OpenGL std140 layout spec (GLSL 4.5 §7.6.2.2)。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

precision highp float;

// stroke 数の hardcoded 上限 (Rust MAX_GLYPH_STROKES と一致)。
const int MAX_STROKES = 16;

// Bezier polyline 分割数 (per-fragment cost = MAX_STROKES × POLYLINE_STEPS)。
const int POLYLINE_STEPS = 16;

// ===========================================================================
// std140 UBO block layout — Rust to_std140_bytes と bit-exact 一致
// ===========================================================================

// Stroke = 3 × vec4 = 48 byte (vec4-grouped、std140 pad なし)。
struct Stroke {
    vec4 p01;    // (p0.x, p0.y, p1.x, p1.y)
    vec4 p23;    // (p2.x, p2.y, p3.x, p3.y)
    vec4 widths; // (half_width_start, half_width_end, 0, 0)
};

layout(std140) uniform GlyphBlock {
    Stroke uStrokes[MAX_STROKES];  // 768 B
    vec4 uPen;                      // 16 B: (base_width, contrast, pen_angle, roundness)
    vec4 uMeta;                     // 16 B: (advance, stroke_count as float, 0, 0)
};
// Total UBO = 800 B、GL_MAX_UNIFORM_BLOCK_SIZE (最低保証 16 KB) の 5% 以下。

// Non-UBO uniforms (色 / AA、既存 example と互換維持)。
uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

in vec2 vUV;
out vec4 fragColor;

// ===========================================================================
// SDF primitives (v0.5.1 と同一 logic、UBO 経由の uniform 参照に切替のみ)
// ===========================================================================

/// Point-to-segment distance + segment 内 nearest tLocal ∈ [0,1]。
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

/// Cubic Bezier position at t ∈ [0, 1]。
vec2 bezierPos(vec2 A, vec2 B, vec2 C, vec2 D, float t) {
    float mt = 1.0 - t;
    float mt2 = mt * mt;
    float mt3 = mt2 * mt;
    float t2 = t * t;
    float t3 = t2 * t;
    return mt3 * A + 3.0 * mt2 * t * B + 3.0 * mt * t2 * C + t3 * D;
}

/// Cubic Bezier polyline 近似 SDF + nearest global t (return.x=dist, return.y=t)。
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

/// Cubic Bezier + variable-width pen envelope 全体の signed distance。
/// pen 半幅を nearest-t で線形補間 (S6.3.2 sub-goal 3)、UBO 経由 (v0.5.2)。
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
    vec2 uv = vUV;

    // uMeta.y = stroke_count as float (Rust 側で cast 済)。
    int strokeCount = int(uMeta.y);

    float minSdf = 1e10;
    for (int i = 0; i < MAX_STROKES; i++) {
        if (i >= strokeCount) {
            break;
        }
        minSdf = min(minSdf, sdStroke(uv, uStrokes[i]));
    }

    // AA band + fragment 出力。
    float coverage = 1.0 - smoothstep(-uAntiAlias, uAntiAlias, minSdf);
    fragColor = mix(uBgColor, uFgColor, coverage);
}
