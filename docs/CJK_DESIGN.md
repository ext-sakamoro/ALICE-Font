# CJK_DESIGN — ALICE-Font CJK 拡張 設計書

ALICE-Font に日本語 (ひらがな・カタカナ・漢字) を含む CJK 文字描画機能を追加するための設計書。

---

## 1. 背景と動機

### 1.1 現状の制約

ALICE-Font v0.1.0 は以下の制約により、CJK 描画に未対応:

1. `SdfAtlas::get_or_insert` が ASCII (0-127) のみ許容、それ以外は `?` に置換
2. `MAX_ATLAS_DIM = 16` × `GLYPH_SDF_SIZE = 32` = **512×512 px / 256 タイル**
3. `cjk` モジュールは部首データ 40 件のみ (214 中)、`CompositionLayout` は 5 種のみ (IDS 12 種中)
4. `CharComposition` は構造体定義のみ、グリフ合成ロジック未実装

### 1.2 目標

- **ALICE-Font 単体で extoria.co.jp (日本語) の全テキストを描画可能にする**
- 「Extoria のフォント技術で Extoria のサイトを描画」というブランドメッセージを完成させる
- 副次効果: alicelaw.net, alice-metal-card, ALICE-SaaS 299 個など、全 ALICE プロジェクトで日本語描画が可能になる

### 1.3 非目標

- 商用フォントと完全に同等の品質を初版で達成すること (継続改善とする)
- 中国語 (簡体/繁体) 完全対応 (第一目標は日本語、漢字字形は新字体優先)
- 韓国語ハングル対応 (将来検討)
- 縦書き (横書きを先行)

---

## 2. 全体アーキテクチャ

### 2.1 描画パイプライン (拡張後)

```
char (Unicode codepoint)
    │
    ▼
[GlyphDispatcher]  ←── codepoint 範囲で振り分け
    │
    ├─ U+0020-U+007E   → ASCII path     → glyph::upper/lower/digits/punct
    ├─ U+3040-U+309F   → Hiragana path  → glyph::hiragana (新規)
    ├─ U+30A0-U+30FF   → Katakana path  → glyph::katakana (新規)
    ├─ U+4E00-U+9FFF   → CJK Kanji path → glyph::kanji (新規) → ids_composer
    └─ それ以外          → fallback (?)
    │
    ▼
GlyphSdf (32×32 f32 SDF)
    │
    ▼
SdfAtlas (Unicode 対応、可変サイズ)
```

### 2.2 主要モジュール (新規/拡張)

| モジュール | 状態 | 役割 |
|---|---|---|
| `glyph::dispatcher` | **新規** | codepoint → グリフ生成パスへの振り分け |
| `glyph::hiragana` | **新規** | ひらがな 85 字をストローク合成 |
| `glyph::katakana` | **新規** | カタカナ 85 字をストローク合成 |
| `glyph::kanji` | **新規** | 漢字描画エントリ、IDS 経由で部品合成 |
| `glyph::cjk_strokes` | **拡張** | 既存 9 ストロークに、CJK 用バリエーション追加 |
| `cjk` | **大幅拡張** | 部首 40 → 214、合成レイアウト 5 → 12 |
| `cjk::ids` | **新規** | IDS (Ideographic Description Sequence) パーサと合成エンジン |
| `cjk::ids_db` | **新規** | 常用漢字 2,136 字の IDS テーブル (compile-time embedded) |
| `atlas::SdfAtlas` | **拡張** | ASCII 制限撤廃、ディスパッチャ経由でグリフ生成 |

### 2.3 ファイル変更マップ

```
src/
├── lib.rs                         # モジュール公開追加
├── atlas.rs                       # SdfAtlas Unicode 対応
├── cjk.rs                         # 214 部首テーブル + 12 レイアウト
├── glyph/
│   ├── mod.rs                     # dispatcher 統合
│   ├── dispatcher.rs              # 新規
│   ├── hiragana.rs                # 新規
│   ├── katakana.rs                # 新規
│   ├── kanji.rs                   # 新規
│   └── cjk_strokes.rs             # 拡張
└── cjk/
    ├── ids.rs                     # 新規 (IDS パーサ)
    └── ids_db.rs                  # 新規 (常用漢字 IDS テーブル、生成スクリプト出力)
```

注: `cjk.rs` を `cjk/mod.rs` + 配下サブモジュール構成に再編する。

---

## 3. 核となる設計判断

### 3.1 グリフ表現形式: パラメトリック合成

漢字を **ビットマップ** や **ベクトルパス** で持つのではなく、**部首と合成オペレータの組合せ** として記述する。

```rust
// 例: '明' = ⿰日月 (左に日、右に月)
KanjiDef {
    codepoint: '明',
    composition: Ids::LeftRight(
        Ref::Component('日'),
        Ref::Component('月'),
    ),
    total_strokes: 8,
}
```

これにより:
- 各漢字定義が ~50-200 バイト程度
- 2,136 字 ≈ 200KB の compile-time テーブル
- パラメータ (太さ・コントラスト等) 変更時にリレンダリング可能

### 3.2 IDS (Ideographic Description Sequences) の採用

CJK 漢字の構造を記述する Unicode 標準。

- 12 種類の構造演算子: `⿰⿱⿲⿳⿴⿵⿶⿷⿸⿹⿺⿻`
- 例: 「鳥」= `⿱日匕` ではなく `⿱⿱白匕灬` のように再帰的に分解可能
- データソース: CHISE プロジェクト、Wikimedia Commons IDS、Unicode Unihan データベース

詳細は [IDS_DATA_SOURCES.md](./IDS_DATA_SOURCES.md) 参照。

### 3.3 SDF タイルサイズの維持 (32×32)

GLYPH_SDF_SIZE = 32 を維持する理由:
- 既存テスト、benchmark 互換性を保つ
- 16-byte alignment が崩れない
- 漢字でも 32×32 で読める品質を、SDF の解像度独立性で担保

ただし、漢字の細い線や込み入った字 (画) のために、内部生成解像度を 64×64 → 32×32 ダウンサンプル方式を採用する。

```rust
// 内部生成
let high_res: [f32; 64*64] = render_kanji_at_64();
// ダウンサンプル (SDF は線形合成可能)
let glyph_sdf: [f32; 32*32] = downsample_4x(&high_res);
```

### 3.4 SdfAtlas の再設計

#### 課題
- 現在: 16×16 = 256 タイル上限
- 必要: ASCII 95 + カナ 170 + 常用漢字 2,136 = **2,401 タイル**

#### 解決策: ページング + 階層化アトラス

```
SdfAtlasMulti {
    pages: Vec<SdfAtlasPage>,  // 各ページが 32×32 タイル
    lookup_table: HashMap<char, (page_id, slot)>,
}
```

- 1 ページ: 32×32 タイル × 32×32 px = 1024×1024 px
- 3 ページ: 3072 タイル収容 (常用漢字 + カナ + ASCII を全部入れて余裕)
- GPU 側は ARRAY_TEXTURE_2D としてバインド

[CJK_ATLAS_REDESIGN.md](./CJK_ATLAS_REDESIGN.md) で詳細仕様。

### 3.5 LRU eviction の維持 vs 全載せ

| 戦略 | メリット | デメリット |
|---|---|---|
| LRU (動的) | 任意の文字を扱える、メモリ効率 | ランタイムでグリフ生成、レイテンシあり |
| 全載せ (静的) | ゼロレイテンシ、再生成不要 | アトラスサイズ固定 |

→ **両方サポート**: API で切り替え可能にする。Web サイトでは全載せ (ビルド時生成)、ゲームでは LRU。

---

## 4. テスト戦略

### 4.1 ユニットテスト (既存スタイル踏襲)

- 各カナ字の SDF 生成テスト (内部一貫性)
- IDS パーサの構文テスト
- 合成オペレータ各々の挙動テスト

### 4.2 ビジュアル回帰テスト (新規)

各代表文字を PNG 出力し、Git で差分管理:

```
tests/golden/
├── ascii/
│   ├── A.png
│   └── ...
├── hiragana/
│   ├── あ.png
│   ├── い.png
│   └── ...
├── katakana/
│   └── ...
└── kanji/
    ├── 漢.png
    ├── 字.png
    └── ...
```

PR 時に視覚的回帰がないことを確認。

### 4.3 SDF 品質メトリクス

- エッジの距離値が `[-1, 1]` の範囲内
- グリフの bbox がタイルに収まる
- ストローク交点で SDF の最小値が連続的
- 合成漢字の見た目妥当性 (現状は手動チェック)

---

## 5. パフォーマンス目標

| 操作 | 目標 |
|---|---|
| ASCII グリフ生成 | < 100 µs/字 |
| カナグリフ生成 | < 200 µs/字 |
| 漢字グリフ生成 (簡単な字) | < 500 µs/字 |
| 漢字グリフ生成 (複雑な字、再帰3層以上) | < 2 ms/字 |
| 全常用漢字プリロード | < 30 秒 (シングルスレッド) |
| 全常用漢字プリロード (Rayon) | < 5 秒 |
| アトラスメモリ (常用漢字フル) | < 16 MB (f32) / < 4 MB (PNG R8) |

---

## 6. 段階的リリース戦略

### v0.2.0: インフラ整備
- SdfAtlas Unicode 対応
- ディスパッチ機構
- カナ実装

### v0.3.0: 漢字基盤
- 214 部首テーブル
- IDS パーサ
- 12 合成レイアウト

### v0.4.0: 常用漢字 (一部)
- 頻出 500 字程度

### v0.5.0: 常用漢字フル
- 2,136 字完備

### v1.0.0: 品質安定
- 視覚回帰テスト完備
- ベンチマーク達成
- ドキュメント完全

---

## 7. 関連ドキュメント

- [CJK_ROADMAP.md](./CJK_ROADMAP.md) — Step S1-S8 詳細スケジュール
- [CJK_ATLAS_REDESIGN.md](./CJK_ATLAS_REDESIGN.md) — SdfAtlas Unicode 対応の詳細仕様
- [CJK_KANA_SPEC.md](./CJK_KANA_SPEC.md) — カナ 170 字の合成定義
- [CJK_KANJI_SPEC.md](./CJK_KANJI_SPEC.md) — 漢字合成エンジン仕様
- [IDS_DATA_SOURCES.md](./IDS_DATA_SOURCES.md) — IDS データソース調査

---

## 8. 設計レビュー対象

このドキュメントに対する想定される懸念と回答:

| 懸念 | 回答 |
|---|---|
| 32×32 SDF で漢字が潰れないか | 内部 64×64 生成 + ダウンサンプル + SDF の解像度独立性で担保 |
| 2,136 字のパラメトリック定義は本当に書けるか | IDS データから自動生成 + 部首 214 種のみ手書き |
| 商用フォントとの品質ギャップ | 初版で完全一致は不可、段階的改善。ブランド優先 |
| ライセンス | IDS データは Public Domain / GPL 系混在、MIT 互換のみ採用 (IDS_DATA_SOURCES.md 参照) |
| `no_std` 互換性 | `cjk::ids_db` は const テーブルで `no_std` 維持、`alloc` のみ依存 |
