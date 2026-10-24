#version 300 es
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
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

precision highp float;

const int MAX_GLYPHS = 4;
const int MAX_STROKES_PER_GLYPH = 16;
const int VEC4_PER_STROKE = 3; // v0.7.2: 2 → 3 (+ width vec4)
const int BEZIER_SAMPLES = 12;

layout(std140) uniform MultiSkeletonBlock {
    vec4 uStrokes[MAX_GLYPHS * MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE];
    vec4 uGlyphMetas[MAX_GLYPHS];
    vec4 uTextMeta;
};

uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

flat in int vGlyphIdx;
in vec2 vUV;
out vec4 fragColor;

vec2 bezierAt(vec2 p0, vec2 p1, vec2 p2, vec2 p3, float t) {
    float u = 1.0 - t;
    float u2 = u * u;
    float u3 = u2 * u;
    float t2 = t * t;
    float t3 = t2 * t;
    return u3 * p0 + 3.0 * u2 * t * p1 + 3.0 * u * t2 * p2 + t3 * p3;
}

// Return (min_dist, best_t) — polyline approximation with t tracking。
vec2 sdBezierApproxT(vec2 p, vec2 p0, vec2 p1, vec2 p2, vec2 p3) {
    float minD = 1e10;
    float bestT = 0.0;
    vec2 prev = p0;
    for (int i = 1; i <= BEZIER_SAMPLES; i++) {
        float ti = float(i) / float(BEZIER_SAMPLES);
        vec2 curr = bezierAt(p0, p1, p2, p3, ti);
        vec2 ba = curr - prev;
        vec2 pa = p - prev;
        float baLen2 = dot(ba, ba);
        float segT = 0.0;
        float d;
        if (baLen2 < 1e-12) {
            d = length(pa);
        } else {
            segT = clamp(dot(pa, ba) / baLen2, 0.0, 1.0);
            d = length(pa - ba * segT);
        }
        if (d < minD) {
            minD = d;
            // Bezier t at this closest point = mix of prev sample's t and curr's t
            float tPrev = float(i - 1) / float(BEZIER_SAMPLES);
            bestT = mix(tPrev, ti, segT);
        }
        prev = curr;
    }
    return vec2(minD, bestT);
}

void main() {
    vec2 p = vUV;
    int glyphIdx = vGlyphIdx;
    vec4 meta = uGlyphMetas[glyphIdx];
    int strokeCount = int(meta.y);
    float defaultPen = meta.w;

    int strokeBase = glyphIdx * MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE;

    float maxCoverage = 0.0;
    float aa = uAntiAlias;

    for (int s = 0; s < MAX_STROKES_PER_GLYPH; s++) {
        if (s >= strokeCount) {
            break;
        }
        // Fetch 3 vec4 for this stroke (v0.7.2)
        int base = strokeBase + s * VEC4_PER_STROKE;
        vec4 v01 = uStrokes[base + 0];
        vec4 v23 = uStrokes[base + 1];
        vec4 vw = uStrokes[base + 2]; // (w_start, w_end, 0, 0)
        vec2 p0 = v01.xy;
        vec2 p1 = v01.zw;
        vec2 p2 = v23.xy;
        vec2 p3 = v23.zw;
        float wStart = vw.x;
        float wEnd = vw.y;
        // Fallback: if widths are 0 (uninitialized), use defaultPen
        if (wStart < 1e-6) wStart = defaultPen;
        if (wEnd < 1e-6) wEnd = defaultPen;

        vec2 result = sdBezierApproxT(p, p0, p1, p2, p3);
        float d = result.x;
        float t = result.y;
        float penW = mix(wStart, wEnd, t);

        float cov = 1.0 - smoothstep(penW - aa, penW + aa, d);
        maxCoverage = max(maxCoverage, cov);
    }

    fragColor = mix(uBgColor, uFgColor, maxCoverage);
}
