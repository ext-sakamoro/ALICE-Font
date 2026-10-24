# analytical_shader_poc

ALICE-Font **S6.3.1** milestone example — analytical fragment-shader PoC。

「データを送るな、法則を送れ」ROADMAP core value を **standalone HTML** で
実証。生成された HTML は WebGL2 fragment shader 経由で任意の kanji を
render、bitmap / atlas / texture upload = **0 byte**。

## Usage

```sh
# Serde feature を有効にして example を実行 (JSON serialization 用)。
cargo run --example analytical_shader_poc --features serde -- 森 mori.html
cargo run --example analytical_shader_poc --features serde -- 明 mei.html

# 生成 HTML をブラウザで開く。
open mori.html  # macOS
# または: xdg-open mori.html  (Linux)
# または: start mori.html      (Windows)
```

## 出力

- **単一 standalone HTML** (~10-15 KB): shader source + JSON payload + JS
  harness を inline embed
- 画面 top に metadata 表示 (`payload_bytes`, `stroke_count`, `texture
  upload = 0 byte`)
- WebGL2 canvas に kanji が analytical rendering される

## 実装フロー

1. `pack_glyph_for_shader(ch, params)` → `AnalyticalGlyphPayload` (Rust)
2. `serde_json::to_string_pretty(&payload)` → JSON 文字列
3. `include_str!("../../shaders/analytical_glyph.frag.glsl")` で shader を
   binary に compile-time embed
4. `template.html` の `{{PLACEHOLDER}}` を JSON + shader + metadata に置換
5. output HTML を書き出す
6. ブラウザ側: JSON parse → uniform upload → fullscreen quad draw → per-fragment に fragment shader が analytical Bezier 距離を評価 → smoothstep AA で coverage 出力

## Rendering fidelity

- 単純化: Bezier polyline (16 subdivision) + pen 半幅 t 沿い平均近似
- 十分な視覚品質 (em 単位 ~1/256 精度)
- S6.3.2 で closed-form + t-nearest based pen width に refine 予定

## 関連

- `../shaders/analytical_glyph.frag.glsl` — GLSL shader source
- `../shaders/README.md` — shader 仕様 + uniform layout
- `src/shader.rs` — Rust 側 `pack_glyph_for_shader` 実装
- CHANGELOG v0.5.0 — S6.3.1 milestone entry
