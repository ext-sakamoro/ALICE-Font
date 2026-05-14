# CJK_ATLAS_REDESIGN — SdfAtlas Unicode 対応 詳細仕様

[CJK_DESIGN.md](./CJK_DESIGN.md) §3.4 の SdfAtlas 再設計の詳細。

---

## 1. 既存の制約

### 1.1 ASCII 制限

`src/atlas.rs:154`:

```rust
pub fn get_or_insert(&mut self, ch: char) -> AtlasEntry {
    // ...
    let ascii = if ch.is_ascii() { ch as u8 } else { b'?' };
    let sdf = self.generator.generate(ascii);
    // ...
}
```

`GlyphGenerator::generate(u8)` は ASCII バイトを受け取る前提で、Unicode に拡張不可能。

### 1.2 サイズ上限

`src/atlas.rs:20`:

```rust
pub const MAX_ATLAS_DIM: usize = 16;
```

→ 16×16 = 256 タイル × 32×32 = 512×512 px。常用漢字 2,136 字には大幅に不足。

---

## 2. 再設計方針

### 2.1 後方互換性の維持

既存の `SdfAtlas` API は **そのまま残す** (deprecated 化なし)。
- ASCII 専用シングルページアトラス → 既存ユーザに影響なし
- 既存テスト 12 件は全パス

新規 API として `SdfAtlasMulti` (マルチページ対応) を追加。

### 2.2 グリフ生成の dispatcher 化

`glyph::dispatcher::generate(ch: char, params: &MetaFontParams) -> GlyphSdf`

```rust
// src/glyph/dispatcher.rs (新規)
use crate::glyph::{GlyphSdf, hiragana, katakana, kanji};
use crate::param::MetaFontParams;

pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    let cp = ch as u32;
    match cp {
        0x0020..=0x007E => {
            // ASCII (既存パス)
            let gen = crate::glyph::GlyphGenerator::new(params);
            gen.generate(ch as u8)
        }
        0x3040..=0x309F => hiragana::generate(ch, params),
        0x30A0..=0x30FF => katakana::generate(ch, params),
        0x4E00..=0x9FFF => kanji::generate(ch, params),
        _ => GlyphSdf::empty(),
    }
}
```

- S2 完了時点: ASCII 以外は全て `GlyphSdf::empty()` を返す (パス分岐は完成、実装は S3 以降)
- S3 完了で hiragana が、S4 で katakana が、S5+S6 で kanji が順次有効化

### 2.3 SdfAtlasMulti 構造

```rust
// src/atlas.rs (新規追加部分)
pub const MAX_ATLAS_PAGES: usize = 8;
pub const MAX_ATLAS_DIM_PER_PAGE: usize = 64;  // 16 から拡張

pub struct SdfAtlasPage {
    dim: usize,                              // タイル数 (1 辺)
    pixels: Vec<f32>,                        // (dim * GLYPH_SDF_SIZE)^2
    entries: Vec<Option<AtlasEntry>>,        // dim * dim
    occupied: usize,
}

pub struct SdfAtlasMulti {
    pages: Vec<SdfAtlasPage>,
    page_dim: usize,                         // 全ページ共通のタイル数
    params: MetaFontParams,
    lookup: alloc::collections::BTreeMap<char, (u16, u16)>,  // ch -> (page_id, slot)
    clock: u32,
}

impl SdfAtlasMulti {
    pub fn new(num_pages: usize, page_dim: usize, params: MetaFontParams) -> Self;
    pub fn capacity(&self) -> usize;          // num_pages * page_dim^2
    pub fn occupied(&self) -> usize;
    pub fn get_or_insert(&mut self, ch: char) -> AtlasEntryMulti;
    pub fn lookup(&mut self, ch: char) -> Option<&AtlasEntryMulti>;
    pub fn preload(&mut self, chars: &[char]);
    pub fn page_pixels(&self, page_id: usize) -> &[f32];  // GPU バインド用
    pub fn page_size(&self) -> usize;        // 1 ページのテクスチャ size (px)
    pub fn num_pages(&self) -> usize;
    pub fn contains(&self, ch: char) -> bool;
    pub fn clear(&mut self);
    pub fn set_params(&mut self, params: MetaFontParams);
}

#[derive(Debug, Clone, Copy)]
pub struct AtlasEntryMulti {
    pub codepoint: char,
    pub page_id: u16,
    pub tile_x: u16,
    pub tile_y: u16,
    pub uv_x: f32,
    pub uv_y: f32,
    pub uv_w: f32,
    pub uv_h: f32,
    pub advance: f32,
    pub lsb: f32,
    pub last_used: u32,
}
```

### 2.4 容量計算

| 設定 | ページ数 | ページタイル数 | 総タイル | 1 ページサイズ | 総メモリ (f32) |
|---|---|---|---|---|---|
| デフォルト (Web) | 3 | 32×32 | 3,072 | 1024×1024 px | 3 × 4 MB = 12 MB |
| 最小 (ゲーム LRU) | 1 | 16×16 | 256 | 512×512 px | 1 MB |
| 最大 | 8 | 64×64 | 32,768 | 2048×2048 px | 8 × 16 MB = 128 MB |

extoria-website-sdf-v2 で必要なのは: ASCII 95 + 日本語 ~800 = ~900 タイル → デフォルト (3 ページ) で余裕。

### 2.5 GPU アップロード戦略

#### 旧 (SdfAtlas)
- 1 枚の `R8` テクスチャ

#### 新 (SdfAtlasMulti)
- **オプション A**: ページ毎に別テクスチャ、シェーダで `page_id` を見て切替
- **オプション B**: `GL_TEXTURE_2D_ARRAY` でレイヤーアトラス、`vec3(uv, page_id)` でサンプル

→ **オプション B** を推奨 (シェーダがシンプル、ドローコール削減)。
WebGL2 は `TEXTURE_2D_ARRAY` をネイティブサポート。

シェーダ側:
```glsl
uniform sampler2DArray uAtlas;
in vec3 vAtlasUV;  // (u, v, page_id)
float dist = texture(uAtlas, vAtlasUV).r;
```

JSON メタデータには `page_id` を含める:
```json
{
  "glyphs": {
    "明": {
      "page": 1,
      "uv": [0.5, 0.25, 0.03125, 0.03125],
      "advance": 1.0,
      "lsb": 0.0
    }
  }
}
```

---

## 3. データ量見積

### 3.1 メモリフットプリント

| 構成 | f32 ストレージ | PNG 圧縮後 (R8) |
|---|---|---|
| ASCII 95 字 (1 ページ、10×10) | 320×320 px × 4B = 410 KB | ~30 KB |
| カナ + ASCII (1 ページ、16×16) | 512×512 px × 4B = 1 MB | ~150 KB |
| 教育漢字 + カナ + ASCII (2 ページ、32×32) | 1024×1024 × 2 × 4B = 8 MB | ~2 MB |
| 常用漢字フル (3 ページ、32×32) | 1024×1024 × 3 × 4B = 12 MB | ~3 MB |

### 3.2 ロード時間目標

| 対象 | ロード時間 |
|---|---|
| ASCII 95 字 (シングルスレッド) | < 10 ms |
| ASCII + カナ 170 字 | < 50 ms |
| 教育漢字 1,026 字 (Rayon) | < 3 秒 |
| 常用漢字 2,136 字 (Rayon) | < 5 秒 |

---

## 4. 互換性ポリシー

### 4.1 API レベル

| API | v0.1 | v0.2 (S2 完了後) | 備考 |
|---|---|---|---|
| `SdfAtlas::new` | ✓ | ✓ | 完全互換 |
| `SdfAtlas::get_or_insert` (ASCII) | ✓ | ✓ | 完全互換 |
| `SdfAtlas::get_or_insert` (CJK) | `?` 置換 | dispatcher 経由で空 SDF (S2) → 各カテゴリ実装 (S3+) | ✓ 旧テスト 'A','B' 等は同じ結果 |
| `MAX_ATLAS_DIM` | 16 | **16 維持** (旧 API) | 互換性のため |
| `MAX_ATLAS_DIM_PER_PAGE` | — | 64 (新 API) | 新規定数 |
| `SdfAtlasMulti::*` | — | ✓ 新規 | 全 API 追加 |

### 4.2 セマンティックバージョニング

S2 完了で v0.2.0 リリース。

- メジャー版変更なし (0.x のまま)
- マイナー版 +1 (新機能追加)
- 既存 API 削除なし

---

## 5. リスクと対策

| リスク | 対策 |
|---|---|
| `BTreeMap` の `alloc` 依存で `no_std` 互換性低下 | `feature = "std"` ガード、`no_std` では旧 `SdfAtlas` のみ提供 |
| ページ間のキャッシュミス頻発 | 同言語の文字を同ページにまとめる (preload 順序を制御) |
| LRU eviction のページ跨ぎ複雑化 | 初版は eviction なし (全載せ前提)、後で改良 |
| f32 ストレージのメモリ圧迫 | 設定可能な `u8` 量子化オプション (`feature = "compact"` 検討) |

---

## 6. テスト計画

### 6.1 既存テスト維持

```bash
cargo test --lib atlas::tests  # 12 件全パス
```

### 6.2 新規テスト

```rust
// src/atlas.rs (新規テスト)
#[test]
fn test_multi_atlas_creation() {
    let atlas = SdfAtlasMulti::new(3, 32, MetaFontParams::sans_regular());
    assert_eq!(atlas.num_pages(), 3);
    assert_eq!(atlas.capacity(), 3 * 32 * 32);
}

#[test]
fn test_multi_atlas_insert_cjk() {
    let mut atlas = SdfAtlasMulti::new(1, 16, MetaFontParams::sans_regular());
    let entry = atlas.get_or_insert('あ');
    assert_eq!(entry.codepoint, 'あ');
    // S2 完了時点ではまだ empty SDF だが、bbox は正しく返る
}

#[test]
fn test_multi_atlas_page_overflow() {
    let mut atlas = SdfAtlasMulti::new(1, 2, MetaFontParams::sans_regular()); // 4 slots
    for c in 'A'..='J' { atlas.get_or_insert(c); }  // 10 chars in 4 slots
    // LRU eviction or error policy: 初版は溢れたら最古を破棄
    assert!(atlas.occupied() <= 4);
}

#[test]
fn test_dispatcher_ascii() {
    let sdf = crate::glyph::dispatcher::generate('A', &MetaFontParams::sans_regular());
    assert!(sdf.advance > 0.0);
}

#[test]
fn test_dispatcher_hiragana_returns_empty_in_s2() {
    let sdf = crate::glyph::dispatcher::generate('あ', &MetaFontParams::sans_regular());
    // S2 時点: empty
    // S3 完了後: 実 SDF
    // → assertion はバージョンで分岐するか、初版は empty を確認
    assert!(sdf.advance > 0.0);
}
```
