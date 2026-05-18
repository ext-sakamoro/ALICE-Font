# Changelog

All notable changes to ALICE-Font will be documented in this file.

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
  - `cjk::ids_db::KANJI_DB` — 271 字の手書き IDS 定義 (extoria.co.jp seed)
- **CJK stroke primitives 拡張** (`glyph::cjk_strokes`)
  - 既存 8 種 (永字八法) に Curve / Loop / Hook 追加
  - Loop は (4/3)(√2-1) ≈ 0.5523 近似で 4 cubic Bezier 合成
- **SdfAtlasMulti** (`atlas::SdfAtlasMulti`) — マルチページアトラス対応
  - `MAX_ATLAS_DIM_PER_PAGE = 64`, `MAX_ATLAS_PAGES = 8` (32,768 タイル)
  - `AtlasEntryMulti` / `SdfAtlasPage` に分割管理
  - 既存 `SdfAtlas` は完全後方互換のため残置
- **GlyphGenerator::generate_from_skeleton** — 自前 GlyphSkeleton から SDF を生成する公開 API
- **計画文書** (`docs/`) — CJK_DESIGN / CJK_ROADMAP / CJK_ATLAS_REDESIGN /
  CJK_KANA_SPEC / CJK_KANJI_SPEC / IDS_DATA_SOURCES (合計 1,738 行)

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
