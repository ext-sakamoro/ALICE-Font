#version 300 es
// ALICE-Font: glyph outlines drawn on a deformable lattice (fragment shader)
//
// Pairs with analytical_lattice_outline.vert.glsl. The glyph point of a pixel is computed as in
// analytical_multi_outline.frag.glsl, from the pixel's centre with the same expressions (q below). Its
// rest point r = q + c solves c + disp(q + c) = 0, disp being the Catmull-Rom displacement of the
// triangle's lattice cell (its 4 x 4 nodes from uDisp): the triangle's barycentric map gives the first
// guess (c = R lD - R lR, lD and lR the barycentric coordinates of q in the deformed and in the rest
// triangle), then NEWTON_STEPS Newton steps (a literal). The pixel is not drawn when the residual is above
// RESIDUAL_PX pixels or r lies outside the cell by more than half a sub-cell (a fold: the solve went to
// another layer of the sheet). At rest disp is 0 and both products are the same expression of the same
// numbers: c is exactly 0 and the pixel is evaluated at q, as the plain path does. The fragment's depth
// is 1 - coverage: drawn with a LEQUAL depth test, the fragment with the most ink wins a pixel where
// layers of a folded sheet or glyphs overlap (ink is not covered by another layer's empty area). Outside the glyph box
// (in rest coordinates) the pixel is discarded, as the plain path does. The outline evaluation below the
// rest point is the plain shader's, byte for byte (a test in `outline::lattice` keeps the two copies equal),
// and `outline::lattice::GlyphLattice::rest_point` is this computation on the CPU.
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
flat in vec2 vCell;
uniform highp sampler2D uDisp; // RG32F: (n + 2) wide, 4 (n + 2) high, glyph slot s in rows s (n + 2)..
uniform int uLatticeN;           // nodes per side
uniform int uLatticeK;           // cell subdivision

const int NEWTON_STEPS = 4;
const float RESIDUAL_PX = 0.015625;
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

vec4 catmullRom(float s) {
    float s2 = s * s;
    float s3 = s2 * s;
    return vec4((-s + 2.0 * s2 - s3) * 0.5, (2.0 - 5.0 * s2 + 3.0 * s3) * 0.5,
                (s + 4.0 * s2 - 3.0 * s3) * 0.5, (s3 - s2) * 0.5);
}

vec4 catmullRomD(float s) {
    float s2 = s * s;
    return vec4((-1.0 + 4.0 * s - 3.0 * s2) * 0.5, (-10.0 * s + 9.0 * s2) * 0.5,
                (1.0 + 8.0 * s - 9.0 * s2) * 0.5, (3.0 * s2 - 2.0 * s) * 0.5);
}

// the cell's displacement at (s, t) and its derivatives in s and t
void cellPatch(vec2 nd[16], float s, float t, out vec2 d, out vec2 ds, out vec2 dt) {
    vec4 ws = catmullRom(s);
    vec4 wt = catmullRom(t);
    vec4 dws = catmullRomD(s);
    vec4 dwt = catmullRomD(t);
    d = vec2(0.0);
    ds = vec2(0.0);
    dt = vec2(0.0);
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            vec2 p = nd[y * 4 + x];
            float w = ws[x] * wt[y];
            float wu = dws[x] * wt[y];
            float wv = ws[x] * dwt[y];
            d.x += w * p.x;
            d.y += w * p.y;
            ds.x += wu * p.x;
            ds.y += wu * p.y;
            dt.x += wv * p.x;
            dt.y += wv * p.y;
        }
    }
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
    vec2 c = fromD - fromR;
    int ci = int(vCell.x + 0.5);
    int cj = int(vCell.y + 0.5);
    float hh = 1.0 / float(uLatticeN - 1);
    vec2 o = vec2(float(ci) * hh, float(cj) * hh);
    vec2 nd[16];
    for (int y = 0; y < 4; y++) {
        for (int x = 0; x < 4; x++) {
            nd[y * 4 + x] = texelFetch(uDisp, ivec2(ci + x, glyphIdx * (uLatticeN + 2) + cj + y), 0).xy;
        }
    }
    vec2 d;
    vec2 ds;
    vec2 dt;
    for (int it = 0; it < NEWTON_STEPS; it++) {
        vec2 r = q + c;
        cellPatch(nd, (r.x - o.x) / hh, (r.y - o.y) / hh, d, ds, dt);
        vec2 g = c + d;
        float a = 1.0 + ds.x / hh;
        float b = dt.x / hh;
        float e = ds.y / hh;
        float f = 1.0 + dt.y / hh;
        float det = a * f - b * e;
        if (abs(det) > 0.0) {
            c.x -= (f * g.x - b * g.y) / det;
            c.y -= (a * g.y - e * g.x) / det;
        }
    }
    vec2 rest = q + c;
    cellPatch(nd, (rest.x - o.x) / hh, (rest.y - o.y) / hh, d, ds, dt);
    vec2 g = c + d;
    // one pixel in the glyph's box coordinates, as the vertex shader computes it
    vec2 px = vec2(totalAdv / (max(w, 1.0) * sX * max(advance, 1e-30)), 1.0 / (max(h, 1.0) * sY));
    if (!(abs(g.x) <= RESIDUAL_PX * px.x && abs(g.y) <= RESIDUAL_PX * px.y)) {
        discard;
    }
    float m = hh / (2.0 * float(uLatticeK));
    if (rest.x < o.x - m || rest.x > o.x + hh + m || rest.y < o.y - m || rest.y > o.y + hh + m) {
        discard;
    }
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
    // the most ink wins where layers or glyphs overlap: depth 1 - coverage under a LEQUAL test keeps the
    // fragment of the largest coverage (the colour is a function of coverage alone, so equal coverages give
    // the same colour and the draw order does not matter)
    gl_FragDepth = 1.0 - coverage;
}
