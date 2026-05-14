# CJK_KANA_SPEC — カナ実装仕様

ひらがな・カタカナ合計 170 字を `cjk_strokes` の合成で実装する仕様。

---

## 1. 対象文字

### 1.1 ひらがな (U+3040-U+309F)

| カテゴリ | 字数 | 例 |
|---|---|---|
| 清音 | 46 | あいうえお かきくけこ さしすせそ たちつてと なにぬねの はひふへほ まみむめも やゆよ らりるれろ わをん |
| 濁音 | 25 | がぎぐげご ざじずぜぞ だぢづでど ばびぶべぼ ぱぴぷぺぽ |
| 半濁音 | 5 | ぱぴぷぺぽ |
| 小書き仮名 | 9 | ぁぃぅぇぉ ゃゅょっ |
| **合計** | **85** | |

注: 半濁音は濁音と重複カウント。実装は濁音と半濁音で別グリフ。

詳細リスト: `U+3041` (ぁ) - `U+3094` (ゔ) + `U+3095` (ゕ) `U+3096` (ゖ)。
ただし extoria.co.jp で必要な範囲のみ実装し、`ゕゖ` 等は除外可。

### 1.2 カタカナ (U+30A0-U+30FF)

| カテゴリ | 字数 | 例 |
|---|---|---|
| 清音 | 46 | アイウエオ カキクケコ サシスセソ タチツテト ナニヌネノ ハヒフヘホ マミムメモ ヤユヨ ラリルレロ ワヲン |
| 濁音 | 25 | ガギグゲゴ ザジズゼゾ ダヂヅデド バビブベボ パピプペポ |
| 半濁音 | 5 | パピプペポ |
| 小書き | 9 | ァィゥェォ ャュョッ |
| **合計** | **85** | |

長音記号 `ー` (U+30FC) は別途追加。

---

## 2. 設計原則

### 2.1 ストローク合成方式

各字を `cjk_strokes` の 9 種ストローク (Heng/Shu/Pie/Na/Dian/Zhe/Ti + α) の組合せとして定義。

```rust
// 例: ひらがな 'く'
KanaDef {
    codepoint: 'く',
    strokes: &[
        // 単一の屈曲線
        KanaStroke {
            stroke_type: CjkStrokeType::Zhe,
            start: Point2::new(0.6, 0.2),
            mid: Point2::new(0.3, 0.5),
            end: Point2::new(0.6, 0.8),
            width_start: 0.06,
            width_end: 0.06,
        },
    ],
    advance: 0.7,
}
```

### 2.2 カナ固有のストロークパターン

ひらがなは丸みのある形状、カタカナは直線的形状を持つ。`CjkStrokeType` を拡張:

```rust
// src/glyph/cjk_strokes.rs (S3 で拡張)
pub enum CjkStrokeType {
    Heng,    // 横
    Shu,     // 竪
    Pie,     // 撇
    Na,      // 捺
    Dian,    // 点
    Zhe,     // 折
    Ti,      // 提
    // 新規追加 (S3)
    Curve,   // 曲線 (ひらがな専用、Bezier)
    Loop,    // 閉曲線 ('め' 'ぬ' 等の渦)
    Hook,    // 鉤 (シ' 'ツ' 等の最後の跳ね)
}
```

### 2.3 字形パラメータ化

各字に対し、`MetaFontParams` の weight (太さ) と contrast (硬さ) に応答する設計:

```rust
pub fn render_hiragana(ch: char, params: &MetaFontParams) -> GlyphSdf {
    let def = lookup_hiragana_def(ch);
    let pen = PenModel::from_params(params);
    let skeleton = build_skeleton(&def, params);
    rasterize_sdf(&skeleton, &pen)
}
```

---

## 3. 字形定義の詳細

### 3.1 命名規則

`glyph::hiragana::a` = ひらがな「あ」
`glyph::katakana::a` = カタカナ「ア」

```
src/glyph/hiragana.rs
├── pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf
├── mod kana_a;      // あ
├── mod kana_i;      // い
├── ...
└── mod kana_n;      // ん
```

代替案: 1 ファイルに全 85 字定義をテーブル化 (推奨、コード量削減)。

### 3.2 1 字あたりの定義例

```rust
// ひらがな 'あ' (3 画)
pub fn glyph_a() -> KanaDef {
    KanaDef {
        codepoint: 'あ',
        strokes: &[
            // 第1画: 横線 (上半分)
            KanaStroke::heng(Point2::new(0.15, 0.3), Point2::new(0.75, 0.3), 0.06),
            // 第2画: 縦線 (中央、少し右へ傾斜)
            KanaStroke::shu_with_hook(
                Point2::new(0.5, 0.15), Point2::new(0.45, 0.95),
                0.06, hook_direction::Left,
            ),
            // 第3画: 曲線 (下半分、ループ含む)
            KanaStroke::curve_with_loop(
                Point2::new(0.7, 0.45),
                Point2::new(0.85, 0.7),
                Point2::new(0.5, 0.85),
                Point2::new(0.2, 0.7),
                Point2::new(0.4, 0.5),
                0.06,
            ),
        ],
        advance: 0.9,
        lsb: 0.05,
    }
}
```

### 3.3 ひらがな vs カタカナの差

カタカナは直線中心 → `KanaStroke::heng/shu/pie/na` で構成。
ひらがなは曲線中心 → `KanaStroke::curve/loop/hook` が頻出。

カタカナ「ア」(2 画):
```rust
pub fn glyph_a_kata() -> KanaDef {
    KanaDef {
        codepoint: 'ア',
        strokes: &[
            // 第1画: 横線 + 鉤
            KanaStroke::heng_with_hook(Point2::new(0.2, 0.25), Point2::new(0.8, 0.25), 0.06, 0.5, 0.4),
            // 第2画: 縦線 + 跳ね
            KanaStroke::pie_long(Point2::new(0.5, 0.25), Point2::new(0.15, 0.85), 0.06),
        ],
        advance: 0.85,
        lsb: 0.075,
    }
}
```

---

## 4. 濁点・半濁点の合成

### 4.1 戦略

濁点 `゛` (U+309B) と半濁点 `゜` (U+309C) を別グリフとして定義し、清音と合成:

```rust
pub fn render_dakuten(base: char, params: &MetaFontParams) -> GlyphSdf {
    let base_ch = remove_dakuten(base);  // 'が' -> 'か'
    let base_sdf = generate(base_ch, params);
    let dakuten_sdf = render_dakuten_mark(params);
    composite_sdf_min(&base_sdf, &dakuten_sdf, Point2::new(0.7, 0.15))
}
```

### 4.2 合成のメモリ効率

- 清音 + 濁点で 1 タイル = OK
- 専用タイルとして焼き込む (アトラスエントリは個別)
- 描画時は SDF レベルの min 合成

---

## 5. 小書き仮名

ぁぃぅぇぉ ゃゅょっ は通常字を **0.7 倍にスケール + 右下シフト** で生成:

```rust
pub fn render_small_kana(base: char, params: &MetaFontParams) -> GlyphSdf {
    let normal = generate(unsmall(base), params);  // 'ぁ' -> 'あ'
    transform_sdf(&normal, scale: 0.7, offset: Point2::new(0.15, 0.2))
}
```

---

## 6. 品質基準

### 6.1 視覚回帰テスト

`tests/golden/hiragana/` に各字 64×64 px の PNG を Git 管理。

```bash
cargo test --test visual_regression hiragana
```

差分が出たら手動レビュー → ゴールデン更新。

### 6.2 数値テスト

各字に対し:
- SDF 値域 `[-1, 1]`
- bbox `[0,1]^2`
- advance `0.5 < x < 1.2`
- ストローク数が画数と一致

### 6.3 全 170 字レンダリングテスト

```rust
#[test]
fn render_all_hiragana() {
    let params = MetaFontParams::sans_regular();
    for cp in 0x3041..=0x3094 {
        let ch = char::from_u32(cp).unwrap();
        let sdf = generate(ch, &params);
        assert!(sdf.advance > 0.0, "advance for {} is zero", ch);
        for px in sdf.data.iter() {
            assert!(px.is_finite(), "non-finite SDF for {}", ch);
            assert!(px.abs() <= 1.1, "out-of-range SDF for {}", ch);
        }
    }
}
```

---

## 7. 実装順序 (S3 内)

1. `cjk_strokes` の Curve/Loop/Hook 追加 (2 日)
2. 母音 5 字 (あいうえお) — 最も難しい曲線の典型例で品質確認 (2 日)
3. か行〜は行 (35 字、直線+曲線の標準) (4 日)
4. ま〜わ行 (清音残り) (2 日)
5. 濁音・半濁音合成 (1 日)
6. 小書き仮名 (1 日)
7. テスト + 視覚回帰 (2 日)

合計: 約 2 週間。

---

## 8. 既知の難点

| 字 | 難点 |
|---|---|
| あ | 第3画の渦巻きループ |
| ぬ | 末尾のループ |
| ね | 渦巻き構造 |
| る | 渦巻き構造 |
| を | カーブが多い |
| む | 鉤付きの複雑構造 |

これらは個別チューニングが必要。先に簡単な字 (い、く、つ、し、く、り、へ) で品質確認してから着手。

---

## 9. カタカナ実装の追加考慮

- カタカナは画数が少ない (多くて 3 画)、直線中心 → 実装容易
- ただし「ヌ」「ヲ」等の曲線含む字は注意
- 全 85 字 で 3-5 日想定

S4 はこの仕様で進行。
