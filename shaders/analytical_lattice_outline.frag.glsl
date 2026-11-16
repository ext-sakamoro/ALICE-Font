#version 300 es
// ALICE-Font: glyph outlines drawn on a deformable lattice (fragment shader)
//
// Pairs with analytical_lattice_outline.vert.glsl. The glyph point of a pixel is computed as in
// analytical_multi_outline.frag.glsl, from the pixel's centre with the same expressions (q below),
// and then moved by the lattice: with lD = the barycentric coordinates of q in the deformed triangle
// and lR = those of q in the rest triangle, the rest point is q + (R·lD - R·lR). R·lD is the point of
// the rest glyph that the deformation carried to this pixel; R·lR reproduces q in the same arithmetic,
// so the difference is the deformation's displacement at q, and at rest (D = R, bit for bit) the two
// products are the same expression of the same numbers and the difference is exactly 0: the pixel is
// evaluated at q, as the plain path does. Outside the glyph box (in rest coordinates) the pixel is
// discarded, as the plain path does. The outline evaluation below the rest point is the plain
// shader's, byte for byte (a test in `outline::lattice` keeps the two copies equal).
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

uniform vec2 uCanvasSize; // the viewport's size, pixels
uniform float uYPadding;  // em above and below the glyph box
uniform vec2 uViewOrigin; // the viewport's lower left corner in the framebuffer, pixels (0 for a canvas of its own)
uniform vec4 uFgColor;
uniform vec4 uBgColor;
uniform float uAntiAlias;

flat in int vGlyphIdx;
flat in vec4 vD01;
flat in vec4 vD2R0;
flat in vec4 vR12;
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

// the point with barycentric coordinates of `q` in the triangle (a, b, c), carried to the triangle
// (ra, rb, rc); `ok` is false when (a, b, c) has no area
vec2 carry(vec2 q, vec2 a, vec2 b, vec2 c, vec2 ra, vec2 rb, vec2 rc, out bool ok) {
    float den = cross2d(b - a, c - a);
    ok = abs(den) > 0.0;
    float l1 = cross2d(q - a, c - a) / den;
    float l2 = cross2d(b - a, q - a) / den;
    float l0 = 1.0 - l1 - l2;
    return ra * l0 + rb * l1 + rc * l2;
}

void main() {
    int glyphIdx = vGlyphIdx;
    vec4 meta = uGlyphMetas[glyphIdx];
    // the glyph-box point of this pixel's centre, as the plain fragment shader computes it
    float w = uCanvasSize.x;
    float h = uCanvasSize.y;
    vec2 at = gl_FragCoord.xy - uViewOrigin;
    float nx = at.x / w * 2.0 - 1.0;
    float ny = 1.0 - (h - at.y) / h * 2.0;
    float emH = 1.0 + 2.0 * uYPadding;
    float totalAdv = uTextMeta.x;
    float canvasAspect = w / max(h, 1.0);
    float glyphAspect = totalAdv / emH;
    float sX = 1.0;
    float sY = 1.0;
    if (glyphAspect >= canvasAspect) {
        sY = canvasAspect / glyphAspect;
    } else {
        sX = glyphAspect / canvasAspect;
    }
    float v0 = (ny / sY + 1.0) * 0.5;
    float xEm = (nx / sX + 1.0) * 0.5 * totalAdv;
    float advance = meta.x;
    float u0 = (xEm - meta.z) / advance;
    vec2 q = vec2(u0, v0);
    vec2 d0 = vD01.xy;
    vec2 d1 = vD01.zw;
    vec2 d2 = vD2R0.xy;
    vec2 r0 = vD2R0.zw;
    vec2 r1 = vR12.xy;
    vec2 r2 = vR12.zw;
    bool okD;
    bool okR;
    vec2 fromD = carry(q, d0, d1, d2, r0, r1, r2, okD);
    vec2 fromR = carry(q, r0, r1, r2, r0, r1, r2, okR);
    if (!okD || !okR) {
        discard;
    }
    vec2 rest = q + (fromD - fromR);
    float u = rest.x;
    float v = rest.y;
    // outside this glyph's box: another glyph (or nothing) owns the pixel
    if (advance <= 0.0 || u < 0.0 || u >= 1.0 || v < 0.0 || v >= 1.0) {
        discard;
    }
    vec2 p = vec2(u, v * emH - uYPadding);
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
