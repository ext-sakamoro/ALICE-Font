#version 300 es
// ALICE-Font: glyph outlines drawn on a deformable lattice (vertex shader)
//
// Pairs with analytical_lattice_outline.frag.glsl and reads the same MultiOutlineBlock as
// analytical_multi_outline.vert.glsl. The geometry is not one quad per glyph but the triangles of
// `outline::lattice::lattice_triangles` (one draw, no instancing). Every vertex carries all three
// vertices of its triangle in the glyph's box coordinates (u across the advance, v up the em box,
// both 0..1 at rest): deformed (D), at rest (R), and each one's outward direction on the lattice's
// border (O, -1 / 0 / +1 per axis). A border vertex is rasterized one pixel further out, as the plain
// path grows its quad, so a pixel whose centre lies on the glyph box's edge is rasterized. The
// triangle passed to the fragment shader is the ungrown one (D and R as given): its barycentric map
// extends past its edges unchanged, so the band is mapped as the triangle maps its inside, and the
// fragment shader decides what to draw from the pixel's rest point.
//
// The fit of the text's em box into the canvas is the one of analytical_multi_outline.vert.glsl.
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

const int MAX_GLYPHS = 4;
const int MAX_POINTS_PER_GLYPH = 384;
const int MAX_CONTOURS_PER_GLYPH = 16;

layout(std140) uniform MultiOutlineBlock {
    vec4 uPoints[MAX_GLYPHS * MAX_POINTS_PER_GLYPH / 2];
    vec4 uContourInfos[MAX_GLYPHS * MAX_CONTOURS_PER_GLYPH];
    vec4 uGlyphMetas[MAX_GLYPHS];
    vec4 uTextMeta;
};

uniform vec2 uCanvasSize; // pixel dimensions (W, H)
uniform float uYPadding;  // em above and below the glyph box

in vec2 aSlot;  // (glyph slot in the chunk 0..3, which of the triangle's vertices this one is 0..2)
in vec4 aD01;   // D0.xy, D1.xy
in vec4 aD2R0;  // D2.xy, R0.xy
in vec4 aR12;   // R1.xy, R2.xy
in vec4 aO01;   // O0.xy, O1.xy
in vec2 aO2;    // O2.xy

flat out int vGlyphIdx;
flat out vec4 vD01;
flat out vec4 vD2R0;
flat out vec4 vR12;

void main() {
    int idx = int(aSlot.x + 0.5);
    int which = int(aSlot.y + 0.5);
    vec4 meta = uGlyphMetas[idx];
    float advance = meta.x;
    float xOff = meta.z;
    float totalAdv = uTextMeta.x;
    float glyphEmH = 1.0 + 2.0 * uYPadding;
    float canvasAspect = uCanvasSize.x / max(uCanvasSize.y, 1.0);
    float glyphAspect = totalAdv / glyphEmH;
    float sX = 1.0;
    float sY = 1.0;
    if (glyphAspect >= canvasAspect) {
        sY = canvasAspect / glyphAspect;
    } else {
        sX = glyphAspect / canvasAspect;
    }
    // one pixel, in the glyph's box coordinates
    vec2 px = vec2(totalAdv / (max(uCanvasSize.x, 1.0) * sX * max(advance, 1e-30)),
                   1.0 / (max(uCanvasSize.y, 1.0) * sY));
    vGlyphIdx = idx;
    vD01 = aD01;
    vD2R0 = aD2R0;
    vR12 = aR12;
    // this vertex, rasterized one pixel further out when it is on the border
    vec2 d = (which == 0) ? aD01.xy + aO01.xy * px
           : ((which == 1) ? aD01.zw + aO01.zw * px : aD2R0.xy + aO2 * px);
    float emX = ((xOff + d.x * advance) / totalAdv) * 2.0 - 1.0;
    float emY = d.y * 2.0 - 1.0;
    gl_Position = vec4(emX * sX, emY * sY, 0.0, 1.0);
}
