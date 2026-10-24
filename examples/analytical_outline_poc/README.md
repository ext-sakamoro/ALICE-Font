# analytical_outline_poc

ALICE-Font **S6.3.3** milestone example — outline (polygon contour) shader
PoC (v0.6.0)。

TTF-derived polygon outline を fragment shader が winding number test で
per-pixel evaluation する path。runtime texture upload = **0 byte** 継続、
実 font glyph shape を law-only shipping で render。

## Usage

```sh
# Synthetic sample outlines (not from any copyrighted font)。
cargo run --example analytical_outline_poc --features serde -- three /tmp/three.html
cargo run --example analytical_outline_poc --features serde -- ten /tmp/ten.html
cargo run --example analytical_outline_poc --features serde -- box /tmp/box.html
cargo run --example analytical_outline_poc --features serde -- rice /tmp/rice.html

open /tmp/three.html  # macOS
```

## Sample outlines (synthetic)

- **three** (三): 3 horizontal rectangles (12 points, 3 contours)
- **ten** (十): 1 horizontal + 1 vertical bar (8 points, 2 contours)
- **box** (囗): outer boundary + inner hole (8 points, 2 contours)
- **rice** (田): outer boundary + 4 inner holes (20 points, 5 contours)

**注**: これらは synthetic 図形、実 BIZ UDPGothic / Noto Sans 等 copyrighted
font 由来ではない。実 book glyph rendering は outline provider (L1) 経由で
consumer が data を inject する design (下記 L0/L1 分離)。

## L0/L1 分離 design

- **L0 (ALICE-Font)** = `OutlineGlyph` API + `pack_outline_for_shader` +
  GLSL shader (`shaders/analytical_outline.{vert,frag}.glsl`)、data 非依存
- **L1 (outline provider)** = BIZ UDPGothic 等 TTF-derived outline data の
  provider (`biz_udpgothic::regular_outline(ch) -> OutlineGlyph`)
- **Consumer** = L0 + L1 両方 import、L1 で data 取得 → L0 で shader pack + render

## UBO layout

**Fixed 2,256 byte per glyph** (`shaders/analytical_outline.frag.glsl` と一致):
- Offset 0..2,048: `vec4 uPoints[128]` (256 point packed 2 per vec4)
- Offset 2,048..2,240: `vec4 uContourInfo[12]` (start_f, count_f, 0, 0)
- Offset 2,240..2,256: `vec4 uOutlineMeta` (contour_count_f, total_points_f, advance, 0)

## Shader algorithm

Fragment shader per-pixel:
1. Iterate 全 contour の 全 edge (最大 12 × 256 = 3,072 op)
2. 各 edge で winding number test contribution
3. 各 edge で nearest distance を追跡 (AA 用)
4. `wn != 0 → inside`、`nearest_dist → smoothstep(-aa, +aa)` で coverage
5. `mix(bg, fg, coverage)` で fragment output

## 関連

- `../../src/outline.rs` — Rust API 実装
- `../../shaders/analytical_outline.{vert,frag}.glsl` — GLSL shader
- `../../shaders/README.md` — shader 仕様
- CHANGELOG v0.6.0 — S6.3.3 milestone entry
