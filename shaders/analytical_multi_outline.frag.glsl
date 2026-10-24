#version 300 es
// ALICE-Font S6.3.3 v0.6.7 — instanced multi-outline fragment shader
//
// vGlyphIdx (flat varying from vertex) で glyph 特定、flat UBO index で
// per-glyph の point / contour 情報を参照、winding number test で
// per-pixel inside/outside 判定。
//
// v0.6.7: MAX_CONTOURS_PER_GLYPH 12 → 16 (護 = 13 contours が truncate されて
// いた事案を解消。BIZ UDPGothic JIS L1 3,344 char 実測で 護 (13) が唯一の
// 12 超過、16 cap で完全カバー)。
//
// UBO layout (Rust `MultiOutlineGlyphPayload::to_std140_ubo_chunks` と一致):
//   - uPoints[768]: 4 glyph × 384 point / 2 per vec4 = flat packed
//   - uContourInfos[64]: 4 × 16 vec4 = per-contour (start_f, count_f, 0, 0)
//     (start = 該当 glyph 内 point index、後で glyphIdx * 384 で offset 加算)
//   - uGlyphMetas[4]: (advance, contour_count_f, x_offset, 0)
//   - uTextMeta: (total_advance, chunk_glyph_count, 0, 0)
//
// Per-fragment cost: MAX_CONTOURS_PER_GLYPH × MAX_POINTS_PER_GLYPH = 16 × 384
// = 6,144 op 上限 / fragment (v0.6.4 の 4,608 から 1.33x 増、全 BIZ UDPGothic
// JIS L1 kanji が truncate なしで完全描画可能)。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

precision highp float;

const int MAX_GLYPHS = 4;
const int MAX_POINTS_PER_GLYPH = 384;
const int MAX_CONTOURS_PER_GLYPH = 16;

layout(std140) uniform MultiOutlineBlock {
    vec4 uPoints[MAX_GLYPHS * MAX_POINTS_PER_GLYPH / 2];
    vec4 uContourInfos[MAX_GLYPHS * MAX_CONTOURS_PER_GLYPH];
    vec4 uGlyphMetas[MAX_GLYPHS];
    vec4 uTextMeta;
};

uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

flat in int vGlyphIdx;
in vec2 vUV;
out vec4 fragColor;

// Packed vec4 から global index (0..1023) の point (vec2) を取得。
vec2 getPointGlobal(int globalIdx) {
    vec4 packed = uPoints[globalIdx / 2];
    return ((globalIdx & 1) == 0) ? packed.xy : packed.zw;
}

float sdSegment(vec2 p, vec2 a, vec2 b) {
    vec2 ba = b - a;
    vec2 pa = p - a;
    float baLen2 = dot(ba, ba);
    if (baLen2 < 1e-12) {
        return length(pa);
    }
    float t = clamp(dot(pa, ba) / baLen2, 0.0, 1.0);
    return length(pa - ba * t);
}

float cross2d(vec2 a, vec2 b) {
    return a.x * b.y - a.y * b.x;
}

void main() {
    vec2 p = vUV;
    int glyphIdx = vGlyphIdx;
    vec4 meta = uGlyphMetas[glyphIdx];
    int contourCount = int(meta.y);

    // 該当 glyph の point base offset (flat array 内)
    int pointBase = glyphIdx * MAX_POINTS_PER_GLYPH;
    // 該当 glyph の contour info base
    int contourBase = glyphIdx * MAX_CONTOURS_PER_GLYPH;

    int wn = 0;
    float minDist = 1e10;

    for (int c = 0; c < MAX_CONTOURS_PER_GLYPH; c++) {
        if (c >= contourCount) {
            break;
        }
        vec4 info = uContourInfos[contourBase + c];
        int localStart = int(info.x); // glyph 内 point index
        int count = int(info.y);
        if (count < 3) {
            continue;
        }

        vec2 first = getPointGlobal(pointBase + localStart);
        vec2 prev = first;
        for (int i = 1; i < MAX_POINTS_PER_GLYPH; i++) {
            if (i >= count) {
                break;
            }
            vec2 curr = getPointGlobal(pointBase + localStart + i);
            if (prev.y <= p.y) {
                if (curr.y > p.y && cross2d(curr - prev, p - prev) > 0.0) {
                    wn++;
                }
            } else {
                if (curr.y <= p.y && cross2d(curr - prev, p - prev) < 0.0) {
                    wn--;
                }
            }
            minDist = min(minDist, sdSegment(p, prev, curr));
            prev = curr;
        }
        // Close contour
        if (prev.y <= p.y) {
            if (first.y > p.y && cross2d(first - prev, p - prev) > 0.0) {
                wn++;
            }
        } else {
            if (first.y <= p.y && cross2d(first - prev, p - prev) < 0.0) {
                wn--;
            }
        }
        minDist = min(minDist, sdSegment(p, prev, first));
    }

    float sd = (wn != 0) ? -minDist : minDist;
    float coverage = 1.0 - smoothstep(-uAntiAlias, uAntiAlias, sd);
    fragColor = mix(uBgColor, uFgColor, coverage);
}
