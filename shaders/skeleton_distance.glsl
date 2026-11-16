// ALICE-Font — the parametric skeleton as a distance (GLSL ES 3.00, no `#version` and no `precision`: the
// including shader states both). One source for the skeleton's text shader and for any scene that needs the
// glyph's distance (a groove cut along it, a field the physics reads): the arithmetic is the same in both.
//
// Rust twin: `skeleton_shader::skeleton_stroke` / `skeleton_distance` (the same operations in the same order,
// on the same std140 numbers).
//
// UBO layout (Rust `MultiSkeletonGlyphPayload::to_std140_ubo_chunks`):
//   - uStrokes[192]: 4 glyph × 16 stroke × 3 vec4, flat
//     stroke i: vec4[i*3+0] = (p0.x, p0.y, p1.x, p1.y)
//               vec4[i*3+1] = (p2.x, p2.y, p3.x, p3.y)
//               vec4[i*3+2] = (width_start, width_end, 0, 0)
//   - uGlyphMetas[4]: (advance, stroke_count_f, x_offset, default_pen)
//   - uTextMeta: (total_advance, chunk_glyph_count, 0, 0)
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

const int MAX_GLYPHS = 4;
const int MAX_STROKES_PER_GLYPH = 16;
const int VEC4_PER_STROKE = 3; // v0.7.2: 2 → 3 (+ width vec4)
const int BEZIER_SAMPLES = 12;

layout(std140) uniform MultiSkeletonBlock {
    vec4 uStrokes[MAX_GLYPHS * MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE];
    vec4 uGlyphMetas[MAX_GLYPHS];
    vec4 uTextMeta;
};

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

// How many strokes glyph `glyphIdx` of the bound chunk has
int skeletonStrokeCount(int glyphIdx) {
    return int(uGlyphMetas[glyphIdx].y);
}

// Stroke `s` of glyph `glyphIdx` seen from `p` (the glyph's em units): (the distance to its centre line, its pen
// half-width there). The pen width is interpolated along the stroke at the closest point's t; a width left at 0
// takes the glyph's default pen
vec2 skeletonStroke(vec2 p, int glyphIdx, int s) {
    float defaultPen = uGlyphMetas[glyphIdx].w;
    int base = glyphIdx * MAX_STROKES_PER_GLYPH * VEC4_PER_STROKE + s * VEC4_PER_STROKE;
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
    return vec2(d, penW);
}

// The glyph's signed distance at `p` (em units): negative inside its strokes, the least over its strokes of the
// distance to the stroke's centre line less the pen there. 1e10 for a glyph with no strokes
float skeletonDistance(vec2 p, int glyphIdx) {
    int strokeCount = skeletonStrokeCount(glyphIdx);
    float best = 1e10;
    for (int s = 0; s < MAX_STROKES_PER_GLYPH; s++) {
        if (s >= strokeCount) {
            break;
        }
        vec2 dw = skeletonStroke(p, glyphIdx, s);
        best = min(best, dw.x - dw.y);
    }
    return best;
}
