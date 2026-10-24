# ALICE-Font GLSL shaders

S6.3.2b.3 milestone deliverable: **analytical fragment-shader path** with
**instanced rendering** for runtime-texture-free multi-glyph rendering.

「データを送るな、法則を送れ」ROADMAP core value の直接実証。

## Files

| file | 用途 |
|---|---|
| `analytical_glyph.frag.glsl` | 単一 kanji analytical Bezier + pen envelope (v3、std140 UBO) |
| `analytical_multi_glyph.vert.glsl` | Multi-kanji vertex shader (gl_InstanceID → per-glyph NDC) |
| `analytical_multi_glyph.frag.glsl` | Multi-kanji fragment shader (UBO 内 glyph 配列 flat index) |
| **`analytical_outline.vert.glsl`** | **v0.6.0 outline path vertex (fullscreen quad)** |
| **`analytical_outline.frag.glsl`** | **v0.6.0 outline path fragment (winding number test)** |
| **`analytical_multi_outline.vert.glsl`** | **v0.6.1 multi-outline vertex (gl_InstanceID → per-glyph NDC)** |
| **`analytical_multi_outline.frag.glsl`** | **v0.6.1 multi-outline fragment (flat UBO winding number)** |

## Version history

- **v3.1** (v0.5.3, S6.3.2b.3): 新 shader ペア `analytical_multi_glyph.
  {vert,frag}.glsl` 追加、`drawArraysInstanced` + `MultiGlyphBlock` UBO で
  N glyph 単一 draw call rendering (per-glyph draw loop 廃止)。
- **v3** (v0.5.2, S6.3.2b): `layout(std140) uniform GlyphBlock` に切替。
  Rust `to_std140_bytes` から直接 upload、uniform arrays 廃止。
- **v2** (v0.5.1, S6.3.2): pen 半幅 t-nearest 補間 (`mix(w0, w1, t)`)。
- **v1** (v0.5.0, S6.3.1): 単純 polyline + pen 平均 (`0.5 * (w0 + w1)`)。

## v3 std140 UBO layout (Rust `to_std140_bytes` と bit-exact 一致)

**Total UBO size = 800 byte fixed** (stroke 数によらず一定)。

```glsl
struct Stroke {
    vec4 p01;    // (p0.x, p0.y, p1.x, p1.y) — 16B
    vec4 p23;    // (p2.x, p2.y, p3.x, p3.y) — 16B
    vec4 widths; // (half_width_start, half_width_end, 0, 0) — 16B
};  // 48 byte / stroke (std140 spec 準拠、vec4-grouped)

layout(std140) uniform GlyphBlock {
    Stroke uStrokes[16];  //  0..768 B (48 × 16)
    vec4 uPen;             // 768..784 B: (base_width, contrast, pen_angle, roundness)
    vec4 uMeta;            // 784..800 B: (advance, stroke_count as float, 0, 0)
};
```

### Rust ↔ GLSL 対応表

| Rust field | GLSL member | 型 | Offset (byte) |
|---|---|---|---|
| `strokes[i].p0` | `uStrokes[i].p01.xy` | vec2 → vec4 の xy | i × 48 + 0 |
| `strokes[i].p1` | `uStrokes[i].p01.zw` | vec2 → vec4 の zw | i × 48 + 8 |
| `strokes[i].p2` | `uStrokes[i].p23.xy` | vec2 → vec4 の xy | i × 48 + 16 |
| `strokes[i].p3` | `uStrokes[i].p23.zw` | vec2 → vec4 の zw | i × 48 + 24 |
| `strokes[i].half_width_start` | `uStrokes[i].widths.x` | float | i × 48 + 32 |
| `strokes[i].half_width_end` | `uStrokes[i].widths.y` | float | i × 48 + 36 |
| (未使用 stroke slot) | (zero-fill) | — | 実 stroke 以降 |
| `pen.base_width` | `uPen.x` | float | 768 |
| `pen.contrast` | `uPen.y` | float | 772 |
| `pen.pen_angle` | `uPen.z` | float | 776 |
| `pen.roundness` | `uPen.w` | float | 780 |
| `advance` | `uMeta.x` | float | 784 |
| `strokes.len() as f32` | `uMeta.y` | float | 788 |

**strokes.len() > 16 の場合**: Rust 側は `debug_assert_eq!` で catch、実際
は `MAX_GLYPH_STROKES = 16` 制約により起きない。

## Non-UBO uniforms (色 + AA、per-draw 更新用)

| GLSL uniform | 型 | 用途 |
|---|---|---|
| `uFgColor` | vec4 | glyph 前景色 (RGBA) |
| `uBgColor` | vec4 | 背景色 (RGBA) |
| `uAntiAlias` | float | AA 幅 (em 単位、~0.003 が典型) |

## Algorithm

1. **Polyline 近似**: Cubic Bezier を 16 sample に分割、隣接 sample の
   line segment min-distance を SDF として採用 + nearest global t を保持。
2. **Variable pen envelope**: `mix(half_width_start, half_width_end, t)` で
   nearest-t 線形補間 (v0.5.1 → v0.5.2 で pen average 廃止)。
3. **Union**: 全 stroke の SDF を min で合成 (glyph = 全 stroke の和集合)。
4. **AA**: `smoothstep(-uAntiAlias, +uAntiAlias, sdf)` で band coverage。

## Reference

- Inigo Quilez "Distance to Bezier": <https://iquilezles.org/articles/distfunctions2d/>
- OpenGL 4.5 spec §7.6.2.2 (std140 layout rules)
- ALICE-Font `src/shader.rs` docs (Rust 側 payload + `to_std140_bytes`)
- ALICE-Font `examples/analytical_shader_poc/` (WebGL2 UBO harness、single)
- ALICE-Font `examples/analytical_shader_multi_poc/` (WebGL2 UBO harness、multi)

## Payload size proof

`AnalyticalGlyphPayload::to_std140_bytes()` は **fixed 800 byte / glyph**。
bitmap / texture upload は 0 byte。

| 例 | strokes | UBO size |
|---|---|---|
| 森 (v3) | 12 → padded to 16 slots | 800 B |
| 明 (v3) | 10 → padded to 16 slots | 800 B |
| 森明林 (v3、3 glyph) | 3 × 800B UBO | 2400 B |
| 日月火水木金土 (v3、7 glyph) | 7 × 800B UBO | 5600 B |

UBO 固定 800B の理由: GLSL 側 uniform block layout が静的サイズ、Rust 側
`MAX_GLYPH_STROKES = 16` 上限が SDF-generator 由来 hardcoded。未使用 slot
は zero-fill され、shader は `uMeta.y` = actual stroke_count で loop 上限
を制御する。

## MultiGlyphBlock UBO layout (v3.1、S6.3.2b.3)

`shaders/analytical_multi_glyph.{vert,frag}.glsl` で共有される UBO。
Rust `MultiGlyphPayload::to_std140_ubo_chunks` の 1 chunk と bit-exact 一致。

**Total UBO size = 12,816 byte fixed per chunk** (glyph 数 < 16 でも一定)。

```glsl
struct GlyphData {
    Stroke strokes[16];  // 768B
    vec4 pen;             // 16B: base_width, contrast, pen_angle, roundness
    vec4 meta;            // 16B: (advance, stroke_count_f, x_offset, 0)
};  // 800B / glyph

layout(std140) uniform MultiGlyphBlock {
    GlyphData uGlyphs[16];  //   0..12,800 B (800 × 16、未使用 slot は zero-fill)
    vec4 uTextMeta;          // 12,800..12,816 B: (total_advance, chunk_glyph_count_f, 0, 0)
};
```

### 主要差分 (GlyphBlock vs MultiGlyphBlock)

| 項目 | GlyphBlock (v3、single) | MultiGlyphBlock (v3.1、multi) |
|---|---|---|
| Struct 数 | 1 glyph (uStrokes[16] flat) | 16 glyph slots (uGlyphs[16] nested) |
| UBO size | 800 B | 12,816 B |
| meta.z 用途 | 0 (未使用) | **x_offset** (per-glyph text 内配置) |
| uTextMeta 有無 | なし (single glyph) | あり (total_advance + chunk instance count) |
| Draw call | 1 quad | 1 `drawArraysInstanced` (N glyph) |

### Instanced rendering flow

1. Rust: `payload.to_std140_ubo_chunks()` → `Vec<Vec<u8>>` (each 12,816B)
2. JS: per-chunk に UBO を `bufferSubData` 書き換え + `bindBufferBase(0, ubo)`
3. `drawArraysInstanced(TRIANGLE_STRIP, 0, 4, instance_count)` で 1 call
4. Vertex shader: `gl_InstanceID` で `uGlyphs[i].meta` を参照、per-instance NDC を計算
5. Fragment shader: `vGlyphIdx` (flat varying) で `uGlyphs[vGlyphIdx].strokes` を dynamic index

### chunk 分割の仕様

- ≤ 16 glyph text: 1 chunk = 1 draw call
- 17 ~ 32 glyph text: 2 chunks = 2 draw calls
- ceil(N / 16) chunks で任意長 text 対応
- text 全体の `total_advance` は全 chunk で共有 (vertex shader の NDC 計算基準)

### Draw call 削減効果 (v0.5.2 → v0.5.3)

| Text | glyph_count | draws v0.5.2 (per-glyph) | draws v0.5.3 (instanced) | 削減 |
|---|---|---|---|---|
| 森明林 | 3 | 3 | 1 | -67% |
| 日月火水木金土 | 7 | 7 | 1 | -86% |
| 16 glyph text | 16 | 16 | 1 | -94% |
| 17 glyph text | 17 | 17 | 2 | -88% |
| 32 glyph text | 32 | 32 | 2 | -94% |

## Roadmap

- **S6.3.1** ✓ (v0.5.0) — single-glyph uniform arrays
- **S6.3.2** ✓ (v0.5.1) — multi-kanji + pen t-nearest + std140 Rust prep
- **S6.3.2b** ✓ (v0.5.2) — UBO GLSL 完全統合、fixed 800B / glyph、
  Rust `to_std140_bytes` を JS Uint8Array 経由で `bufferData` upload
- **S6.3.2b.3** ✓ (v0.5.3) — **instanced rendering**、`drawArraysInstanced` +
  `MultiGlyphBlock` UBO で N glyph 単一 draw call、chunk 分割で任意長 text 対応
- **S6.3.2c** pending — radical library UBO packing、fragment shader が
  IDS 解決 + shared radical UBO 参照して stroke 重複削減
- **S6.3.3** pending — MetaFontParams runtime morph (push-constant / uniform)
