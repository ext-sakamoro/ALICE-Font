# analytical_outline_multi_poc

ALICE-Font **v0.6.1 S6.3.3** example — instanced multi-outline text rendering。

`drawArraysInstanced` + flat UBO (9,872B fixed / chunk) で multi-glyph outline
text を 1 chunk = 1 draw call で render。任意長 text は chunk 分割 (≤ 8
glyph/chunk)。

## Usage

```sh
cargo run --example analytical_outline_multi_poc --features serde -- "三十囗田" /tmp/text.html
cargo run --example analytical_outline_multi_poc --features serde -- "三十三十" /tmp/repeat.html
open /tmp/text.html
```

Supported synthetic samples: **三 / 十 / 囗 / 田**。実 BIZ UDPGothic は
outline provider (L1) 経由で inject する design (別 example)。

## UBO Layout (flat、1 段 dynamic index)

**Total 9,872 byte per chunk** (`shaders/analytical_multi_outline.frag.glsl` と一致):
- Offset 0..8,192: `vec4 uPoints[512]` (flat 8 glyph × 128 point / 2 per vec4)
- Offset 8,192..9,728: `vec4 uContourInfos[96]` (flat 8 × 12 contour info)
- Offset 9,728..9,856: `vec4 uGlyphMetas[8]` (advance, contour_count_f, x_offset, 0)
- Offset 9,856..9,872: `vec4 uTextMeta` (total_advance, chunk_glyph_count, 0, 0)

Fragment shader access: `uPoints[glyphIdx * 128 + localIdx]` = 1 段 arithmetic
index (v0.5.4 で得た flat indexing 設計を outline path にも適用)。

## Draw call 削減効果

| Text | glyph_count | chunks | draws | 削減 |
|---|---|---|---|---|
| 三十 | 2 | 1 | 1 | -50% |
| 三十囗田 | 4 | 1 | 1 | -75% |
| 三十三十三十三十 | 8 | 1 | 1 | -88% |
| 三十囗田三十囗田三 | 9 | 2 | 2 | -78% |

## 関連

- `../analytical_outline_poc/` — single-glyph outline (v0.6.0)
- `../../src/outline.rs::pack_outline_string` — Rust API
- `../../shaders/analytical_multi_outline.{vert,frag}.glsl` — GLSL shaders
