# CJK_KANJI_SPEC — 漢字合成エンジン仕様

常用漢字 2,136 字を IDS (Ideographic Description Sequence) 駆動で描画する仕様。

---

## 1. 基本原理

### 1.1 IDS による漢字記述

各漢字を「合成オペレータ + 構成要素」のツリーとして表現する。

```
明 = ⿰日月          (左右合成)
森 = ⿱⿰木木木      (上に木、下に左右木木)
照 = ⿱⿰日召灬       (上に日召、下に灬)
```

Unicode の IDS 演算子は 12 種:

| 演算子 | 名称 | 構造 | 例 |
|---|---|---|---|
| `⿰` | 左右 | 左 \| 右 | 明=⿰日月 |
| `⿱` | 上下 | 上 / 下 | 森=⿱木林 |
| `⿲` | 左中右 | 左 \| 中 \| 右 | 街=⿲彳圭亍 (の一部) |
| `⿳` | 上中下 | 上 / 中 / 下 | 章=⿳立日十 |
| `⿴` | 全囲み | 外囲 + 中 | 国=⿴囗玉 |
| `⿵` | 上囲み | ⊓ + 中 | 同=⿵冂一口 |
| `⿶` | 下囲み | ⊔ + 中 | 凶=⿶凵乂 |
| `⿷` | 左囲み | ⊏ + 中 | 区=⿷匚乂 |
| `⿸` | 左上囲み | Γ + 中 | 病=⿸疒丙 |
| `⿹` | 右上囲み | ⌐ + 中 | 句=⿹勹口 |
| `⿺` | 左下囲み | ⌊ + 中 | 道=⿺辶首 |
| `⿻` | 重ね | 上に重なる | 半は ⿻八 + 一+ 丨 (例による) |

### 1.2 再帰的な分解

要素自体が IDS で記述されることもある:

```
鳥 = ⿱⿱白匕灬   (3 階層)
聞 = ⿵門耳        (2 階層)
警 = ⿱敬言        (敬自体が ⿰苟攵)
```

最大ネストは実用上 5 階層程度。

---

## 2. データ構造

### 2.1 Rust 表現

```rust
// src/cjk/ids.rs
use alloc::boxed::Box;

#[derive(Debug, Clone)]
pub enum Ids {
    /// 終端: 単一の文字 (部首 or 既定義の漢字)
    Leaf(char),
    /// 左右合成 ⿰
    LeftRight(Box<Ids>, Box<Ids>),
    /// 上下合成 ⿱
    TopBottom(Box<Ids>, Box<Ids>),
    /// 左中右 ⿲
    LeftMidRight(Box<Ids>, Box<Ids>, Box<Ids>),
    /// 上中下 ⿳
    TopMidBottom(Box<Ids>, Box<Ids>, Box<Ids>),
    /// 全囲み ⿴
    Enclosure(Box<Ids>, Box<Ids>),
    /// 上囲み ⿵
    TopSurround(Box<Ids>, Box<Ids>),
    /// 下囲み ⿶
    BottomSurround(Box<Ids>, Box<Ids>),
    /// 左囲み ⿷
    LeftSurround(Box<Ids>, Box<Ids>),
    /// 左上囲み ⿸
    TopLeftSurround(Box<Ids>, Box<Ids>),
    /// 右上囲み ⿹
    TopRightSurround(Box<Ids>, Box<Ids>),
    /// 左下囲み ⿺
    BottomLeftSurround(Box<Ids>, Box<Ids>),
    /// 重ね ⿻
    Overlay(Box<Ids>, Box<Ids>),
}
```

### 2.2 IDS 文字列パース

```rust
// src/cjk/ids.rs
pub fn parse(input: &str) -> Result<Ids, IdsParseError> {
    let mut chars = input.chars().peekable();
    parse_recursive(&mut chars)
}

fn parse_recursive(chars: &mut Peekable<Chars>) -> Result<Ids, IdsParseError> {
    let c = chars.next().ok_or(IdsParseError::Empty)?;
    match c {
        '⿰' => Ok(Ids::LeftRight(
            Box::new(parse_recursive(chars)?),
            Box::new(parse_recursive(chars)?),
        )),
        '⿱' => Ok(Ids::TopBottom(
            Box::new(parse_recursive(chars)?),
            Box::new(parse_recursive(chars)?),
        )),
        // ... 他のオペレータ
        c if is_terminal(c) => Ok(Ids::Leaf(c)),
        _ => Err(IdsParseError::UnknownChar(c)),
    }
}
```

### 2.3 漢字テーブル定義

```rust
// src/cjk/ids_db.rs (生成スクリプトから出力)
pub struct KanjiDef {
    pub codepoint: char,
    pub ids: &'static str,   // 例: "⿰日月"
    pub stroke_count: u8,
    pub joyo_grade: Option<u8>,  // 1-6: 小学校学年、7: 中学、8: 高校
}

pub const JOYO_KANJI: &[KanjiDef] = &[
    KanjiDef { codepoint: '明', ids: "⿰日月", stroke_count: 8, joyo_grade: Some(2) },
    KanjiDef { codepoint: '森', ids: "⿱木⿰木木", stroke_count: 12, joyo_grade: Some(1) },
    // ... 2,136 件
];
```

このテーブルは IDS データソースから生成スクリプトで出力する (build.rs ではなく、事前生成して `git` 管理)。

---

## 3. 描画パイプライン

### 3.1 IDS → 32x32 SDF

```rust
// src/glyph/kanji.rs
use crate::cjk::ids::{Ids, parse};
use crate::cjk::ids_db::lookup;
use crate::glyph::GlyphSdf;
use crate::param::MetaFontParams;

pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    // 1. テーブルから IDS 取得
    let def = match lookup(ch) {
        Some(d) => d,
        None => return GlyphSdf::empty(),
    };

    // 2. IDS パース
    let tree = parse(def.ids).unwrap_or_else(|_| Ids::Leaf(ch));

    // 3. 再帰的にレンダリング (高解像度内部バッファ)
    let high_res = render_ids_tree(&tree, params, Bbox::unit(), 64);

    // 4. ダウンサンプル (64×64 → 32×32)
    downsample_4x_to_2x(&high_res)
}

fn render_ids_tree(
    ids: &Ids,
    params: &MetaFontParams,
    bbox: Bbox,
    resolution: usize,
) -> SdfBuffer {
    match ids {
        Ids::Leaf(ch) => {
            // 部首 or 既定義漢字ならテーブル参照 + bbox 内に描画
            render_component(*ch, params, bbox, resolution)
        }
        Ids::LeftRight(left, right) => {
            let (bbox_l, bbox_r) = bbox.split_horizontal(0.5);
            let l = render_ids_tree(left, params, bbox_l, resolution);
            let r = render_ids_tree(right, params, bbox_r, resolution);
            sdf_union(&l, &r)
        }
        Ids::TopBottom(top, bottom) => {
            let (bbox_t, bbox_b) = bbox.split_vertical(0.5);
            let t = render_ids_tree(top, params, bbox_t, resolution);
            let b = render_ids_tree(bottom, params, bbox_b, resolution);
            sdf_union(&t, &b)
        }
        // ... 他のオペレータ
        _ => SdfBuffer::empty(resolution),
    }
}
```

### 3.2 合成の比率調整

均等分割では見栄えが悪い字も多い (例: 「明」は日:月 = 0.45:0.55)。
レイアウト毎に推奨比率を持つテーブルを設ける:

```rust
// src/cjk/layout_ratios.rs
pub fn left_right_ratio(left: char, right: char) -> f32 {
    // 部首によって最適比率を返す
    match (left, right) {
        ('日', '月') => 0.45,
        ('木', '木') => 0.5,
        _ => 0.5,  // デフォルト
    }
}
```

このテーブルは段階的に拡充 (S5 で 50 ペア、S6 で 200 ペア程度)。

### 3.3 部首の bbox 微調整

部首は単独使用時と、合成内のサブ要素として使用時で形が変わる (へん・つくり化):

| 単独 | 合成内 |
|---|---|
| 人 | 亻 (にんべん) |
| 心 | 忄 (りっしんべん) |
| 水 | 氵 (さんずい) |
| 火 | 灬 (れっか) |

これらを別グリフ定義として持つ:

```rust
// src/cjk/radicals.rs
pub fn variant_for_layout(radical: char, layout: CompositionLayout, position: Position) -> char {
    match (radical, layout, position) {
        ('人', CompositionLayout::LeftRight, Position::Left) => '亻',
        ('心', CompositionLayout::LeftRight, Position::Left) => '忄',
        _ => radical,  // 変化なし
    }
}
```

---

## 4. 部首ライブラリ (S5 で完成)

### 4.1 214 部首の網羅

康熙部首番号順に全 214 部首を `cjk::RADICALS` に登録。
各部首について:
- `id`: 1-214
- `character`: 元の部首字
- `variants`: 偏旁化した形 (1-3 種)
- `stroke_count`: 画数
- `glyph_skeleton`: ストローク定義 (`cjk_strokes::CjkStrokeType` の組合せ)

```rust
pub struct CjkRadical {
    pub id: u16,
    pub character: char,
    pub variants: &'static [char],
    pub stroke_count: u8,
    pub name: &'static str,
    pub skeleton: RadicalSkeleton,
}
```

### 4.2 部首スケルトンの実装

```rust
pub enum RadicalSkeleton {
    /// パラメトリック定義 (推奨)
    Parametric(&'static [CjkStrokeDef]),
    /// 再帰的に他の漢字/部首から組立
    Recursive(&'static str),  // IDS 文字列
    /// 未実装 (placeholder)
    Empty,
}
```

S5 完了時点で全 214 部首が `Parametric` で実装されることを目標。

---

## 5. データソース戦略

### 5.1 IDS データの取込

詳細は [IDS_DATA_SOURCES.md](./IDS_DATA_SOURCES.md) 参照。

候補:
1. **CHISE IDS** — 京都大 CHISE プロジェクト、CC-BY-SA、最も網羅的
2. **GlyphWiki IDS** — Public Domain、日本字形特化
3. **Wikimedia IDS** — Public Domain、Unicode IRG 由来

ライセンス互換性 (MIT/Apache 系) を考慮し、**GlyphWiki + Wikimedia** をベースに、CHISE はリファレンスのみ参照。

### 5.2 取込スクリプト

```
tools/import_ids/
├── Cargo.toml
└── src/
    └── main.rs
```

実行:
```bash
cargo run --bin import_ids -- --joyo --output ../../src/cjk/ids_db_generated.rs
```

出力は `git` 管理し、再生成は人手判断で行う (build.rs では走らせない、ビルド時間と再現性のため)。

---

## 6. 品質基準

### 6.1 視覚回帰テスト

全 2,136 字を 64×64 PNG として `tests/golden/kanji/` に格納し、Git 管理。
変更時は人間レビュー + ゴールデン更新。

### 6.2 構造的妥当性

```rust
#[test]
fn all_joyo_kanji_have_valid_ids() {
    for def in JOYO_KANJI {
        let parsed = parse(def.ids);
        assert!(parsed.is_ok(), "invalid IDS for {}: {}", def.codepoint, def.ids);
    }
}

#[test]
fn all_joyo_kanji_render_without_panic() {
    let params = MetaFontParams::sans_regular();
    for def in JOYO_KANJI {
        let _ = generate(def.codepoint, &params);
    }
}

#[test]
fn all_joyo_kanji_sdf_in_range() {
    let params = MetaFontParams::sans_regular();
    for def in JOYO_KANJI {
        let sdf = generate(def.codepoint, &params);
        for px in sdf.data.iter() {
            assert!(px.is_finite() && px.abs() <= 1.1);
        }
    }
}
```

### 6.3 パフォーマンス

```rust
#[bench]
fn bench_kanji_generation_simple(b: &mut Bencher) {
    let params = MetaFontParams::sans_regular();
    b.iter(|| generate('明', &params));
}

#[bench]
fn bench_kanji_generation_complex(b: &mut Bencher) {
    let params = MetaFontParams::sans_regular();
    b.iter(|| generate('鬱', &params));  // 29 画
}
```

目標:
- 簡単な字 (~10 画): < 500 µs
- 複雑な字 (~20-30 画): < 2 ms

---

## 7. 段階的実装 (S5+S6)

### 7.1 S5: 基盤完成

- 12 オペレータ全実装
- 214 部首全実装
- IDS パーサ
- 簡単な漢字 (50 字程度) で動作確認

### 7.2 S6: 常用漢字フル

| サブフェーズ | 対象 | 字数 |
|---|---|---|
| S6.1 | 教育漢字 1 年 | 80 |
| S6.2 | 教育漢字 2 年 | 160 |
| S6.3 | 教育漢字 3 年 | 200 |
| S6.4 | 教育漢字 4 年 | 200 |
| S6.5 | 教育漢字 5 年 | 185 |
| S6.6 | 教育漢字 6 年 | 181 |
| S6.7 | 中学校漢字 | 1,110 |
| S6.8 | 残り | 20 |

各サブフェーズで品質確認 + 視覚回帰テスト → 次へ進む。
