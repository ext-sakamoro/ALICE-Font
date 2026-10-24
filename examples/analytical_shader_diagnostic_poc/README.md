# analytical_shader_diagnostic_poc

**S6.3.2b.4** diagnostic example — 単一 kanji を大 canvas + side panel + grid
overlay で render、shader compile log を UI 表示する debug view。

Multi-glyph 描画壊れの原因究明 (v0.5.4 flat UBO fix の validation) + shader
path の readability tuning に使用。

## Usage

```sh
cargo run --example analytical_shader_diagnostic_poc --features serde -- 三 /tmp/three.html
cargo run --example analytical_shader_diagnostic_poc --features serde -- 森 /tmp/mori_debug.html
open /tmp/three.html
```

## 表示情報

- **Main canvas** (512×512): kanji を single-glyph shader + `sans_hairline`
  preset で render、grid overlay + center cross 付き
- **Side panel**: stroke count / advance / pen params (base_width, contrast,
  pen_angle, roundness) の numeric + payload JSON dump
- **Shader log**: compile / link log (成功時 green、error 時 red で全文)

## 関連

- `../analytical_shader_poc/` — single-glyph 通常 example
- `../analytical_shader_multi_poc/` — multi-glyph instanced example
- `../../shaders/analytical_glyph.frag.glsl` — 使用 shader
- `../../src/shader.rs::pack_glyph_for_shader`
