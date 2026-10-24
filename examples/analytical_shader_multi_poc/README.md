# analytical_shader_multi_poc

ALICE-Font **S6.3.2** milestone example — multi-kanji analytical
fragment-shader PoC。

`analytical_shader_poc` (S6.3.1) の拡張、複数 kanji を **1 canvas で一
表示** し、per-glyph draw call + transform uniform で並列 rendering。
runtime texture upload = **0 byte** は継続。

## Usage

```sh
# Serde feature を有効にして example を実行 (JSON serialization 用)。
cargo run --example analytical_shader_multi_poc --features serde -- "森明林" text.html
cargo run --example analytical_shader_multi_poc --features serde -- "森羅万象" text2.html
cargo run --example analytical_shader_multi_poc --features serde -- "日月火水木金土" week.html

# 生成 HTML をブラウザで開く。
open text.html  # macOS
```

## 出力

- **単一 standalone HTML** (~30 KB): shader source + JSON payload (N glyph)
  + JS harness を inline embed
- Canvas 幅は `total_advance` に応じて 512..2048 px 可変、高さ 512 px 固定
- 画面 top に metadata 表示 (`glyph_count`, `payload_bytes`,
  `total_advance`, `texture upload = 0 byte`)

## Rendering flow

1. `pack_string_for_shader(text, params)` → `MultiGlyphPayload`
   {glyphs: [...], layout: [{x_offset, advance}], total_advance}
2. `serde_json::to_string_pretty(&payload)` → JSON 文字列を HTML embed
3. `include_str!("../../shaders/analytical_glyph.frag.glsl")` で shader を
   binary に compile-time embed
4. Browser 側 JS: 各 glyph について
   - shader uniforms を upload (stroke data + pen)
   - NDC quad を per-glyph 位置 (`layout.x_offset / total_advance`) に更新
   - `drawArrays(TRIANGLE_STRIP, 4)`
5. Per-fragment に shader v2 が analytical Bezier + pen 半幅 t-nearest 補間
   を評価、smoothstep AA で coverage 出力

## Rendering fidelity (S6.3.2 improvements)

- **pen 半幅 t-nearest 補間** (`mix(w0, w1, t)`): 現行 average 近似から
  改善、細り端 / 太り端の thickness が直感通り遷移
- **未登録 kanji skip**: text 内の未登録 char (例: 龥、hiragana) は自動
  skip され、登録済 glyph だけ詰めた layout になる

## 実装フロー例 (森明林 の場合)

```
"森明林"
  → pack_string_for_shader
    → MultiGlyphPayload {
        glyphs: [<森 payload>, <明 payload>, <林 payload>],
        layout: [
          {x_offset: 0.0, advance: 1.0},
          {x_offset: 1.0, advance: 1.0},
          {x_offset: 2.0, advance: 1.0},
        ],
        total_advance: 3.0,
      }
  → HTML 生成 (JSON embed + shader inline)
  → Browser JS loop:
      draw 森 in canvas x=[0, 0.333], quad NDC=[-1, -0.333]
      draw 明 in canvas x=[0.333, 0.667], quad NDC=[-0.333, 0.333]
      draw 林 in canvas x=[0.667, 1.0], quad NDC=[0.333, 1]
```

## 関連

- `../analytical_shader_poc/` — S6.3.1 single-glyph 版
- `../../shaders/analytical_glyph.frag.glsl` — GLSL shader v2 (pen t-nearest)
- `../../shaders/README.md` — shader 仕様
- `src/shader.rs` — Rust 側 `MultiGlyphPayload` + `pack_string_for_shader` 実装
- CHANGELOG v0.5.1 — S6.3.2 milestone entry
