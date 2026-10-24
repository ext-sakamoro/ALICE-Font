#version 300 es
// ALICE-Font S6.3.3 — outline (polygon contour) vertex shader (v0.6.0)
//
// Fullscreen quad で fragment に vUV (0..1 × 0..1) を渡すだけ。
// Outline data は fragment shader が UBO 経由で参照、winding number test を
// per-fragment に行う。
//
// SPDX-License-Identifier: LicenseRef-Proprietary
// Author: Moroya Sakamoto

in vec2 aQuadPos;   // [-1, 1] × [-1, 1]
out vec2 vUV;

void main() {
    vUV = aQuadPos * 0.5 + 0.5;
    gl_Position = vec4(aQuadPos, 0.0, 1.0);
}
