# Changelog

All notable changes to ALICE-Font will be documented in this file.

## [Unreleased]

### Added

- `outline::rasterize(&OutlineGlyph) -> GlyphSdf`: contour から `GLYPH_SDF_SIZE`² の符号付き距離 (em) を作る 塗りは non-zero winding、距離は塗った領域の境界まで (重なった contour の内側の辺は使わない)、`f64` と `alice_det_math::ops::sqrt64` で計算
- `outline::TILE_EM_LEFT` / `TILE_EM_RIGHT` / `TILE_EM_BOTTOM` / `TILE_EM_TOP` (tile の範囲 0.0 / 1.0 / -0.2 / 0.9)
- 依存 `alice-det-math` (git)、`deny.toml` の allow-git に ALICE-DetMath
- 試験 `tests/outline_raster.rs` (箱の閉じた式、手で書いた和集合の多角形への総当たり、穴、重なり、接する contour、出力の bit の digest)

### Changed

- Licence: `AGPL-3.0-only OR LicenseRef-Commercial` → `LicenseRef-Proprietary`
- `publish = false`、authors からアドレスを削除
- `LICENSE-AGPL` / `LICENSE-COMMERCIAL.md` / `LICENSE-MIT-HISTORICAL` / `LICENSES/BIZUDPGothic-OFL.txt` を削除し `LICENSE-Proprietary` を追加、`TRADEMARK_NOTICE` を変更
- CI: package integrity と cargo-semver-checks の job を `security-audit.yml` から dispatch 専用の `package-integrity.yml` に移動
- CI: 全 feature (pyo3 を含む) の clippy を Python 3.12 に固定した job で実行
- CI: ci.yml の ALICE-SDF の checkout を削除し、checkout を repository の root に置く
- CI: outline raster の試験 (`tests/outline_raster.rs`、出力の digest の golden を含む) を Linux x86_64 / arm64・macOS・Windows の matrix で実行
- CI: ci.yml / fuzz.yml / security-audit.yml を `ci/**` の push でも実行
- CI: 全 job を GitHub-hosted の runner (`ubuntu-24.04-arm`、fmt / actionlint / audit は `ubuntu-latest`) で実行し、`vars.CI_RUNS_ON` の参照と `pull_request` の trigger を削除
- source / 例 / shader / binding の license の行を `SPDX-License-Identifier: LicenseRef-Proprietary` に変更
- 試験の名前を `registered_count_matches_seed_extract` に変更
- comment と例と script の記述を変更 (非公開の repository の名前と手元の path を削除)

### Removed

- `ROADMAP.md` / `CONTRIBUTING.md` / `docs/SDF_FONT_APPROACHES.md` / `docs/CJK_ROADMAP.md` / `docs/IDS_DATA_SOURCES.md`
- README の approaches と Roadmap の節
- `.github/dependabot.yml`
- `scripts/capture-media-v2.mjs` の capture 対象 1 件
- `.gitignore` の手元専用の行

## [0.7.6] - 2026-09-17 — crates.io 初 publish

### Added
- `tests/analytic_oracle.rs` — 閉形式 / 独立参照との突合 oracle 14 本: 直線 stroke の position / tangent / arc_length (steps 1..1024 で不変) / cubic arc_length の f64 Simpson 参照 + 単調性 / fast sqrt の Lomont bound / pen の broad-nib 閉形式 (Bhaskara bound 由来) / slant = shear / wire round-trip + lerp 端点 / rasterize SDF = capsule 閉形式 + union = min + Lipschitz + 平行移動不変、`sdf` feature で Font → alice-sdf の契約 (union = min / sampling bound) を CI 専用 step で実行 (`Sdf2dNode::Bezier` の閉形式一致は alice-sdf 側 17 点 sampling のため `#[ignore]`、SDF 側修正待ち)

### Fixed
- `GlyphGenerator` rasterize の距離法則: 17 sample **点** への距離の min + 「最初に負になった sample で break」だった → 16 **線分** (capsule chain、半幅は線分上で lerp) の min に 旧法則は sample 間で最大 L/32 の過大評価 (stroke 縁で符号反転)、内部値が最小でなく最初の負値 (半幅 1 個分ずれ、1-Lipschitz 破れ) — capsule / union-min / Lipschitz oracle で検出 直線 stroke + contrast 0 は閉形式 capsule に 1e-4 em 以内で一致
- `MetaFontParams::lerp` / `Point2::lerp` を `a·(1−t) + b·t` に (旧 `a + (b−a)·t` は `lerp(1) ≠ other`、roundness 0.3→0.1 が 0.099999994)
- fast inverse sqrt の doc claim「4.7e-7、f32 ulp 以内」→ 実測 4.6e-6 (≈ 77 ulp) に訂正

### Changed

- `sdf` feature: `alice-sdf` is a crates.io dependency (`2.x`) instead of a
  `path = "../ALICE-SDF"` dep; CI no longer builds a manifest-only alice-sdf
  stub, so `cargo audit` / `cargo deny` see the real dependency tree.

### Fixed
- clippy `manual_checked_div` 2 件 (examples `analytical_*_multi_poc` の削減率計算 → `checked_div`) と rustdoc の unresolved link 1 件 (`medial_axis::rasterize` の `bitmap[resolution²]`)、CI に clippy / doc が無く未検出

### Changed
- CI: fmt + `cargo check` のみ → test (default + `std,ffi,serde`) / clippy `--workspace --all-targets -D warnings` 2 variant / no_std は `continue-on-error` を撤去 (以前から green) + bare-metal `thumbv7em-none-eabihf` / `feature-powerset` (cargo-hack、std 固定 depth 2) / doc `-D warnings` を追加 `rust-toolchain.toml` に thumbv7em target

## [0.7.5] - 2026-07-31 — Font DSL problematic kanji direct definition (**alpha / preview**)

### Fixed — 5 problematic composed kanji visual quality (音 意 想 雲 電)

nested IDS
composition (top_bottom で 2-3 段 stacking) が visual density 過剰で潰れる。

**Root cause**:
- 音 = ⿱ 立(6) 日(5) を top/bottom half に squish → 立 の 6 stroke が 0.5 em
  高さに compress で potato ball 化
- 意 = ⿱ 音 心 の triple nested → さらに compression
- 想 = ⿱ 相 心 同上
- 雲 電 = 雨 (9 stroke) を top half に squish で内 4 dots 消える

**Fix**: 5 kanji を **直接 stroke 定義** に書き換え (nested composition 廃止):
- 音 → direct 11-stroke definition (立 top + 日 bottom manually placed)
- 意 → direct 13-stroke definition (立 + 日 + 心 3 段直接)
- 想 → direct 14-stroke definition (木 + 目 + 心 直接)
- 雲 → direct 11-stroke (雨 simplified + 云 substitute)
- 電 → direct 14-stroke (雨 simplified + 田 + 下 descender hook)

各 kanji は em box 内で strokes を manual placement、compression 回避、
書体密度改善。

### 手法 documentation (Approach 4)

「**Direct definition vs nested composition**」を Quality Improvement
Techniques Tier 1 に追記。単純 kanji (林 森 好 男) は nested OK、
複雑 kanji (音 意 想 雲 電) は direct definition 必須。

判定基準:
- Nested の各 sub-glyph 合計 stroke count ≤ 10 → nested OK
- ≥ 11 → direct definition 推奨

### 変更前後 stroke count

| kanji | v0.7.4 (nested) | v0.7.5 (direct) |
|---|---|---|
| 音 | 11 (立 6 + 日 5) | **11 direct** |
| 意 | 15 (音 11 + 心 4) | **13 direct** |
| 想 | 14 (相 10 + 心 4) | **14 direct** |
| 雲 | 11 (雨 9 + 二 2) | **11 direct** |
| 電 | 15 (雨 9 + 田 6) | **14 direct + 1 descender hook** |

### Tests + docs

- 24 font_dsl tests all pass
- 347 total tests pass、clippy 0/0、fmt clean
- CHANGELOG + ROADMAP + docs update

## [0.7.4] - 2026-07-31 — Font DSL Library Expansion (Approach 4 continuation、**alpha / preview**)

### Added — 35 new radicals + 11 new composed kanji

**v0.7.1-alpha 20 → v0.7.4-alpha 70 char (+50 char)**、DSL 経由の hand-crafted
Bezier stroke で全 real render (placeholder rectangle 0 件)。

**New radicals (35)**:
- **数字完成 (9)**: 四 五 六 七 八 九 百 千 万 (all 1-9 + 100/1000/10000)
- **方向・位置 (7)**: 上 下 中 前 後 左 右
- **自然 (10)**: 川 石 天 空 花 草 雨 雪 白 青
- **人体・動作 (5)**: 手 足 耳 見 走
- **構造 (4)**: 立 米 糸 車

**New composed kanji (11)**:
- 男 = ⿱ 田 力
- 好 = ⿰ 女 子
- 岩 = ⿱ 山 石
- 早 = ⿱ 日 十
- 音 = ⿱ 立 日
- 意 = ⿱ 音 心 (nested composition)
- 想 = ⿱ 相 心 (nested composition)
- 志 = ⿱ 十 心 (士 approximated by 十)
- 半 = ⿱ 八 十 (干 approximation)
- 雲 = ⿱ 雨 二 (云 approximation)
- 電 = ⿱ 雨 田 (电 approximation)

### Coverage totals (v0.7.4-alpha)

- Primitive radicals: **55** (up from 20)
- Composed kanji: **15** (up from 4)
- Total DSL-defined: **70 char** (up from 20 = 3.5x)
- All render as real Bezier strokes with variable-width pen (v0.7.2)

### Tests

- Existing `all_mvp_radicals_have_multi_strokes` expanded to 55 radical smoke test
- New `all_composed_kanji_have_valid_strokes` test (15 composed verification)
- 全 347 tests pass (346 前 + 1 new composed test)、clippy 0/0、fmt clean

### Position (Approach 4)

- **Content coverage**: 70 / 常用漢字 2,136 (~3.3%)、v0.7.1 の 1% から 3.3x 拡大
- **Session 2026-07-31 拡張履歴**: v0.7.1 (20 char) → v0.7.2 (variable-width pen) →
  v0.7.4 (70 char content expansion)
- **限界継続**: parametric radicals は書体品質に構造的限界、visual quality は
  v0.7.2 と同等 (書体プロポーション改善なし、DSL 手動 authoring の accuracy
  依存)
- **Long-term path**: 常用漢字 2,136 coverage には radical library を 214 まで拡張 +
  IDS decomposition 網羅、数百時間 authoring 必要

## [0.7.2] - 2026-07-31 — Variable-width pen for Font DSL (**alpha / preview**)

### Added — per-stroke variable-width pen (書道 taper 表現)

- `SkeletonGlyph.stroke_widths: [(f32, f32); MAX_GLYPH_STROKES]` per-stroke
  (width_start, width_end) 追加
- Fragment shader が Bezier closest-t を tracking、`mix(w_start, w_end, t)` で
  pen radius interpolate
- `sdBezierApproxT()` shader function 新設 (polyline distance + t 同時 track)
- Coverage: min-distance → max-coverage-across-strokes に変更 (variable width 対応)

### DSL enhancement

- `GlyphBuilder::add_variable(stroke, w_start, w_end)` メソッド追加
- 全 20 radical を variable-width 化:
  - 縦棒: heavy → tapered bottom (主幹)
  - 撇: heavy → tapered end (taper 表現)
  - 捺: thin → heavy end (書道 8th stroke 典型)
  - 横棒: 起筆 thin → 収筆 heavy
  - Dots: 起筆 round-heavy

### UBO layout change (**breaking**)

- `VEC4_PER_STROKE: 2 → 3` (追加 vec4 = width_start / width_end)
- `MULTI_SKELETON_STROKES_AREA_SIZE`: 2,048 → 3,072 byte
- `MULTI_SKELETON_UBO_SIZE`: **2,128 → 3,152 byte** fixed
- WebGL2 min UBO 使用率: 13.0% → 19.2% (余裕十分)

### user 視覚評価

- v0.7.1-alpha (uniform pen): 「ギリ読めるかどうかレベル」
- v0.7.2-alpha (variable pen): **「読みにくいよ」** — variable-width では未達
- 判定: parametric skeleton path は 書体品質に構造的限界 (radical 数百時間 authoring 必要)
- 次段: **Approach 6 (Medial Axis Extraction from outline)** に pivot 決定
  (v0.6.7 outline から skeleton を自動抽出、書体品質 + zero-asset 両立目標)

### Tests

- 333 pass (325 既存 + skeleton_shader 1 新規 stroke_width bit-exact test)
- clippy 0/0、fmt clean

## [0.7.1] - 2026-07-31 — Font DSL MVP (S6.4 Approach 4、**alpha / preview**)

### Added — Metafont-style Font DSL (`src/font_dsl/`)

Approach 4 の実装 MVP。Rust builder DSL で
永字八法 8 stroke primitives + IDS 12 operators を提供、手書き定義した
15 primitive radicals + 4 composed kanji が 100% real Bezier stroke で
描画される (v0.7.0-alpha の 73/1053 leaf placeholder 問題 を DSL 経由で回避)。

- `src/font_dsl/mod.rs` — GlyphBuilder + `glyph()` constructor + `dsl_lookup()`
- `src/font_dsl/stroke.rs` — 永字八法 8 primitives:
  - `horizontal`, `vertical` (straight)
  - `left_falling` (撇), `right_falling` (捺) (curved)
  - `dot` (点), `rising` (提)
  - `turn` (折), `hook` (鈎)
- `src/font_dsl/compose.rs` — IDS 4 core operators:
  - `left_right` (⿰), `top_bottom` (⿱)
  - `triple_horizontal` (⿲), `triple_vertical` (⿳)
  - Future: enclose_* (⿴⿵⿶⿷⿸⿹⿺) / overlap (⿻)
- `src/font_dsl/radicals.rs` — 20 char MVP library:
  - **数字**: 一 二 三 十
  - **自然**: 木 日 月 田 口 目 山 水 火 土
  - **人体**: 心 大 人 力 女 子
  - **合成**: 明 (⿰日月) / 林 (⿰木木) / 森 (⿱木⿰木木) / 相 (⿰木目)
- `examples/font_dsl_demo/` — 20 char showcase HTML (skeleton shader 経由)

### DSL vs kanji::build_skeleton

DSL 経由の `dsl_lookup(ch)` は 20 char 全部が real Bezier stroke を返す。
一方 `kanji::build_skeleton(ch)` は 1,053 char のうち 73 char のみ real
(残り 980 char は placeholder rectangle)。Consumer は:

```rust
dsl_lookup(ch).or_else(|| kanji::build_skeleton(ch))
```

の fallback pattern で DSL 優先 + 従来 skeleton path 経由の混合使用 可能。

### Metrics (20-char default demo)

- HTML size: 42 KB
- UBO total: 12,768 byte (6 chunks × 2,128 B)
- Runtime texture upload: **0 byte**
- Real render coverage: **20 / 20 (100%)**、placeholder rectangle 0 件

### 23 新規 tests

- font_dsl::mod (5 tests): GlyphBuilder / dsl_lookup / overflow truncation
- font_dsl::stroke (5 tests): horizontal / vertical / left_falling / dot / hook
- font_dsl::compose (6 tests): scale_point / left_right / top_bottom /
  triple_horizontal / MAX_STROKES truncation
- font_dsl::radicals (7 tests): ichi shape / san / ki structure / mei
  composition / hayashi / mori truncation / 20-radical smoke test

全 332 test pass (v0.7.0 の 309 + 23 新規)、clippy 0/0、fmt clean。

### Position

- **Content coverage**: 20 / 常用漢字 2,136 (~1%)、future v0.7.x-v0.8.x で
  DSL radical library を 214 まで拡張、IDS composition で 常用漢字 2,136
  カバー target
- **Consumer choice**: production 品質が必要 → v0.6.7 outline path (outline provider)、
  zero-asset ideal → v0.7.1-alpha DSL、両 path 併存

## [0.7.0] - 2026-07-31 — S6.4 zero-asset infrastructure (**alpha / preview**)

### Added — parametric skeleton shader path (S6.4 MVP Tier B)

- `src/skeleton_shader.rs` — `SkeletonGlyph` + `MultiSkeletonGlyphPayload` + `pack_skeleton_string()`
- `shaders/analytical_multi_skeleton.{vert,frag}.glsl` — Bezier stroke +
  variable-width pen SDF、aspect-fit + descender pad uniforms (v0.6.4 準拠)
- `examples/s64_zero_asset_demo/` — landing HTML + text input + parametric
  path 経由の render demo
- 9 skeleton_shader tests (UBO layout / chunk split / bit-exact serialization)

### Payload budget (structural)

- UBO per chunk: **2,128 byte** (4 glyph × 16 stroke × 2 vec4 = 2,048 + meta 80)
  = outline path 13,392B の **1/6.3**
- Sample demo (29 kanji, 8 chunks): 17 KB UBO + 5 KB shader + 54 KB HTML =
  **合計 ~76 KB total payload**
- Runtime texture upload: **0 byte** ✅
- WebGL2 16 KB UBO 使用率: 13.0% (余裕十分)

### ⚠ Known limitation (visual coverage) — alpha state

`kanji::build_skeleton()` は内部で `KANJI_DB` の IDS 定義を lookup し、
radicals + composition で Bezier stroke を出力する。ただし現状の KANJI_DB
1,053 entries のうち **~90% が Leaf placeholder** (単一 codepoint 登録、
本格 IDS decomposition 未実装、視覚的には矩形 placeholder が出力される)。

実 kanji shape がまともに描画されるのは:

- 一 / 二 / 三 / 十 (簡単 stroke)
- 田 / 川 / 山 等 (radical 直接展開の kanji)
- 一部 の grade 1-2 kanji (~50-100 char)

複雑 kanji (感 / 議 / 護 / 明 / 顔 / 憲 / 想 / 思 / 考 等) は matrix 座標に
矩形 placeholder が描画される (実描画品質は v0.6.x outline path 経由の
BIZ UDPGothic に遠く及ばない)。

### Path to S6.4 complete (future work)

真の S6.4 完成には以下が必要:

1. **radicals.rs の 214 radical Bezier stroke 完全定義** (現状の CjkRadical
   struct は id + character + name + stroke_count までで、Bezier data は
   partial 実装)
2. **ids_db.rs の Leaf entries の real IDS decomposition 追記** (seed
   subset 267 + missing 教育漢字 patch 対象)
3. **layout.rs の IDS operator (⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻) 12 種の bounding box
   分割 logic 完全化**

v0.7.0 は **infrastructure (shader path + UBO + demo shell)** を提供、
content (real radical / IDS) は future release で拡充。

### 位置付け

- **v0.7.0-alpha** — S6.4 の **structural proof of concept**、真の visual
  完成度は v0.7.x〜v0.8.x で実装する radical + IDS content 拡充を待つ
- **v0.6.7 (outline path)** は依然として **最も visual 品質の高い path**、
  L1 (outline provider) BIZ UDPGothic 経由の実 book glyph 描画に用いる
- 両 path は **併存**、outline path が production、skeleton path が
  research + zero-asset ideal 追求

## [0.6.7] - 2026-07-30

### 🔧 Fixed — MAX_CONTOURS 12 → 16 (護 の 13-contour truncation 解消)

L1 (outline provider) JIS X 0208 Level 1 全 2,965 kanji subset 展開後の
diagnostic 実測で判明: 護 = **13 contours** が `MAX_CONTOURS_PER_GLYPH = 12`
cap を超過し 13 個目の contour が silent skip されていた。JIS L1 3,344 char
全体で 12 超過は 護 が唯一。

**Fix**: MAX_CONTOURS 12 → 16 に拡張、UBO fit 維持:

- `OUTLINE_MAX_CONTOURS: 12 → 16` (single-glyph path)
- `MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH: 12 → 16`

Trade-off:
- Single UBO: 2,256B → **2,320B** (+64B、13.8% 使用率)
- Multi UBO: 13,136B → **13,392B** (+256B、min 16KB の 81.7% 使用)
- Per-fragment cost (single): 3,072 → 4,096 op 上限 (1.33x)
- Per-fragment cost (multi): 4,608 → 6,144 op 上限 (1.33x)

Text で 5 glyph 以上の chunk 分割は変わらず (MAX_GLYPHS = 4)。

### Changed — Rust constants (`src/outline.rs`)

- `OUTLINE_MAX_CONTOURS: 12 → 16`
- `OUTLINE_CONTOURS_AREA_SIZE: 192 → 256 byte`
- `OUTLINE_UBO_SIZE: 2,256 → 2,320 byte`
- `MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH: 12 → 16`
- `MULTI_OUTLINE_CONTOURS_AREA_SIZE: 768 → 1,024 byte`
- **`MULTI_OUTLINE_UBO_SIZE: 13,136 → 13,392 byte fixed**

### Changed — GLSL shaders

- `analytical_outline.frag.glsl`: `MAX_CONTOURS = 12 → 16`
- `analytical_multi_outline.{vert,frag}.glsl`: `MAX_CONTOURS_PER_GLYPH = 12 → 16`
- UBO struct layout bit-exact 一致継続。

### Tests updated

- `ubo_size_constant_is_2256_bytes` → `_2320_bytes`
- `std140_bytes_size_is_fixed_2256` → `_2320`
- `std140_meta_encodes_contour_count_total_points_advance`: offset 2240 → 2304
- `multi_outline_ubo_size_is_fixed_13136_bytes` → `_13392_bytes`
- meta_area offset 13,056 → 13,312
- utext_meta offset 13,120 → 13,376

### L1 (outline provider) 追従

- ALICE-Font v0.6.7 path dep 経由で自動反映、L1 側 code 変更なし。
- 護 の 13 contour が完全描画確認済 (gallery)。

## [0.6.4] - 2026-07-30

### 🔧 Fixed — canvas layout (aspect-preserve + descender padding)

user report (2026-07-30、v0.6.3 rendering): rendering quality は完全だが
canvas 内で glyph が overflow (愛感想思考 の 心 radical bottom cut、京都
大阪東京 の 都 阝 bottom cut、一二三十百千万 の 千/万 right cut、議護 の
両端 cut)。3 root cause:

1. canvas_h 固定 512 px、W:H aspect が glyph 数と em box に非追従
2. Descender (Y < 0 em、心 / 阝 / 月 tail) が UV [0, 1] 範囲外
3. Vertex shader が glyph を canvas 高 full stretch (aspect 無視)

**Fix**: vertex shader に `uniform vec2 uCanvasSize` + `uniform float
uYPadding` を追加、aspect-preserving fit + descender Y padding を実装。
Fragment shader は無変更 (vUV expand で descender 込みの em 位置を受け取る)。

Example canvas sizing:
- `PX_PER_EM = 200`
- `Y_PAD_EM = 0.15` (心/阝/月 TTF descender 0.12-0.20 em カバー)
- `canvas_w = clamp(total_advance × 200, 512, 2048)`
- `canvas_h = ⌈(1 + 2 × 0.15) × 200⌉ = 260 px`

### Changed — GLSL vertex shader

- Added `uniform vec2 uCanvasSize` + `uniform float uYPadding`
- Rewrote `gl_Position` transform for aspect-fit + letterbox
- Expanded `vUV.y` output to `[-uYPadding, 1 + uYPadding]` (descender coverage)
- Fragment shader 無変更 (bit-exact 継続)

### Breaking change

Consumer が `uCanvasSize` + `uYPadding` uniform を設定しないと glyph 位置
undefined。既存 L0 example は
本 release で追従。

### Fixed — examples

- `examples/analytical_outline_multi_poc/main.rs` — canvas sizing 更新
- `examples/analytical_outline_multi_poc/template.html` — uniform 設定追加

## [0.6.3] - 2026-07-30

### 🔧 Fixed — BIZ UDPGothic complex kanji truncation (384 pts × 4 glyphs)

L1 (outline provider) 経由の BIZ UDPGothic diagnostic 実測で判明: v0.6.2 の
`MAX_POINTS_PER_GLYPH = 256` cap を超える 300-400 point kanji が存在:

- 感: 371 pts (144% of 256 cap) ⚠ truncated
- 議: 366 pts (142%)
- 務: 355 pts (138%)
- 義: 320 pts (125%)
- 海: 299 pts (116%)
- 願: 253 pts (98% = near cap)

**Fix**: 256 → 384 point per glyph に拡張 (measured max 371 pt + safety
margin)、`MAX_GLYPHS: 6 → 4` に削減 (UBO fit 維持)。全 measured BIZ
UDPGothic 常用 kanji が truncate なしで完全描画可能。

Trade-off:
- UBO size: 13,552B → **13,136B** (-3%、min 16KB の 80.2% 使用、余裕 3,248B)
- Text > 4 glyph で chunk 分割: 6 glyph = 1 chunk (v0.6.2) → 5 glyph = 2 chunks (v0.6.3)
- Per-fragment cost: 3,072 → 4,608 op 上限 (1.5x、modern GPU で 15ms 級)

### Changed — Rust constants (`src/outline.rs`)

- `MULTI_OUTLINE_MAX_POINTS_PER_GLYPH: 256 → 384`
- `MULTI_OUTLINE_MAX_GLYPHS: 6 → 4`
- `MULTI_OUTLINE_POINTS_AREA_SIZE: 12,288 → 12,288 byte` (偶然一致)
- `MULTI_OUTLINE_CONTOURS_AREA_SIZE: 1,152 → 768 byte`
- `MULTI_OUTLINE_METAS_AREA_SIZE: 96 → 64 byte`
- **`MULTI_OUTLINE_UBO_SIZE: 13,552 → 13,136 byte fixed**

### Changed — GLSL shader

- `MAX_GLYPHS = 6 → 4`
- `MAX_POINTS_PER_GLYPH = 256 → 384`
- Same UBO struct layout, bit-exact 一致継続。

### Tests updated

- `multi_outline_ubo_size_is_fixed_13552_bytes` → `_13136_bytes`
- `multi_outline_7_glyphs_splits_to_2_chunks` → `_5_glyphs_splits_to_2_chunks`
- `multi_outline_6_glyphs_still_single_chunk` → `_4_glyphs_still_single_chunk`
- meta_area offset 13,440 → 13,056
- utext_meta offset 13,536 → 13,120

### Example template fix (cosmetic)

- `examples/analytical_outline_multi_poc/template.html` title:
  "v0.6.1" hardcoded → "v0.6.3" (v0.6.2 で見逃していた表示 bug)。

## [0.6.2] - 2026-07-30

### 🔧 Fixed — multi-outline truncation artifact (256 pts × 6 glyphs)

user report (2026-07-30、明月夜風鳥 rendering): v0.6.1 の `MAX_POINTS_PER_
GLYPH = 128` 制限で BIZ UDPGothic 複雑 kanji (200+ point) が末尾 point
切り捨て → 不完全 contour → winding number 誤判定 → 明 内側反転 / 月 余分
縦線 / 鳥 細部分離 の visible artifact。

**Fix**: 128 → 256 point per glyph に拡張 (single-outline path と一致)、
BIZ UDPGothic 常用 kanji の 90%+ を完全描画可能に。

Trade-off: `MULTI_OUTLINE_MAX_GLYPHS: 8 → 6` (UBO 16KB min 制限内 fit)。
- UBO size: 9,872B → **13,552B** (+37%、min の 82.7% 使用、余裕 2,832B)
- Text > 6 glyph で chunk 分割: 8 glyph = 2 chunks (v0.6.1) → 7 glyph = 2 chunks (v0.6.2)
- Per-fragment cost: 1,536 → 3,072 op 上限 (2x、modern GPU で 10ms 級)

### Changed — Rust constants (`src/outline.rs`)

- `MULTI_OUTLINE_MAX_POINTS_PER_GLYPH: 128 → 256`
- `MULTI_OUTLINE_MAX_GLYPHS: 8 → 6`
- `MULTI_OUTLINE_POINTS_AREA_SIZE: 8,192 → 12,288 byte`
- `MULTI_OUTLINE_CONTOURS_AREA_SIZE: 1,536 → 1,152 byte`
- `MULTI_OUTLINE_METAS_AREA_SIZE: 128 → 96 byte`
- **`MULTI_OUTLINE_UBO_SIZE: 9,872 → 13,552 byte fixed**

### Changed — GLSL shader (`analytical_multi_outline.{vert,frag}.glsl`)

- `const int MAX_POINTS_PER_GLYPH = 256`
- `const int MAX_GLYPHS = 6`
- Fragment shader `pointBase = glyphIdx * 256` (256 stride)、UBO layout 対応

### Changed — tests (byte offsets updated)

- `multi_outline_ubo_size_is_fixed_13552_bytes` (was 9872_bytes)
- `multi_outline_9_glyphs_splits_to_2_chunks` → `multi_outline_7_glyphs_splits_to_2_chunks`
- 新規: `multi_outline_6_glyphs_still_single_chunk` (境界値)
- byte offsets: meta_area 9,728 → 13,440、uTextMeta 9,856 → 13,536

### Test count

300 unit — all pass (no test count change、byte-level offsets のみ更新)。

### Version: 0.6.1 → 0.6.2 (patch bump = artifact fix、breaking なし)

## [0.6.1] - 2026-07-30

### 🎉 Added — S6.3.3 v0.6.1: instanced multi-outline text rendering (L0 part)

v0.6.0 single-outline path を multi-glyph text 対応に拡張。`drawArraysInstanced`
+ flat UBO (9,872B fixed / chunk) で N glyph = 1 draw call、任意長 text は
chunk 分割 (≤ 8 glyph/chunk)。runtime texture upload = 0 byte 継続。

### Added — Rust API (src/outline.rs 拡張、~250 LOC)

- **Const** (all in `alice_font::outline`):
  - `MULTI_OUTLINE_MAX_GLYPHS = 8`
  - `MULTI_OUTLINE_MAX_POINTS_PER_GLYPH = 128`
  - `MULTI_OUTLINE_MAX_CONTOURS_PER_GLYPH = 12`
  - `MULTI_OUTLINE_UBO_SIZE = 9_872` (fixed)
- **`OutlineGlyphLayout { x_offset, advance }`** — per-glyph text 配置
- **`MultiOutlineGlyphPayload { glyphs, layout, total_advance }`**:
  - `glyph_count()` / `chunk_count()` / `to_std140_ubo_chunks() -> Vec<Vec<u8>>`
- **`pack_outline_string(text, provider: impl Fn(char) -> Option<OutlineGlyph>)
  -> Option<MultiOutlineGlyphPayload>`** — 公開 API、provider callback で
  data injection (synthetic sample or L1 BIZ UDPGothic adapter どちらも)

### Added — GLSL shader pair (~180 LOC)

- **`shaders/analytical_multi_outline.vert.glsl`** (35 LOC): `gl_InstanceID`
  → per-glyph NDC via `uGlyphMetas[i]`
- **`shaders/analytical_multi_outline.frag.glsl`** (~130 LOC): flat
  `uPoints[glyphIdx * 128 + i]` 1 段 arithmetic indexing で winding number test

UBO Layout (Rust ↔ GLSL bit-exact 一致):
```
Offset 0..8,192       vec4 uPoints[512]         // 8 glyph × 128 point / 2 per vec4
Offset 8,192..9,728   vec4 uContourInfos[96]    // 8 × 12 (start_f, count_f, 0, 0)
Offset 9,728..9,856   vec4 uGlyphMetas[8]       // (advance, contour_count, x_offset, 0)
Offset 9,856..9,872   vec4 uTextMeta            // (total_advance, chunk_count, 0, 0)
Total = 9,872 byte per chunk (fixed)
```

### Added — `examples/analytical_outline_multi_poc/` (~250 LOC)

Synthetic multi-outline text demo:

```sh
cargo run --example analytical_outline_multi_poc --features serde -- "三十囗田" out.html
cargo run --example analytical_outline_multi_poc --features serde -- "三十三十三十三十" 8gl.html
```

**実行例** (実測):
- 三十囗田 (4 glyph): 1 chunk / 1 draw call (-75%)
- 三十三十三十三十 (8 glyph): 1 chunk / 1 draw call (-88%)
- 9+ glyph: 2+ chunks

### Added — tests (10 multi-outline)

- `multi_outline_ubo_size_is_fixed_9872_bytes` / `_fits_in_16kb_webgl2_min_spec`
- `pack_outline_string_empty_text_returns_none`
- `pack_outline_string_layout_is_cumulative` (三十三 → x_offset 0, 1, 2)
- `multi_outline_ubo_chunk_size_is_fixed`
- `multi_outline_9_glyphs_splits_to_2_chunks` (境界検証)
- `multi_outline_first_glyph_points_bit_exact`
- `multi_outline_per_glyph_meta_encodes_advance_contour_count_xoffset`
- `multi_outline_utext_meta_encodes_total_advance_and_glyph_count`

### Test count

300 unit — all pass on 2026-07-30 (+10 multi-outline)。`--features pyo3` 込みで 310 pass。

### Version: 0.6.0 → 0.6.1

### Next (L1 part、別 commit)

- **outline provider adapter**: `biz_udpgothic::regular_outline(ch)
  -> Option<alice_font::outline::OutlineGlyph>` glue module。
  1424 BIZ UDPGothic kanji を実描画可能に。

## [0.6.0] - 2026-07-30

### 🎉 Added — S6.3.3: outline (polygon contour) shader path (architecture pivot)

現行 `component_strokes` 191 手書き match arm による
kanji 描画は根本的に品質・網羅性で限界。**v0.6.0 で TTF-derived polygon
outline path を L0 に追加**、runtime は winding number test で per-pixel
inside/outside 判定、texture upload = 0 byte 継続。

「データを送るな、法則を送れ」原則遵守: polygon points (contour list) は
uniform buffer 経由の数値、bitmap / atlas / TTF outline binary 一切なし。

### L0/L1 分離 design (2026-07-30 制定)

- **L0 (ALICE-Font)** = `OutlineGlyph` API + `pack_outline_for_shader` +
  GLSL shader (`shaders/analytical_outline.{vert,frag}.glsl`)、data 非依存
- **L1 (outline provider)** = BIZ UDPGothic 等 TTF-derived outline data の
  provider (`biz_udpgothic::regular_outline(ch) -> alice_font::outline::OutlineGlyph`)
- **Consumer** = L0 + L1 両方 import、L1 で data 取得 → L0 で shader pack + render
- ALICE-Font 単独では synthetic sample (三/十/囗/田) のみ使用可

### Added — `src/outline.rs` (~250 LOC)

- **`OutlineGlyph`** — `{codepoint, contours: Vec<Vec<[f32; 2]>>, advance}`。
  各 contour = 閉じた polygon (最初と最後の点は implicit に接続)。
- **`OutlineGlyphPayload`** — bounding_box 含む shader-ready 表現
- **`pack_outline_for_shader(outline) -> Option<OutlineGlyphPayload>`** —
  公開 API、contour ≥ 1 かつ total points ≥ 3 で `Some`
- **`OutlineGlyphPayload::to_std140_bytes() -> Vec<u8>`** — 2,256 byte fixed
- **`OUTLINE_UBO_SIZE = 2_256`** / **`OUTLINE_MAX_POINTS = 256`** /
  **`OUTLINE_MAX_CONTOURS = 12`** const

### Added — GLSL shader pair (~150 LOC)

- **`shaders/analytical_outline.vert.glsl`** (25 LOC): fullscreen quad
- **`shaders/analytical_outline.frag.glsl`** (~120 LOC):
  - Packed vec4 point unpacking helper (`getPoint(i)`)
  - Winding number test (non-zero fill rule) + nearest-edge distance for AA
  - Per-fragment cost: `MAX_CONTOURS × MAX_POINTS = 12 × 256 = 3,072 op` 上限
  - 実 sample (三 12 point / 田 20 point) は数 × 10 op のみ

### UBO Layout (Rust ↔ GLSL bit-exact 一致、std140 spec 準拠)

```
Offset 0..2,048    vec4 uPoints[128]        // 256 point packed 2 per vec4
Offset 2,048..2,240  vec4 uContourInfo[12]   // (start_f, count_f, 0, 0) × 12
Offset 2,240..2,256  vec4 uOutlineMeta       // (contour_count_f, total_points_f, advance, 0)
Total = 2,256 byte fixed
```

### Added — `examples/analytical_outline_poc/` (~250 LOC)

Synthetic sample outlines (not from any copyrighted font):

- **three** (三): 3 horizontal rectangles (12 point, 3 contour)
- **ten** (十): horizontal + vertical bar (8 point, 2 contour)
- **box** (囗): outer boundary + inner hole (8 point, 2 contour)
- **rice** (田): outer boundary + 4 inner holes (20 point, 5 contour)

```sh
cargo run --example analytical_outline_poc --features serde -- three /tmp/three.html
open /tmp/three.html
```

**実行例** (実測):
- three: 3 contour / 12 point / UBO 2,256B / HTML 13.5 KB / texture 0B
- rice: 5 contour / 20 point / UBO 2,256B (fixed) / HTML 13.6 KB / texture 0B

実 book glyph rendering は outline provider (L1) 経由で consumer が data を
inject する design。

### Added — tests (11 outline)

- `ubo_size_constant_is_2256_bytes` / `ubo_fits_in_16kb_webgl2_min_spec`
- `outline_glyph_total_points_and_contour_count` / `bounding_box`
- `pack_outline_synthetic_three_succeeds`
- `pack_outline_empty_returns_none` / `too_few_points_returns_none`
- `std140_bytes_size_is_fixed_2256`
- `std140_first_point_bit_exact` — 三 point 0 が UBO offset 0..8 に一致
- `std140_contour_info_encodes_start_and_count` — 3 contour info byte-level
- `std140_meta_encodes_contour_count_total_points_advance`

### Test count

290 unit — all pass on 2026-07-30 (+11 outline tests)。`--features pyo3` 込みで 300 pass。

### Version: 0.5.4 → 0.6.0 (minor bump = architecture pivot、breaking なし additive)

### Deferred to v0.6.1+

- **Medial axis fitting**: BIZ UDPGothic outline → medial axis → ALICE-LOL
  DSL 経由 の parametric stroke definition (Path C.b、v0.7.x 予定)
- **ALICE-LOL crate integration**: typography branch 追加
- **Multi-glyph outline text rendering**: 現行 outline は 1 glyph/UBO、
  MultiGlyphOutlinePayload + instanced rendering は次段
- **outline provider adapter**: `biz_udpgothic → alice_font::outline::OutlineGlyph`
  glue module (L1 side、別 repo)

## [0.5.4] - 2026-07-30

### 🔧 Fixed — multi-kanji readability (flat UBO indexing)

user report (2026-07-30): v0.5.3 multi-glyph render で 3 本線 kanji (三 等) が
solid rectangle として表示、single-glyph は正常 → **nested UBO `uGlyphs[i].
strokes[j]` の 2 段 dynamic indexing が一部 GPU で壊れる** ことが原因と特定。

**Fix**: UBO 内部 layout を flat 化。`Stroke uStrokes[256]` + `GlyphMeta
uMetas[16]` の 1 段 arithmetic indexing (`uStrokes[vGlyphIdx * MAX_STROKES +
i]`) に置換。UBO 総 byte 数は 12,816B fixed 不変、GLSL ↔ Rust bit-exact 一致。

- `pub const MULTI_GLYPH_STROKE_AREA_SIZE: usize = 12288` (256 stroke × 48B)
- `pub const MULTI_GLYPH_METAS_AREA_SIZE: usize = 512` (16 GlyphMeta × 32B)
- `MULTI_GLYPH_UBO_SIZE` = 12,288 + 512 + 16 = **12,816B** (v0.5.3 と同 size)
- `MultiGlyphPayload::to_std140_ubo_chunks()` — flat 出力に refactor
- `shaders/analytical_multi_glyph.{vert,frag}.glsl` v3.2 — nested struct
  廃止、flat uStrokes / uMetas 参照に refactor

### Added — `sans_hairline` preset (shader path 用 thin pen)

`sans_regular` (weight=0.45、`stroke_half_width` ≈ 0.046 em) は CPU
rasterisation 64² tile 向け tune で、shader 高解像度描画では 3 本線が
merge しがち (三 spacing 0.3 em、pen 直径 0.092 em)。

**`sans_hairline` (weight=0.15、half_width ≈ 0.022 em、直径 0.044 em)** で
gap 保証、3 本線が読み分けられる。examples (single + multi + diagnostic)
全て hairline preset に切替。

### Added — `analytical_shader_diagnostic_poc` example

Multi-glyph 描画壊れの diagnostic + shader tuning 用 debug view:

- 大 canvas 512×512、grid overlay + center cross
- Side panel: stroke count / advance / pen params + payload JSON dump
- Shader compile log を UI 表示 (成功 green / エラー red 全文)
- 使い方: `cargo run --example analytical_shader_diagnostic_poc --features serde -- 三 /tmp/out.html`

### Test count

280 unit — all pass on 2026-07-30 (+4 flat UBO byte-level tests + 1 hairline
preset test)。`--features pyo3` 込みで 290 unit pass。

### Version: 0.5.3 → 0.5.4

### Related: architecture pivot (v0.6.0 pending)

現行 `component_strokes` 191 手書き match arm は根本的に品質・
網羅性で限界。**v0.6.0 で TTF (Noto Sans JP / BIZ UDPGothic 等 permissive
license) を compile-time 参照して medial axis fitting → ALICE-LOL DSL 経由
の parametric stroke definition** に架け替え予定 (Path C Hybrid、CHANGELOG
の別 milestone として実装)。runtime は法則配信 (Bezier + pen) のまま、
atlas / bitmap 一切なし。

## [0.5.3] - 2026-07-30

### 🎉 Added — S6.3.2b.3: instanced multi-kanji rendering

`drawArraysInstanced` + `MultiGlyphBlock` UBO で **N glyph = 1 draw call**
に統合。v0.5.2 の per-glyph draw call loop を廃止、chunk 分割で任意長 text
に対応。runtime texture upload = 0 byte 継続。

### Added — Rust API

- **`pub const MAX_GLYPHS_PER_DRAW: usize = 16`** — 1 chunk 上限
- **`pub const MULTI_GLYPH_UBO_SIZE: usize = 12_816`** — fixed UBO size (16
  glyph × 800 + 16 uTextMeta = 12,816 byte)
- **`MultiGlyphPayload::chunk_count(&self) -> usize`** — ceil(glyph_count / 16)
- **`MultiGlyphPayload::to_std140_ubo_chunks(&self) -> Vec<Vec<u8>>`** —
  各 chunk 12,816 byte fixed size、per-glyph `meta.z = x_offset` を埋め込み、
  末尾 `uTextMeta = (total_advance, chunk_glyph_count_f, 0, 0)`

### Added — GLSL shader pair

- **`shaders/analytical_multi_glyph.vert.glsl`** (~55 LOC): `gl_InstanceID`
  → per-glyph NDC quad 配置、`vGlyphIdx` flat varying を fragment に渡す
- **`shaders/analytical_multi_glyph.frag.glsl`** (~120 LOC): `vGlyphIdx` で
  UBO 内 GlyphData を dynamic index、既存 SDF logic (Bezier polyline + pen
  t-nearest) 再利用

UBO layout (Rust ↔ GLSL bit-exact 一致):
```glsl
struct GlyphData {
    Stroke strokes[16];  // 768B
    vec4 pen;             // 16B
    vec4 meta;            // 16B: (advance, stroke_count_f, x_offset, 0)
};  // 800B / glyph

layout(std140) uniform MultiGlyphBlock {
    GlyphData uGlyphs[16];  // 12,800B (未使用 slot は zero-fill)
    vec4 uTextMeta;          // 16B: (total_advance, chunk_glyph_count_f, 0, 0)
};  // = 12,816B fixed per chunk
```

### Changed — `examples/analytical_shader_multi_poc/`

- **Per-glyph draw call loop 廃止**、`drawArraysInstanced` + per-chunk UBO
  update に置換
- Rust: `to_std140_ubo_chunks()` + `instance_counts` を JSON embed
- JS: 1 chunk = 1 `bufferSubData` + 1 `drawArraysInstanced` call
- Shader ペア差替え (single `analytical_glyph.frag.glsl` → multi `.vert +
  .frag.glsl`)

### Draw call 削減効果 (v0.5.2 → v0.5.3)

| Text | glyph_count | draws v0.5.2 | draws v0.5.3 | 削減 |
|---|---|---|---|---|
| 森明林 | 3 | 3 | 1 | **-67%** |
| 日月火水木金土 | 7 | 7 | 1 | **-86%** |
| 16 glyph | 16 | 16 | 1 | **-94%** |
| 17 glyph | 17 | 17 | 2 (chunk 分割) | **-88%** |
| 32 glyph | 32 | 32 | 2 | **-94%** |

**実行例** (実測):
```
$ cargo run --example analytical_shader_multi_poc --features serde -- "森明林" text.html
  chunks: 1 (12,816B fixed UBO)
  draw calls: 3 → 1 (-67% via drawArraysInstanced)
  html size: 39,579 byte

$ cargo run --example analytical_shader_multi_poc --features serde -- "日月火水木金土" week.html
  chunks: 1 / draw calls: 7 → 1 (-86%)
  html size: 39,976 byte

$ cargo run --example analytical_shader_multi_poc --features serde -- \
    "一二三四五六七八九十百千万木火水土" split.html
  chunks: 2 / draw calls: 17 → 2 (-89%)
  html size: 67,484 byte
```

### Added — tests (8 new)

- `multi_glyph_ubo_size_is_fixed_12816_bytes` — 定数値検証
- `multi_glyph_ubo_chunks_fits_in_16kb_webgl2_min_spec` — WebGL2 min UBO
  size (16 KB) 内に fit 保証
- `multi_glyph_short_text_produces_single_chunk` (森明林 = 1 chunk)
- `multi_glyph_16_glyphs_still_single_chunk` (境界値、16 = 1 chunk)
- `multi_glyph_17_glyphs_splits_to_two_chunks` (境界値+1、17 = 2 chunks)
- `multi_glyph_per_slot_meta_encodes_advance_stroke_count_xoffset` — per-glyph
  meta.x/y/z の byte-level 検証
- `multi_glyph_utext_meta_encodes_total_advance_and_chunk_count` —
  uTextMeta byte-level 検証
- `multi_glyph_ubo_second_chunk_encodes_partial_glyph_count` — 17 chars で
  chunk 1 の instance count = 1 検証

### Test count

276 unit — all pass on 2026-07-30 (+8 shader tests)。
`--features pyo3` 込みで 286 unit pass。

### Version: 0.5.2 → 0.5.3

### Deferred to S6.3.2c

- **Radical library UBO packing**: fragment shader 内で IDS 解決 + shared
  radical UBO 参照。stroke 重複削減、大量 kanji rendering で payload 節約。

### Deferred to S6.3.3

- MetaFontParams runtime morph (push-constant / uniform で slider 反映)

## [0.5.2] - 2026-07-30

### 🎉 Added — S6.3.2b sub-goal 1: UBO GLSL 完全統合

v0.5.1 の Rust `to_std140_bytes` を GLSL `layout(std140) uniform GlyphBlock`
経由で **実際に消費する** path を実装。uniform arrays (`vec2 uP0[16]` 等)
を廃止し、単一 UBO の bit-exact upload に切替。

runtime texture upload = 0 byte 継続、shader / example / Rust 全て一貫。

### Fixed — std140 layout spec 違反 (v0.5.1 の内部 bug、外部影響なし)

**v0.5.1 の `AnalyticalStroke::write_std140_bytes` は spec 違反** で 56 byte
/ stroke を出力していた (float 間に不要 pad 挿入)。**v0.5.2 で正しい
vec4-grouped layout に修正、48 byte / stroke** に。

- `AnalyticalStroke::STD140_SIZE`: 56 → **48** (vec4 × 3 = p01 / p23 / widths)
- `write_std140_bytes`: rewrite to vec4-grouped output (no inter-float pad)
- Byte-level test 全 update (offset 32-36 = w_start, 36-40 = w_end, 40-48 = pad × 2 vs
  以前の 32-36 = w_start, 36-40 = pad, 40-44 = w_end, 44-56 = pad × 3)

v0.5.1 の `to_std140_bytes` は consumer なし (Rust prep 段階) だったため
downstream 影響ゼロ。v0.5.2 で GLSL UBO と bit-exact 一致達成。

### Added — `GLYPH_UBO_SIZE = 800 byte` fixed size constant

```rust
pub const GLYPH_UBO_SIZE: usize = MAX_STROKES * 48 + 16 + 16;  // = 800
pub const MAX_STROKES: usize = crate::glyph::MAX_GLYPH_STROKES;  // = 16
```

UBO layout (Rust ↔ GLSL bit-exact 一致):
- `Stroke uStrokes[16]` — 768 B (48 × 16、実 stroke 数以降は zero-fill)
- `vec4 uPen` — 16 B (base_width, contrast, pen_angle, roundness)
- `vec4 uMeta` — 16 B (advance, stroke_count_as_f32, 0, 0)

**stroke 数によらず fixed 800 byte** (GLSL 静的 uniform block layout)、
shader は `uMeta.y` で loop 上限を動的判定。

### Changed — `AnalyticalGlyphPayload::to_std140_bytes` semantics

- v0.5.1: 可変長 (`strokes.len() * 56 + 32` byte)
- v0.5.2: **fixed 800 byte** (未使用 slot は zero-fill、GLSL 静的 UBO と一致)

### Changed — GLSL shader v3 (`shaders/analytical_glyph.frag.glsl`)

- uniform arrays (`vec2 uP0[16]` 等) を廃止
- `struct Stroke { vec4 p01; vec4 p23; vec4 widths; }` + `layout(std140)
  uniform GlyphBlock { Stroke uStrokes[16]; vec4 uPen; vec4 uMeta; }`
- Fragment logic (Bezier polyline + pen t-nearest) は v0.5.1 と同一

### Changed — examples (both single + multi 対応)

- `examples/analytical_shader_poc/`: JSON payload embed → **std140 bytes
  JSON array embed** (Uint8Array 経由で `bufferData(UNIFORM_BUFFER, bytes)`)
- `examples/analytical_shader_multi_poc/`: 同上、per-glyph loop で
  `bufferSubData` 書き換え + `drawArrays`

**実行例** (実測):
```
$ cargo run --example analytical_shader_poc --features serde -- 森 mori.html
  char: 森 (U+68EE) / strokes: 12
  UBO bytes: 800 (fixed 800B, texture upload = 0)
  html size: 12773 byte  (v0.5.1 の 15807B から圧縮 -19%)

$ cargo run --example analytical_shader_multi_poc --features serde -- "森明林" text.html
  glyphs packed: 3 / UBO total: 2400 byte (800 × 3)
  html size: 18005 byte

$ cargo run --example analytical_shader_multi_poc --features serde -- "日月火水木金土" week.html
  glyphs packed: 7 / UBO total: 5600 byte (800 × 7)
  html size: 24921 byte
```

### Added — tests (2 new)

- `std140_glyph_ubo_uMeta_encodes_advance_and_stroke_count` — uMeta vec4
  末尾 offset 784..800 の byte 検証
- `std140_glyph_ubo_pads_unused_stroke_slots_to_zero` — 森 (12 stroke) で
  slot 12..16 が zero-fill 確認

**変更 tests** (byte layout 検証):
- `std140_stroke_size_is_56_bytes` → `std140_stroke_size_is_48_bytes`
- `std140_glyph_payload_size_matches_stroke_count_times_56` →
  `std140_glyph_payload_size_is_fixed_800_bytes`
- `std140_stroke_layout_byte_level_verification` — vec4-grouped layout に更新

### Test count

268 unit — all pass on 2026-07-30 (net +2 tests、既存 test の layout 期待値 update)。
`--features pyo3` 込みで 278 unit pass。

### Deferred to S6.3.2b.3

- **Instanced rendering**: 現行 per-glyph draw call を `gl_InstanceID` 経由
  1 call で N glyph render に統合。per-glyph UBO offset を uniform buffer
  array で管理。

### Deferred to S6.3.2c

- **Radical library UBO packing**: fragment shader 内で IDS 解決 + shared
  radical UBO 参照。stroke 重複削減、大量 kanji rendering で payload 節約。

### Deferred to S6.3.3

- MetaFontParams runtime morph (push-constant / uniform で slider 反映)

## [0.5.1] - 2026-07-30

### 🎉 Added — S6.3.2: multi-kanji + pen refinement + std140 UBO prep

S6.3.1 の single-kanji analytical shader path を **3 sub-goal で拡張**:

1. **Pen 半幅 t-nearest 精緻化** (sub-goal 3) — shader-only 変更
2. **Multi-kanji text rendering** (sub-goal 2) — Rust + 新 example
3. **std140 UBO 互換 serialization** (sub-goal 1、Rust prep、GLSL 統合は S6.3.2b)

runtime texture upload = 0 byte を継続、text rendering に拡張。

### Added — GLSL shader v2 (`shaders/analytical_glyph.frag.glsl`)

- `sdSegmentWithT(p, a, b) -> vec2` — segment 距離 + tLocal (0..1) を返す
- `sdBezierPolylineWithT(p, A, B, C, D) -> vec2` — polyline 近似 + nearest
  Bezier global t
- `sdStroke` — pen 半幅を `mix(w0, w1, t)` で t-nearest 線形補間
  (従来 `0.5 * (w0 + w1)` 平均近似 から精緻化)

**視覚品質改善**: 細り端 / 太り端の thickness が直感通り遷移、
Bezier 曲率 + variable pen envelope の consistency が向上。

### Added — `src/shader.rs` MultiGlyphPayload (~150 LOC)

- **`MultiGlyphPayload`** — text 全体の shader-ready payload
  - `glyphs: Vec<AnalyticalGlyphPayload>` (reading order、未登録 skip)
  - `layout: Vec<GlyphLayout>` (per-glyph x_offset + advance)
  - `total_advance: f32` (text 全体幅)
  - `payload_bytes()` / `glyph_count()`
- **`GlyphLayout { x_offset, advance }`** (`#[repr(C)]`、8 byte 固定)
- **`pack_string_for_shader(text, params) -> Option<Self>`** — 公開 API
  - reading order で `pack_glyph_for_shader` を loop
  - 未登録 char は skip、`x_offset` は累積で計算 (text は詰まった layout)
  - text 空 or 全 char skip 時は `None`

### Added — std140 UBO serialization (~80 LOC)

Rust 側で **GLSL std140 UBO layout** に準拠する byte-level serialization
を先行実装 (GLSL 統合は S6.3.2b で完全対応):

- `AnalyticalStroke::STD140_SIZE = 56` (native 40B + std140 align pad 16B)
- `AnalyticalPen::STD140_SIZE = 16`
- `AnalyticalStroke::write_std140_bytes(&self, out: &mut Vec<u8>)`
- `AnalyticalPen::write_std140_bytes(&self, out: &mut Vec<u8>)`
- `AnalyticalGlyphPayload::to_std140_bytes(&self) -> Vec<u8>`
- `AnalyticalGlyphPayload::std140_size(&self) -> usize`

**Byte-level verification test**: 4 vec2 (32B) + float+pad (8B) +
float+pad12 (16B) = 56B のレイアウト byte 単位で検証。

### Added — `examples/analytical_shader_multi_poc/` (~260 LOC)

Multi-kanji WebGL2 standalone HTML generator:

- `main.rs` — CLI: `cargo run --example analytical_shader_multi_poc
  --features serde -- <text> <output.html>`
- `template.html` — WebGL2 harness、per-glyph draw loop + transform
  uniforms + blending
- `README.md` — 使い方 + rendering flow + fidelity improvements

**実行例** (実測):
```
$ cargo run --example analytical_shader_multi_poc --features serde -- "森明林" text.html
  text: 森明林 (3 chars) / glyphs packed: 3
  total advance: 3.00 em / canvas: 512×512 px
  payload_bytes: 1288 / html: 25155 byte
  texture upload: 0 byte

$ cargo run --example analytical_shader_multi_poc --features serde -- "日月火水木金土" week.html
  glyphs packed: 7 / total advance: 7.00 em / canvas: 896×512 px
  payload_bytes: 1560 / html: 27551 byte / texture: 0 byte

$ cargo run --example analytical_shader_multi_poc --features serde -- "森龥明" mixed.html
  glyphs packed: 2 (skip 未登録 龥) / total advance: 2.00 em
  payload_bytes: 940 / html: 21937 byte / texture: 0 byte
```

### Added — tests (10 new)

- `pack_string_empty_returns_none`
- `pack_string_all_unregistered_returns_none`
- `pack_string_single_glyph_matches_single_pack` — API consistency
- `pack_string_multi_glyph_layout_is_cumulative` — x_offset / advance
  chain 検証 (森明林 で 0.0 → 1.0 → 2.0)
- `pack_string_skips_unregistered_glyphs` — "森龥明" → 2 glyph 検証
- `multi_glyph_payload_bytes_matches_sum` — payload_bytes 一貫性
- `std140_stroke_size_is_56_bytes`
- `std140_pen_size_is_16_bytes`
- `std140_glyph_payload_size_matches_stroke_count_times_56`
- `std140_stroke_layout_byte_level_verification` — 4 vec2 + 2 float + pad
  layout の byte 単位検証

### Changed

- `Cargo.toml` に 2 個目の `[[example]]` セクション追加
  (analytical_shader_multi_poc、`required-features = ["serde"]`)

### Test count

266 unit — all pass on 2026-07-30 (S6.3.2 milestone +10 tests)。
`--features pyo3` 込みで 276 unit pass。

### Deferred to S6.3.2b

- **UBO GLSL 完全統合**: 現行 shader path は uniform 配列 (`uniform vec2
  uP0[16]` 等)、UBO 完全統合は text-rendering demand 顕在化時 (5 glyphs
  上限の壁が視野に入った時) に着手。Rust 側 `to_std140_bytes` は既に
  準備済で、GLSL 側 `layout(std140) uniform GlyphBlock { ... }` に切替
  すれば移行完了。

### Deferred to S6.3.3

- MetaFontParams runtime morph (push-constant / uniform で slider 反映)

## [0.5.0] - 2026-07-30

### 🎉 Added — S6.3.1: fragment-shader analytical Bezier distance PoC

**ROADMAP core value「データを送るな、法則を送れ」の直接実証達成**。
CPU rasterisation を bypass し、cubic Bezier control points +
variable-width pen envelope を uniform として GPU に送り、fragment shader
が per-pixel に SDF を analytical 評価する path を実装。

**runtime texture upload = 0 byte を数値保証**
(`AnalyticalGlyphPayload::payload_bytes()` で実測可能)。

### Added — `src/shader.rs` new module (~230 LOC)

- **`AnalyticalGlyphPayload`** — glyph 全体の shader-ready 表現
  - `strokes: Vec<AnalyticalStroke>` (全 Bezier stroke)
  - `pen: AnalyticalPen` (base_width / contrast / pen_angle / roundness)
  - `advance: f32` (em width)
  - `payload_bytes()` — texture 抜き uniform 総 byte 数を返す
  - `stroke_count()` — GLSL uStrokeCount uniform 用
- **`AnalyticalStroke`** — 単一 Bezier stroke (`#[repr(C)]`, 40 byte 固定)
  - 4 control points (`[f32; 2]` × 4) + `half_width_start` + `half_width_end`
- **`AnalyticalPen`** — pen model (`#[repr(C)]`, 16 byte 固定)
- **`pack_glyph_for_shader(ch, params) -> Option<Self>`** — 公開 API
  - `crate::glyph::kanji::build_skeleton` 経由で IDS 展開 stroke 取得
  - per-stroke に endpoint tangent 経由で pen 半幅を計算 → uniform ready

### Added — `src/glyph/kanji.rs::build_skeleton`

`generate()` を 2 段階に refactor:
- `build_skeleton(ch)` — IDS 経由で `GlyphSkeleton` 構築 (CPU rasterisation 抜き)
- `generate(ch, params)` = `build_skeleton` + `GlyphGenerator::generate_from_skeleton`

これにより shader path (`pack_glyph_for_shader`) が CPU pixel 生成を経ずに
skeleton を再利用可能。既存 `generate()` 挙動は完全に維持。

### Added — GLSL shader (`shaders/analytical_glyph.frag.glsl`, ~130 lines)

- `sdSegment(p, a, b)` — point-to-segment distance
- `bezierPos(A, B, C, D, t)` — cubic Bezier position
- `sdBezierPolyline(p, A, B, C, D)` — 16-subdivision polyline SDF 近似
- `sdStroke(p, A, B, C, D, w0, w1)` — variable-width pen 統合 signed distance
- `main()` — 全 stroke min union → smoothstep AA → fragment 出力

**Uniforms**: `uStrokeCount / uP0[16] / uP1[16] / uP2[16] / uP3[16] /
uHalfWidthStart[16] / uHalfWidthEnd[16] / uPenBaseWidth / uPenContrast /
uPenAngle / uPenRoundness / uFgColor / uBgColor / uAntiAlias`

### Added — `examples/analytical_shader_poc/` standalone WebGL2 demo

- `main.rs` — CLI: `cargo run --example analytical_shader_poc --features serde -- 森 mori.html`
- `template.html` — WebGL2 harness (inline vertex/fragment shader + JSON payload embed)
- `README.md` — 使い方 + 実装フロー + 関連リンク

**実行例** (実測):
```
$ cargo run --example analytical_shader_poc --features serde -- 森 mori.html
  char: 森 (U+68EE)
  strokes: 12
  payload_bytes: 500 (bitmap/atlas/texture upload = 0)
  html size: 15149 byte

$ cargo run --example analytical_shader_poc --features serde -- 明 mei.html
  char: 明 (U+660E)
  strokes: 10
  payload_bytes: 420 (bitmap/atlas/texture upload = 0)
  html size: 14529 byte
```

### Added — `shaders/README.md`

Shader 仕様 + Rust 側 payload との対応表 + iq's Bezier distance article 参照。

### Added — tests (7 shader module)

- `pack_mori_returns_payload` / `pack_mei_returns_payload`
- `pack_unregistered_returns_none` (U+9FA5 龥)
- `payload_size_below_1kb_for_mori_and_mei` — 森 500 B / 明 420 B 保証
- `pack_grade1_all_80_chars_no_panic` — 全 grade-1 で pack 成功
- `stroke_layout_is_c_repr_40_bytes` — `#[repr(C)]` GLSL 互換保証
- `pack_preserves_stroke_control_points_from_skeleton` — CPU との一致

### Changed

- `AnalyticalGlyphPayload` / `AnalyticalStroke` / `AnalyticalPen` に
  `serde` feature gate 付き `#[derive(Serialize, Deserialize)]`
- `Cargo.toml` に `[[example]]` セクション追加 (analytical_shader_poc、
  `required-features = ["serde"]`)

### Test count

256 unit — all pass on 2026-07-30 (S6.3.1 milestone +7 tests)。
`--features pyo3` 込みで 266 unit pass。

### Runtime texture upload = 0 byte 数値実証

`pack_glyph_for_shader('森', &sans_regular).unwrap().payload_bytes() = 500`。
このうち **bitmap / atlas / texture / TTF outline data = 0 byte**。全て
cubic Bezier control points + pen 半幅 (数値のみ) で構成。fragment shader
は uniform を読んで per-pixel に analytical 評価。

「データを送るな、法則を送れ」= v0.5.0 で構造的に達成。

### Roadmap

- **S6.3.1** ✓ 完了 (本 release)
- **S6.3.2** pending — UBO packing、multi-kanji parallel rendering、pen 半幅 t-nearest 精緻化
- **S6.3.3** pending — MetaFontParams runtime morph (push-constant / uniform 経由)
- **S6.4** pending — zero-asset browser demo (payload < 500 KB 目標)

## [0.4.6] - 2026-07-30

### Added — S6.2.1 COMPLETE — 教育漢字 全 6 学年 carrier-level coverage 達成

**S6.2.1 milestone 完了**: 教育漢字 1026 に対して **1018 字** の carrier-level
IDS DB カバレッジ達成 (**99.2 %**、canonical patch pending 8 字)。
KANJI_DB 892 → 1052 字 (+160)、常用漢字 2,136 に対して **49 %** (前回 42 %
から +7 pt、あと少しで折り返し達成)。

### Added — grade-6 kanji coverage (Plan B subset 182/191)

小学校学習指導要領 別表「学年別漢字配当表」6 学年 の Plan B subset 182 字
カバレッジ達成 (canonical 191 との差 -9、後日 patch 予定)。

**IDS 分解済み**: 12 字
- `城 = ⿰土成` / `奮 = ⿱大⿱隹田` (nested ternary) / `宗 = ⿱宀示`
- `宙 = ⿱宀由` / `幕 = ⿱莫巾` / `忠 = ⿱中心` / `株 = ⿰木朱`
- `砂 = ⿰石少` / `紅 = ⿰糸工` / `討 = ⿰言寸` / `貴 = ⿱⿻中一貝` (overlay op)
- `針 = ⿰金十`

**Leaf 登録**: 148 字 (`component_strokes` 直接 render 9 字 + placeholder
rectangle 139 字)。S6.1 拡充後に real IDS 昇格予定。

### Added — tests (6 grade-6 + 1 milestone)

- `grade6_reference_list_has_182_chars` (MEXT "191" subset 注記)
- `grade6_all_registered_with_correct_joyo_grade`
- `grade6_all_ids_strings_parse_successfully`
- `grade1_through_grade6_are_mutually_disjoint` — 6-way loop-based
- `grade6_all_182_chars_render_non_empty` — panic-free + inside pixel 保証
- **`educational_kanji_grades_1_to_6_total_1018_chars`** — S6.2.1 milestone
  total 検証 (80+160+199+211+186+182=1018 compile-time guarantee)

### Changed

- `KANJI_DB` 総数 892 → 1052 (+160)
- `registered_count_matches_seed_extract` 閾値 `>= 892` → `>= 1052`
- `GRADE_6_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開
- Module doc-comment を "S6.2.1 (grade 1..=6) 教育漢字 全 6 学年" に更新

### Fixed

- `奮` の IDS を `⿱大隹田` (arity 不整合) → `⿱大⿱隹田` (nested、arity 2 準拠) に
  修正。IDS parser の `⿱` は arity 2 のため、3 要素 top-mid-bottom は `⿳` か
  nested `⿱X⿱YZ` で表現する必要がある

### Test count

249 unit — all pass on 2026-07-30 (grade-6 milestone +6 tests、うち
milestone total test 1 件)。

### 次段候補

- **canonical patch batch**: grade 3 (+1 identify)、grade 4 subset (-9)、
  grade 5 (+7)、grade 6 (+9) の char-level canonical リスト確定 → 教育漢字
  1026 字 の bit-exact 到達
- **S6.1 Bezier skeleton 拡充**: `component_strokes` に 40+ 部品追加 →
  grade 1-6 で Leaf 登録された ~700 字を real IDS 昇格
- **S6.3.1 fragment-shader analytical Bezier distance PoC** (ROADMAP core)

## [0.4.5] - 2026-07-30

### Added — S6.2.1 grade-5 kanji coverage (Plan B subset 186/193)

小学校学習指導要領 別表「学年別漢字配当表」5 学年 の Plan B subset 186 字
カバレッジ達成。KANJI_DB 755 → 892 字 (+137)。累計 grade 1..=5 = 836 字
カバー。常用漢字 2,136 に対して 42 % (前回 35 % から +7 pt)。教育漢字
1,026 に対して 81 % (前回 63 % から +18 pt)。

**注**: MEXT 総数は "193 字" と表記されるが、本 crate では確実な 186 字
subset を採用 (canonical 193 char-level リスト判明時に 7 字追加 patch 予定)。
grade 3 (200 vs 199) / grade 4 (202 vs 211 superset) と同型の慣習的差異。

**IDS 分解済み (parts 全て component_strokes 定義済み)**: 13 字
- `因 = ⿴囗大` / `団 = ⿴囗才` / `基 = ⿱其土` / `墓 = ⿱莫土`
- `富 = ⿱宀畐` / `政 = ⿰正攵` / `敵 = ⿰啇攵` / `枝 = ⿰木支`
- `格 = ⿰木各` / `武 = ⿹弋止` / `講 = ⿰言冓` / `迷 = ⿺辶米`
- `預 = ⿰予頁`

**Leaf 登録**: 124 字 (`component_strokes` 直接 render 15 字 + placeholder
rectangle 109 字)。S6.1 拡充後に real IDS 昇格予定。

### Added — tests (5 grade-5)

- `grade5_reference_list_has_186_chars` (MEXT "193" subset 注記)
- `grade5_all_registered_with_correct_joyo_grade`
- `grade5_all_ids_strings_parse_successfully`
- `grade1_through_grade5_are_mutually_disjoint` — 5-way 重複配当予防
- `grade5_all_186_chars_render_non_empty` — panic-free + inside pixel 保証

### Changed

- `KANJI_DB` 総数 755 → 892 (+137)
- `registered_count_matches_seed_extract` 閾値 `>= 755` → `>= 892`
- `GRADE_5_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開
- 5-way disjoint test refactor to loop pattern (grade 6 追加時に extension 容易)

### Test count

243 unit — all pass on 2026-07-30 (grade-5 milestone +5 tests)。

## [0.4.4] - 2026-07-30

### Added — S6.2.1 grade-4 kanji coverage (Plan A superset 211/202)

小学校学習指導要領 別表「学年別漢字配当表」4 学年 の Plan A superset
211 字カバレッジ達成。KANJI_DB 580 → 755 字 (+175)。累計 grade 1+2+3+4
= 650 字カバー。常用漢字 2,136 に対して 35 % (前回 27 % から +8 pt)。
教育漢字 1,026 に対して 63 % (前回 43 % から +20 pt)。

**注**: MEXT 総数は "202 字" と表記されるが、本 crate では 2017 revision
で追加された **20 都道府県漢字を全て包含する 211 字 superset** を採用
(茨/媛/岡/潟/岐/熊/香/佐/埼/崎/滋/鹿/縄/井/沖/栃/奈/梨/阪/阜)。
canonical 202 字 char-level リスト判明時に 9 字 subset patch 予定。

**IDS 分解済み (parts 全て component_strokes 定義済み)**: 13 字
- `位 = ⿰人立` / `奈 = ⿱大示` / `好 = ⿰女子` / `季 = ⿱禾子`
- `念 = ⿱今心` / `材 = ⿰木才` / `牧 = ⿰牛攵` / `特 = ⿰牛寺`
- `省 = ⿱少目` / `議 = ⿰言義` / `辺 = ⿺辶刀` / `関 = ⿵門关`
- `香 = ⿱禾日`

**Leaf 登録**: 162 字 (`component_strokes` 直接 render 16 字 + placeholder
rectangle ~146 字)。S6.1 で `component_strokes` に 40+ 部品追加後、これら
Leaf を real IDS へ順次昇格予定 (grade 1+2+3 と同 deferred 方針)。

### Added — tests (5 grade-4)

- `grade4_reference_list_has_211_chars` (MEXT "202" superset 注記込み)
- `grade4_all_registered_with_correct_joyo_grade`
- `grade4_all_ids_strings_parse_successfully`
- `grade1_through_grade4_are_mutually_disjoint` — 4-way 重複配当予防
- `grade4_all_211_chars_render_non_empty` — panic-free + inside pixel 保証

### Changed

- `KANJI_DB` 総数 580 → 755 (+175)
- `registered_count_matches_seed_extract` 閾値 `>= 580` → `>= 755`
- `GRADE_4_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開 (grade 1+2+3 と同 pattern)

### Test count

238 unit — all pass on 2026-07-30 (grade-4 milestone +5 tests)。

## [0.4.3] - 2026-07-30

### Added — S6.2.1 grade-3 kanji full coverage (199/199)

小学校学習指導要領 別表「学年別漢字配当表」3 学年 199 字の完全カバレッジ
達成。KANJI_DB 439 → 580 字 (+141)。累計 grade 1+2+3 = 439 字カバー。
常用漢字 2,136 に対して 27 % (前回 20 % から +7 pt)。

**注: grade 3 は MEXT 総数 "200 字" と表記されるが、標準列挙は 199 字**
(慣習的差異、canonical 200 字 char-level リスト判明時に patch 予定)。

**IDS 分解済み (parts 全て component_strokes 定義済み)**: 21 字
- `味 = ⿰口未` / `和 = ⿰禾口` / `坂 = ⿰土反` / `委 = ⿱禾女`
- `安 = ⿱宀女` / `対 = ⿰文寸` / `式 = ⿹弋工` / `息 = ⿱自心`
- `昭 = ⿰日召` / `暑 = ⿱日者` / `板 = ⿰木反` / `植 = ⿰木直`
- `死 = ⿰歹匕` / `物 = ⿰牛勿` / `県 = ⿱目糸` / `福 = ⿰示畐`
- `秒 = ⿰禾少` / `章 = ⿱立日` / `詩 = ⿰言寺` / `転 = ⿰車云`
- `集 = ⿱隹木`

**Upgraded existing Leaf → real IDS**: 1 字
- `対`: Leaf → `⿰文寸` (grade-3 addition と同時に既存 seed も real 化)

**Leaf 登録**: 120 字 (`component_strokes` 直接 render 21 字 + placeholder
rectangle 99 字)。S6.1 で `component_strokes` に 30+ 部品追加後、これら
Leaf を real IDS へ順次昇格予定 (grade 1+2 と同 deferred 方針)。

### Added — tests (5 grade-3)

- `grade3_reference_list_has_199_chars` (MEXT "200" との差異注記込み)
- `grade3_all_registered_with_correct_joyo_grade`
- `grade3_all_ids_strings_parse_successfully`
- `grade1_grade2_grade3_are_mutually_disjoint` — 3-way 重複配当予防
- `grade3_all_199_chars_render_non_empty` — panic-free + inside pixel 保証

### Changed

- `KANJI_DB` 総数 439 → 580 (+141)
- `registered_count_matches_seed_extract` 閾値 `>= 439` → `>= 580`
- `GRADE_3_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開 (grade 1+2 と同 pattern)

### Test count

233 unit — all pass on 2026-07-30 (grade-3 milestone +5 tests)。

## [0.4.2] - 2026-07-30

### Added — S6.2.1 grade-2 kanji full coverage (160/160)

小学校学習指導要領 別表「学年別漢字配当表」2 学年 160 字の完全カバレッジ
達成。KANJI_DB 326 → 439 字 (+113)。累計 grade 1+2 = 240 字カバー。

**IDS 分解済み (parts 全て component_strokes 定義済み)**: 16 字
- `回 = ⿴囗口` / `岩 = ⿱山石` / `計 = ⿰言十` / `古 = ⿱十口`
- `国 = ⿴囗玉` / `細 = ⿰糸田` / `思 = ⿱田心` / `時 = ⿰日寺`
- `星 = ⿱日生` / `晴 = ⿰日青` / `多 = ⿱夕夕` / `知 = ⿰矢口`
- `歩 = ⿱止少` / `記 = ⿰言己` / `妹 = ⿰女未` / `肉 = ⿵冂人`

**Leaf 登録**: 97 字 (残り: `component_strokes` 直接 render or placeholder
rectangle fallback)。分解済み既存エントリと重複しないよう、`合 / 内 / 分 /
明 / 同 / 会` 等の grade-2 レベル seed 27 字は grade-2 追加候補から
除外 (既に KANJI_DB に real IDS 登録済み) → `joyo_grade = Some(2)` の付与
のみ。

### Added — tests (5 grade-2)

- `grade2_reference_list_has_160_chars`
- `grade2_all_registered_with_correct_joyo_grade`
- `grade2_all_ids_strings_parse_successfully`
- `grade1_and_grade2_are_disjoint` — 重複配当ミス予防
- `grade2_all_160_chars_render_non_empty` — panic-free + inside pixel 保証

### Changed

- `KANJI_DB` 総数 326 → 439 (+113)
- `registered_count_matches_seed_extract` 閾値 `>= 326` → `>= 439`
- `GRADE_2_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開 (grade-1 と同 pattern)

### Fixed

- `src/python.rs` に `extern crate alloc; use alloc::{string::String,
  vec::Vec};` 追加 (`no_std` 環境 で `--features pyo3` build 時 7 件の
  scope error 修正)
- pyo3 `#[pymethods]` macro-generated code の clippy false-positive
  (`too_many_arguments` / `useless_conversion`) を module-level `#![allow]`
  で抑制 (詳細コメント付き)

### Test count

228 unit — all pass on 2026-07-30 (grade-2 milestone +5 tests)。
`--features pyo3` 込みで 238 unit pass。

## [0.4.1] - 2026-07-30

### Added — S6.2.1 grade-1 kanji full coverage (80/80)

小学校学習指導要領 別表「学年別漢字配当表」1 学年 80 字の完全カバレッジ
達成。既存 267 字 + grade-1 補完 59 字 = 計 326 字。

**IDS 分解済み (parts 全て component_strokes 定義済み)**: 11 字
- `音 = ⿱立日` / `校 = ⿰木交` / `森 = ⿱木⿰木木` (nested)
- `早 = ⿱日十` / `村 = ⿰木寸` / `男 = ⿱田力`
- `天 = ⿱一大` / `百 = ⿱一白` / `林 = ⿰木木`
- `字 = ⿱宀子` / `円 = ⿵冂士`

**Leaf 登録 (component_strokes 直接 render or 未定義部品による placeholder)**:
48 字。分解対象部品 (`儿 / 止 / 凵 / 屮 / 亻 / 艹 / 冖 / 穴 / 亠 / 丶 /
丁 / 化 / 𠂇 / 龰 / 龸 / 龶 / 厂`) が component_strokes に未登録のため、
以降 S6.1 Bezier skeleton work で component_strokes 側を拡充後、これら
Leaf を real IDS へ順次昇格予定。

### Added — tests (3 grade-1 coverage + 1 render smoke)

- `grade1_reference_list_has_80_chars` — canonical reference が 80 字である
  ことを compile-time 保証
- `grade1_all_registered_with_correct_joyo_grade` — 80 字全てが KANJI_DB
  に登録され `joyo_grade == Some(1)` かつ `stroke_count > 0` を検証
- `grade1_all_ids_strings_parse_successfully` — 80 字 IDS 全てが IDS parser
  で valid に parse されることを検証
- `grade1_all_80_chars_render_non_empty` — `glyph::kanji::generate()` が
  80 字全てで panic なく inside pixel + 有限値 SDF を返すことを検証

### Changed

- `KANJI_DB` 総数 267 → 326 (+59)
- `registered_count_matches_seed_extract` 閾値 `>= 267` → `>= 326`
- `GRADE_1_KANJI` を `#[cfg(test)] pub(crate) const` として `cjk::ids_db`
  に公開 (cross-module render test 用の canonical reference)

### Test count

223 unit — all pass on 2026-07-30 (grade-1 milestone +4 tests: coverage 3
+ render 1)。

## [0.3.4] - 2026-07-30

### Added — `TextEffect` 2 new variants (R.3-h composite SDF ops)

Two new variants. Both are GPU-only specs.

**2 variants** (32 → 34 total `TextEffect` kinds):

- `AxisSlice { axis, count, frequency, gap_width, gap_color }` —
  Runtime SDF domain slicing. Splits the glyph tile into N cells along
  vertical / horizontal / grid, displaces each cell to expose the SDF
  cross-section (highlight colour), then reconvenes. Time-driven
  open ↔ close animation via `frequency`. This is a *runtime CSG
  operation* on the glyph, impossible in TTF / OTF / Variable Fonts.
- `FluidForm { intensity, attractor, color }` — Character shape emerges
  from an external fluid density field (e.g. a Navier-Stokes solver)
  filtered by the glyph SDF acting as attractor mask. Empty until
  fluid flows in, character forms as fluid accumulates. Distinct from
  `Sumi` which decorates a static glyph — here the glyph *literally
  does not appear* until enough fluid has collected. No static font
  format can express "shape emerges from external dynamic state".

### Added — canonical GLSL helpers

the effect shader library's R.3-h section
extension:

- `aliceAxisSlice(uv, axis, count, freq, gapWidth, time, out crossSectionMask) -> vec2`
- `aliceFluidForm(sdfSample, fluidDensity, intensity, attractor) -> float`

### Test count

230 unit + 1 doctest — all pass on 2026-07-30 (includes AxisSlice +
FluidForm serde roundtrip).

## [0.3.3] - 2026-07-30

### Added — `TextEffect` 2 new variants (R.3-h Original ALICE-Font effects)

Two fragment-level effects written from scratch (not upstream ports) to
demonstrate what TTF / OTF / Variable Fonts cannot achieve at the
fragment shader level. Both are GPU-only specs — `style_glyph` returns
`Color4::TRANSPARENT` (same fall-through as R.3-c / R.3-d / R.3-g).

**2 variants** (30 → 32 total `TextEffect` kinds):

- `MeltDrip { progress, viscosity, color }` — gravity-integrated
  candlewax / molasses melt with per-column noise drip trails. Distinct
  from `MeltDilate` (which oscillates edges); this one actually drips
  downward over time.
- `WeightMorph { min_weight, max_weight, frequency }` — real-time
  parametric weight interpolation, applied at the fragment shader
  level. Variable Fonts interpolate at shape-glyph level; ALICE-Font
  interpolates per-pixel by remapping the SDF edge threshold.

### Added — canonical GLSL helpers

the effect shader library's R.3-h section:

- `aliceMeltDrip(uv, progress, viscosity, time, out trailMask) -> vec2`
- `aliceWeightMorph(sdfSample, minWeight, maxWeight, time, frequency) -> float`

### Test count

230 unit + 1 doctest — all pass on 2026-07-30.

## [0.3.2] - 2026-07-29

### Added — `TextEffect` 5 new variants (R.3-g Keijiro Kino effects)

Conceptual ports of five Unity image-effect packages from Keijiro
Takahashi's Kino* series (`github.com/keijiro/Kino*`), added as
GPU-only spec variants. `style_glyph` returns `Color4::TRANSPARENT`
for these (same fall-through as R.3-c / R.3-d) — consumers set shader
uniforms and animate per-frame.

**5 variants** (25 → 30 total `TextEffect` kinds):

| Variant | Source repo (github.com/keijiro/) | License |
|--------|-----------------------------------|---------|
| `Datamosh { intensity, block_size, color_steps, seed }` | KinoDatamosh | Unlicense |
| `Contour { color, thickness, threshold }` | KinoContour | MIT |
| `AnamorphicStreak { color, length, angle, threshold }` | KinoStreak | MIT |
| `VolumetricFog { color, near, far, density }` | KinoFog | MIT |
| `Isoline { color, spacing, thickness }` | KinoIsoline | MIT |

Serde JSON roundtrip test coverage for all five variants (5 new samples
in `r3c_r3d_variants_serde_roundtrip`, all pass).

### Added — attribution

- `LICENSE-KINO` — MIT / Unlicense dual attribution to Keijiro Takahashi
  (Copyright 2015-2019). Required by consumers that redistribute the
  crate.

### Consumers

A consumer site added
five `case` branches wiring the new kinds to shader uniforms;
`site_checks.check_text_effect_bridge` now enforces all 30 kinds are
handled.

### Test count

230 unit + 1 doctest — all pass on 2026-07-29.

## [0.3.1] - 2026-07-29

### Added — `TextEffect` 20 new variants (R.3-c / R.3-d)

All variants are **GPU-only specs** — the CPU `style_glyph`
rasteriser returns `Color4::TRANSPARENT` for them so pre-rendered atlas
consumers don't accidentally embed frame-0 samples of an animated effect.
GPU renderers (browser / UE5 / Bamboo) read the spec, set shader uniforms,
and animate per frame.

**R.3-c — 15 variants upstreamed from a consumer site's shader uniforms**:

- `Dissolve`, `DistortionBulge`, `HaloRings`, `EdgeRipple`, `MeltDilate`
- `WaterRipple` (7-parameter water ripple + droplet, consolidated from
  the consumer's `uWaterRipple*` uniform family)
- `MetaballCollision` (up to 3 balls, fixed-array to preserve `Copy`)
- `ChromaticAberration`, `Decompose`, `GravityLens`, `Tendrils`
- `Sumi` (墨 — bleed / kasure / splash / wash, 4-parameter fluid ink
  consolidated from the consumer's `uSumi*` uniform family)
- `Parallax`, `TextWarp`, `Liquid`

**R.3-d — 5 variants for the 9-technique root-quality checklist**:

- `Fbm2Bevel` (#2 fbm displacement)
- `GyroidMicrostructure` (#2 亜種, TPMS lattice)
- `VoronoiCrack` (#2 亜種, cracked edge)
- `BeerLambertAbsorption` (#9, per-channel absorption)
- `IqFlourish` (#5, iq analytic 2D primitives) + `IqPrimitive` enum
  (`Star5` / `Heart` / `Diamond` / `Cross`)

### Added — `serde` feature (optional)

New Cargo feature `serde` (default off) derives `Serialize` / `Deserialize`
on `Color4` and `TextEffect`. The `TextEffect` JSON schema uses
`{ "kind": "<snake_case_variant>", ... }` — matches the convention used by
`pattern-effekseer-showcase.json` and other block schemas.

Enables downstream site consumers to store text-effect specs as JSON and
deserialise into `TextEffect` for shader-uniform dispatch.

```toml
alice-font = { version = "0.3.1", features = ["serde"] }
```

### No breaking changes

- `TextEffect` still derives `Copy` (all new variants use fixed arrays,
  no `Vec<T>`)
- All 5 original variants (`Outline` / `Shadow` / `Glow` / `InnerShadow` /
  `Gradient`) are unchanged
- Default features (`serde` off) produce a bit-identical `alice-font` to
  0.3.0

### Tests

- Default features: **228 tests pass** (unchanged from v0.3.0)
- `--features serde`: **230 tests pass** (+2 JSON roundtrip + kind tag
  snake_case format guards)

## [0.3.0] - 2026-07-29

### Changed

- License: `MIT OR Apache-2.0` → `AGPL-3.0-only OR LicenseRef-Commercial` (2026-07-29)

### Added — alice-font-atlas subcrate (Multi-Font TTF → SDF Atlas CLI)

- **New subcrate** `tools/alice-font-atlas/` — bake arbitrary licence-
  compatible TTFs into ALICE-Font-compatible SDF atlas PNG + JSON at
  build time. Wraps `alice_font::glyph::font_render::rasterize_from_
  contours`, produces output binary-compatible with `SdfAtlasMulti`
  layout.
- **CLI**: `--config <fonts.toml> --ttf-dir <dir> --out-dir <dir>` +
  repeatable `--charset-dir <dir>` / `--charset-inline <str>` /
  `--num-pages N` / `--tile-dim N`.
- **fonts.toml schema** — declarative list of font families + weights +
  TTF filenames.

### Documentation

- README.md — new **Multi-Font Atlas Generation** section documenting
  `alice-font-atlas` CLI, fonts.toml schema, and output conventions.

### No breaking API changes

- All 0.2.x public APIs (`MetaFontParams`, `GlyphGenerator`, `SdfAtlas`,
  `SdfAtlasMulti`, `TextShaper`, `EffectStack`, `TextEffect`,
  `rasterize_from_contours`, glyph dispatcher, CJK strokes) are
  unchanged.

## [0.2.0] - 2026-05-18

### Added — CJK (日本語) サポート

- **Glyph dispatcher** (`glyph::dispatcher`) — Unicode コードポイント別ルーティング
  (ASCII / Hiragana / Katakana / CJK Unified Ideographs / Unsupported)
- **Hiragana** (`glyph::hiragana`) — 82 字 (清音 46 + 濁音 21 + 半濁音 5 + 小書き 10)
- **Katakana** (`glyph::katakana`) — 83 字 (清音 46 + 長音 1 + 濁音 21 + 半濁音 5 + 小書き 10)
- **Kanji composition engine** (`glyph::kanji`) — IDS ツリーから漢字を組立て
  - 13 SDF プリミティブ + 12 合成オペレータ (左右、上下、囲み、重ね等) 全対応
  - Bbox 分割による再帰合成
  - 160+ 種の部品スケルトンを内蔵 (一二三十木日月口田力 + 人偏 + 言偏 + 辶部等)
- **CJK module 再編** (`cjk/`)
  - `cjk::layout::CompositionLayout` — IDS 12 オペレータ (`⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻`)
  - `cjk::radicals::RADICALS` — 康熙部首 214 件完全テーブル
  - `cjk::ids::Ids` + `parse()` — IDS 文字列パーサ (recursive descent)
  - `cjk::ids_db::KANJI_DB` — 271 字の手書き IDS 定義 (seed)
- **CJK stroke primitives 拡張** (`glyph::cjk_strokes`)
  - 既存 8 種 (永字八法) に Curve / Loop / Hook 追加
  - Loop は (4/3)(√2-1) ≈ 0.5523 近似で 4 cubic Bezier 合成
- **SdfAtlasMulti** (`atlas::SdfAtlasMulti`) — マルチページアトラス対応
  - `MAX_ATLAS_DIM_PER_PAGE = 64`, `MAX_ATLAS_PAGES = 8` (32,768 タイル)
  - `AtlasEntryMulti` / `SdfAtlasPage` に分割管理
  - 既存 `SdfAtlas` は完全後方互換のため残置
- **GlyphGenerator::generate_from_skeleton** — 自前 GlyphSkeleton から SDF を生成する公開 API
- **計画文書** (`docs/`) — CJK_DESIGN / CJK_ATLAS_REDESIGN / CJK_KANA_SPEC / CJK_KANJI_SPEC

### Changed

- `SdfAtlas::get_or_insert` を `glyph::dispatcher::generate` 経由に変更
  (旧: ASCII 以外を `?` に置換 → 新: Unicode 対応 placeholder)
- `cjk.rs` を `cjk/` ディレクトリに分割 (mod.rs + layout/radicals/ids/ids_db)

### Quality

- **clippy pedantic + nursery: 0 warnings** (旧 v0.1.0: 内部のみ確認)
- **220 tests** (unit 219 + doc-test 1) 全パス
- **alice-audit Tier A (100/100)** 達成
- fmt clean / doc warnings 0

## [0.1.0] - 2026-02-23

### Added
- `MetaFontParams` — 40-byte parametric font descriptor with 6 presets
- `Stroke` / `PenModel` — variable-width pen along Bezier skeleton
- `GlyphGenerator` / `GlyphSdf` — procedural SDF glyph generation (A-Z, a-z, 0-9, punctuation)
- CJK stroke primitives (horizontal, vertical, dot, turning strokes)
- `SdfAtlas` — GPU-friendly texture atlas with LRU eviction
- `TextShaper` — kerning, horizontal advance, line layout
- `FontLicense` — 32-byte wire format for usage rights tracking
- `GameTextStyle` / `EffectStack` — SDF-based outline, shadow, glow, gradient effects
- `no_std` compatible (requires `alloc`)
- 98 unit tests covering all modules
