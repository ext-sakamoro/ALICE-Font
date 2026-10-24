#version 300 es
// ALICE-Font S6.3.3 — outline (polygon contour) fragment shader (v0.6.0)
//
// TTF-derived polygon outline を winding number test で per-fragment に
// inside/outside 判定、nearest-edge distance で AA。
//
// UBO layout は Rust `OutlineGlyphPayload::to_std140_bytes` と bit-exact
// 一致 (2,256 byte fixed):
//   - vec4 uPoints[128] (256 point packed 2 per vec4、2,048B)
//   - vec4 uContourInfo[16] (start_f, count_f, 0, 0、256B、v0.6.7)
//   - vec4 uOutlineMeta (contour_count_f, total_points_f, advance, 0、16B)
//
// Per-fragment cost: MAX_CONTOURS × MAX_POINTS = 16 × 256 = 4,096 op 上限 (v0.6.7)。
// 実 kanji (e.g. 三 12 point、森 ~60 point) は数 × 10-100 op で十分軽い。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

precision highp float;

const int MAX_POINTS = 256;
const int MAX_CONTOURS = 16;

layout(std140) uniform OutlineBlock {
    vec4 uPoints[MAX_POINTS / 2];   // 128 vec4 = 2,048B (2 point/vec4 packed)
    vec4 uContourInfo[MAX_CONTOURS]; // 16 vec4 = 256B (start_f, count_f, 0, 0)、v0.6.7
    vec4 uOutlineMeta;               // (contour_count_f, total_points_f, advance, 0)
};

uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

in vec2 vUV;
out vec4 fragColor;

// Packed vec4 から index i の point (vec2) を取得。
vec2 getPoint(int i) {
    vec4 packed = uPoints[i / 2];
    return ((i & 1) == 0) ? packed.xy : packed.zw;
}

// Point-to-segment distance (2D)。edge distance for AA。
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

// Cross product z component (for winding number test)。
float cross2d(vec2 a, vec2 b) {
    return a.x * b.y - a.y * b.x;
}

void main() {
    vec2 p = vUV;
    int contourCount = int(uOutlineMeta.x);

    // Winding number test (non-zero fill rule) + 最小 edge distance for AA。
    int wn = 0;
    float minDist = 1e10;

    for (int c = 0; c < MAX_CONTOURS; c++) {
        if (c >= contourCount) {
            break;
        }
        vec4 info = uContourInfo[c];
        int start = int(info.x);
        int count = int(info.y);
        if (count < 3) {
            continue;
        }

        vec2 first = getPoint(start);
        vec2 prev = first;
        for (int i = 1; i < MAX_POINTS; i++) {
            if (i >= count) {
                break;
            }
            vec2 curr = getPoint(start + i);
            // Winding number contribution (edge prev -> curr)
            if (prev.y <= p.y) {
                if (curr.y > p.y && cross2d(curr - prev, p - prev) > 0.0) {
                    wn++;
                }
            } else {
                if (curr.y <= p.y && cross2d(curr - prev, p - prev) < 0.0) {
                    wn--;
                }
            }
            // Edge distance for AA
            minDist = min(minDist, sdSegment(p, prev, curr));
            prev = curr;
        }
        // Close the contour: last edge (prev -> first)
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

    // Signed distance: negative = inside, positive = outside。
    float sd = (wn != 0) ? -minDist : minDist;
    float coverage = 1.0 - smoothstep(-uAntiAlias, uAntiAlias, sd);
    fragColor = mix(uBgColor, uFgColor, coverage);
}
