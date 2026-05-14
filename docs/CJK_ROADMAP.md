# CJK_ROADMAP — ALICE-Font CJK 拡張ロードマップ

[CJK_DESIGN.md](./CJK_DESIGN.md) で定義した CJK 拡張を、S1-S8 のステップに分割。

---

## S1: 計画文書化 (本ドキュメント群)

### 成果物
- `docs/CJK_DESIGN.md`
- `docs/CJK_ROADMAP.md` (本ファイル)
- `docs/CJK_ATLAS_REDESIGN.md`
- `docs/CJK_KANA_SPEC.md`
- `docs/CJK_KANJI_SPEC.md`
- `docs/IDS_DATA_SOURCES.md`

### 完了基準
- 全 6 ファイルが作成されている
- 各文書間の相互参照リンクが有効

### 工数
半日〜1日

---

## S2: SdfAtlas Unicode 対応 + アトラス上限拡張

### 実装内容
- `glyph::dispatcher` モジュール新設、codepoint 別に振り分け
- `SdfAtlas::get_or_insert` の ASCII 制限撤廃
- ASCII 以外の文字は当面 `glyph::empty()` を返す (Phase 3 以降で各カテゴリ実装)
- `SdfAtlasMulti` (ページング対応) を新設 (旧 `SdfAtlas` も互換性のため残す)
- `MAX_ATLAS_DIM` を `MAX_ATLAS_DIM_PER_PAGE` にリネーム、ページ数上限を新設

### 影響範囲
- `src/atlas.rs` — 大幅改修 (互換性維持)
- `src/glyph/mod.rs` — `dispatcher` 公開
- `src/glyph/dispatcher.rs` — 新規
- `src/lib.rs` — モジュール公開
- 既存テスト: 全パス維持

### 完了基準
- [ ] `atlas.get_or_insert('あ')` がパニックせず空グリフを返す
- [ ] `SdfAtlasMulti::new(3, 32, params)` で 3 ページ × 32×32 タイルを保持
- [ ] 既存テスト (atlas.rs 12件) 全パス
- [ ] 新規テスト (dispatcher, multi atlas) 追加
- [ ] `cargo clippy --all-targets -- -D warnings` 通過
- [ ] `cargo fmt --check` 差分なし
- [ ] `cargo doc --no-deps` warnings ゼロ

### 工数
2-3日

---

## S3: ひらがな実装 (~85 字)

### 実装内容
- `glyph::hiragana` モジュール新設
- 85 字 (清音 46 + 濁音 25 + 半濁音 5 + 小書き 9) のストローク合成
- 各字の `GlyphSkeleton` 定義
- ディスパッチャから `U+3040-U+309F` を hiragana 経由に

### 完了基準
- [ ] `atlas.get_or_insert('あ')` が正しい SDF を生成
- [ ] 全 85 字の生成テスト
- [ ] ビジュアル回帰テスト用 PNG ゴールデン (`tests/golden/hiragana/`)
- [ ] 各字の SDF 値域が `[-1, 1]` 内
- [ ] 全字の bbox が `[0,1] × [0,1]` 内
- [ ] cargo test / clippy / fmt 通過

### 工数
1-2 週間

---

## S4: カタカナ実装 (~85 字)

### 実装内容
- `glyph::katakana` モジュール新設
- 85 字 (清音 46 + 濁音 25 + 半濁音 5 + 小書き 9) 実装
- ひらがなより直線的なため、Heng/Shu/Pie/Na ストロークの組合せで構築

### 完了基準
- S3 と同じ品質基準

### 工数
3-5 日

---

## S5: CJK 部首ライブラリ + IDS 合成エンジン

### 実装内容
1. `cjk` モジュールの構造再編 (`cjk.rs` → `cjk/mod.rs` + サブモジュール)
2. 部首テーブル 40 → 214 完全対応
3. `CompositionLayout` を 5 → 12 拡張 (IDS 全演算子対応)
4. `cjk::ids` モジュール — IDS 文字列パーサ
5. `cjk::ids_db` モジュール — 常用漢字 IDS テーブル (生成スクリプト出力 = const Rust データ)
6. `glyph::kanji` モジュール — IDS を実 SDF にレンダリング
7. 各部首の `GlyphSkeleton` 定義 (214 件)

### 完了基準
- [ ] 214 部首全てが `glyph::kanji::render_radical(id)` でレンダリング可能
- [ ] 12 合成レイアウト全て動作 (各レイアウトのテスト含む)
- [ ] `cjk::ids::parse("⿰日月")` が `Ids::LeftRight(Component('日'), Component('月'))` を返す
- [ ] 簡単な合成漢字 (明、林、好、品 等) が描画可能
- [ ] ビジュアル回帰テスト追加

### 工数
2-3 週間

---

## S6: 常用漢字 2,136 字対応

### 実装内容
- IDS データソースから 2,136 字の IDS を取込
- `cjk::ids_db::JOYO_KANJI` に const テーブルとして埋め込み
- 各字の生成テスト
- 視覚回帰テスト

### 完了基準
- [ ] 2,136 字全てが `atlas.get_or_insert('字')` で生成可能
- [ ] 全字のビジュアル回帰テスト
- [ ] ベンチマーク: 全字プリロード < 5 秒 (Rayon)
- [ ] 既存テスト全パス、新規 ~200 件追加

### 工数
3-8 週間 (頻度別フェーズ分けで段階リリース)

#### サブフェーズ
- S6.1: 教育漢字 1,026 字 (小学校で習う字、頻度高)
- S6.2: 中学・高校漢字 1,110 字
- S6.3: 表外漢字 (extoria.co.jp で使う頻度が高い字を優先追加)

---

## S7: 品質チューニング (ALICE 品質サイクル)

### 実装内容
- `cargo test --all-features` 全パス
- `cargo clippy --all-targets --all-features -- -D warnings` 通過
- `cargo fmt --all -- --check` 差分なし
- `cargo doc --no-deps --all-features` warnings ゼロ
- `cargo bench` でパフォーマンス計測
- `alice-audit` 品質サイクル実行 (~/CLAUDE.md 規定)
- CHANGELOG.md 更新 (v0.2.0 → v1.0.0 各リリースノート)
- README.md に CJK サポート明記

### 完了基準
- [ ] ALICE 100/100 品質基準達成
- [ ] パフォーマンス目標達成 (CJK_DESIGN.md §5)

### 工数
継続的

---

## S8: extoria-website-sdf-v2 Phase 1 着手

### 内容
- ALICE-Font CJK 拡張完了後、当初の `extoria-website-sdf-v2/ROADMAP.md` の Phase 1 に着手
- `tools/gen-atlas/` CLI で日本語含む全文字をアトラス化

### 完了基準
- `extoria-website-sdf-v2/assets/fonts/atlas-sans-regular.png` に日本語が含まれる
- `extoria-website-sdf-v2/ROADMAP.md` Phase 1 完了基準を全充足

### 工数
2-3 日 (ALICE-Font が完成していれば短期)

---

## 全体スケジュール (目安)

| Step | 工数 | 累積 |
|---|---|---|
| S1 | 0.5-1 日 | 1 日 |
| S2 | 2-3 日 | 4 日 |
| S3 | 1-2 週間 | 2 週間 |
| S4 | 3-5 日 | 3 週間 |
| S5 | 2-3 週間 | 6 週間 |
| S6 | 3-8 週間 | 10-14 週間 |
| S7 | 継続 | — |
| S8 | 2-3 日 | 10-14 週間 |

**総計**: 最短 **2.5 ヶ月**、品質追求で 3-6 ヶ月。

---

## リリースタイミング

| バージョン | 内容 | 推定 |
|---|---|---|
| v0.2.0 | S2 完了 (インフラ整備) | +1 週間 |
| v0.3.0 | S3+S4 完了 (カナ全対応) | +1 ヶ月 |
| v0.4.0 | S5 完了 (合成エンジン) | +2 ヶ月 |
| v0.5.0 | S6.1 完了 (教育漢字 1,026 字) | +3 ヶ月 |
| v0.6.0 | S6.2 完了 (中高漢字 1,110 字) | +4 ヶ月 |
| v1.0.0 | S7 完了 (品質安定 + 全常用漢字) | +5 ヶ月 |

---

## extoria-website-sdf-v2 への影響

| ALICE-Font バージョン | extoria-website-sdf-v2 で可能なこと |
|---|---|
| v0.2.0 | 英数記号のみアトラス化 (Phase 1 暫定版) |
| v0.3.0 | カナ含むアトラス化 (ひらがな主体のページデモ可能) |
| v0.5.0 | 教育漢字含めて Home / About 等の主要ページ全文 SDF 化可能 |
| v1.0.0 | 全 extoria.co.jp ページの SDF 化完了 |
