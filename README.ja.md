# ALICE-Font

[English](README.md) · [日本語](README.ja.md)

**手続き的な CJK フォント — atlas なし、TTF なし、bitmap なし。**

> 「データを送るな、法則を送れ。」— ALICE の根源原理

ALICE-Font は ALICE 全 crate に共通する根源原理
「データを送るな、法則を送れ」に従います。すべての glyph は
**runtime に手続き的に合成** されます — 送るのは 40 バイトの
`MetaFontParams`、部首の parametric stroke skeleton ライブラリ、
そして「どの漢字がどの部首でどう組立てられるか」を記述した IDS
分解ルールだけ。 runtime はこれらの法則を各 pixel で評価し
glyph を錬成します。 outline table の embed も、SDF atlas の
pre-bake も、`.ttf` / `.otf` の配布も一切ありません。

## なぜこれが重要か

これまでの「効率的 font」実装 — MSDF atlas、TinySDF、Slug、WOFF2、
Variable Fonts — はすべて **データ**を配ります: pixel texture、
Bezier outline table、glyph table。 ALICE-Font が配るのは **法則**
です: 任意の glyph を必要な瞬間に再構成する数式ルール。 結果、
payload は KB 単位で済み、生成された glyph は本質的に解像度非依存です。

## 配るもの

| Primitive | Payload (S6 完成時目標) | 役割 |
|-----------|--------------------------|------|
| `MetaFontParams` | **40 バイト** (10 × f32) | Style: weight, width, serif, contrast, slant, metrics, roundness |
| Radical stroke library | ~150 KB Rust const | 康熙 214 部首を parametric Bezier skeleton で表現 |
| IDS composition rules | ~30 KB | 各漢字を部首に分解する規則 (⿰⿱⿲⿴ …) |
| Runtime pen model | ~5 KB code | `MetaFontParams` で駆動される可変幅 Bezier pen |
| Layout engine | ~5 KB code | IDS operator → 部首の配置 transform |
| **常用漢字 2,136 全対応 renderer の総配送量** | **≈ 200 KB** | — |

比較: Noto Sans JP CJK TTF は 1 weight だけで ~7 MB あります。

## 3 primitive、Rust コード

```rust
use alice_font::{MetaFontParams, glyph_dispatcher};

// 40 バイトの parametric style
let regular = MetaFontParams::sans_regular();

// hiragana glyph が parametric stroke から合成される、texture 不要
let sdf = glyph_dispatcher::generate('あ', &regular);

// 同じ 40 バイトを morph
let bolder = MetaFontParams { weight: 0.18, ..regular };
let sdf_bold = glyph_dispatcher::generate('あ', &bolder);
```

CJK では同じ呼び出しが `cjk::ids_db` + `cjk::radicals` +
`cjk::layout` を経由します — 該当漢字の IDS 分解を lookup し、
各部首の stroke skeleton を scale + 配置、union を `GlyphSdf`
として返します。

## 意図的に *置いていない*もの

- **outline table の embed なし。** 以前 (`v0.3.x`) は BIZ UDPGothic の
  cubic Bezier outline 座標を ~1 MB の Rust const として同梱していました。
  これは crate 境界を経由した「data smuggling」であり、ALICE の
  根源原理に反していました。 `v0.4.0` で削除。歴史 implementation は
  [`archive/sdf-atlas-approach`](https://github.com/ext-sakamoro/ALICE-Font/tree/archive/sdf-atlas-approach)
  branch に保全されています。
- **SDF atlas 生成器なし。** 以前の `tools/alice-font-atlas` CLI
  (TTF → multi-page SDF atlas) も同 archive branch。異なる問題
  (与えられた font を texture に圧縮する) を解いていたもので、
  ALICE-Font の問題 (法則から font を再構成、source TTF 自体不要)
  とは目的が違います。
- **`.ttf` / `.otf` の runtime 依存なし。** crate は手続き的な
  generator のみを公開します。

## Modules

| Module | 説明 |
|--------|------|
| [`param`] | `MetaFontParams` — 40 バイトの parametric style descriptor |
| [`stroke`] | Bezier skeleton curves 沿いに適用する可変幅 pen model |
| [`glyph`] | SDF glyph 手続き的生成 — ASCII, Hiragana, Katakana, CJK stroke primitive |
| [`cjk`] | IDS parser, radical library, 合成 layout, IDS DB |
| [`atlas`] | runtime の GPU 対応 SDF tile cache (手続き的出力の派生、配布 asset ではない) |
| [`shaper`] | text shaper — kerning, horizontal advance, line layout |
| [`game`] | `TextEffect` — outline / shadow / glow / gradient / dissolve / … の canonical spec |
| [`license`] | font-license tracking (per-title, platform, seat-limit、32-byte wire format) |

## 現在の状態 (v0.7.0-alpha)

- **S6.2.1 完了** — 教育漢字 grade 1-6 全 1,026 IDS 完了 (v0.4.6)
- **S6.3.1 完了** — fragment shader analytical Bezier distance PoC (v0.5.0)
- **S6.3.2 完了** — instanced multi-glyph text rendering + UBO packing (v0.5.4)
- **S6.3.3 完了** — polygon outline shader path (v0.6.0-v0.6.7):
  - Winding-number test in fragment shader、texture upload 0 byte
  - **384 pt × 4 glyph × 16 contour** per chunk、UBO 13,392B
  - Aspect-preserving canvas fit + descender Y padding (0.15 em)
  - L0/L1 分離: L0 = shader path のみ、L1 (outline provider) = BIZ UDPGothic
    outline data provider (JIS X 0208 Level 1、3,344 kanji)
- **S6.4 infrastructure** — parametric skeleton shader path (v0.7.0-alpha):
  - L0 単独完結 (radicals + IDS → Bezier skeleton → shader)
  - Zero-asset payload ~76 KB total (shader + UBO + HTML)
  - Content coverage 部分的 (73 / 1,053 KANJI_DB real render、残り placeholder)

## ライセンス

`LicenseRef-Proprietary` 条文は [LICENSE-Proprietary](LICENSE-Proprietary)

Enquiries: https://extoria.co.jp/

## Author

坂本師哉 (Moroya Sakamoto)
