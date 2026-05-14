//! Kanji glyph rendering — U+4E00 to U+9FFF (CJK Unified Ideographs)
//!
//! S5 実装: IDS パーサと部首合成エンジンの基盤を整備。S6 で 214 部首
//! のスケルトン定義と常用漢字 2,136 字を順次追加していく。
//!
//! 現時点では、`cjk::ids_db::KANJI_DB` に登録された漢字を IDS 経由で
//! 合成可能。未登録の漢字は空 SDF を返す。
//!
//! 詳細は `docs/CJK_KANJI_SPEC.md` 参照。
//!
//! License: MIT
//! Author: Moroya Sakamoto
extern crate alloc;
use alloc::vec::Vec;

use crate::cjk::ids::{parse, Ids};
use crate::cjk::ids_db::lookup as kanji_lookup;
use crate::cjk::layout::CompositionLayout;
use crate::glyph::{GlyphGenerator, GlyphSdf, GlyphSkeleton, MAX_GLYPH_STROKES};
use crate::param::MetaFontParams;
use crate::stroke::{Point2, Stroke};

const KANJI_ADVANCE: f32 = 1.0;

/// 描画領域 — `[x, y, x+w, y+h]` の単位正方形内サブ領域。
#[derive(Debug, Clone, Copy)]
struct Bbox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Bbox {
    const fn unit() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        }
    }
}

/// Kanji の SDF を生成する。未登録の漢字は空 SDF を返す。
#[must_use]
pub fn generate(ch: char, params: &MetaFontParams) -> GlyphSdf {
    let Some(def) = kanji_lookup(ch) else {
        let mut sdf = GlyphSdf::empty();
        sdf.advance = KANJI_ADVANCE;
        return sdf;
    };
    let Ok(tree) = parse(def.ids) else {
        let mut sdf = GlyphSdf::empty();
        sdf.advance = KANJI_ADVANCE;
        return sdf;
    };

    let mut skel = GlyphSkeleton::empty();
    skel.advance = KANJI_ADVANCE;
    let bbox = Bbox::unit();
    let mut strokes_added = 0usize;
    add_ids_strokes(&tree, bbox, &mut skel, &mut strokes_added);

    if strokes_added == 0 {
        let mut sdf = GlyphSdf::empty();
        sdf.advance = KANJI_ADVANCE;
        return sdf;
    }

    let gen = GlyphGenerator::new(params);
    gen.generate_from_skeleton(&skel)
}

/// IDS ツリーを再帰的に展開して、各部品のストロークを `skel` に追加する。
fn add_ids_strokes(tree: &Ids, bbox: Bbox, skel: &mut GlyphSkeleton, count: &mut usize) {
    match tree {
        Ids::Leaf(ch) => {
            add_component_strokes(*ch, bbox, skel, count);
        }
        Ids::Binary {
            layout,
            first,
            second,
        } => {
            let (b1, b2) = split_binary(bbox, *layout);
            add_ids_strokes(first, b1, skel, count);
            add_ids_strokes(second, b2, skel, count);
        }
        Ids::Ternary {
            layout,
            first,
            second,
            third,
        } => {
            let (b1, b2, b3) = split_ternary(bbox, *layout);
            add_ids_strokes(first, b1, skel, count);
            add_ids_strokes(second, b2, skel, count);
            add_ids_strokes(third, b3, skel, count);
        }
    }
}

/// 二項合成の bbox 分割。要素の比率はレイアウト毎に決定。
fn split_binary(b: Bbox, layout: CompositionLayout) -> (Bbox, Bbox) {
    match layout {
        CompositionLayout::LeftRight => (
            Bbox {
                x: b.x,
                y: b.y,
                w: b.w * 0.5,
                h: b.h,
            },
            Bbox {
                x: b.x + b.w * 0.5,
                y: b.y,
                w: b.w * 0.5,
                h: b.h,
            },
        ),
        CompositionLayout::TopBottom => (
            // first = top
            Bbox {
                x: b.x,
                y: b.y + b.h * 0.5,
                w: b.w,
                h: b.h * 0.5,
            },
            // second = bottom
            Bbox {
                x: b.x,
                y: b.y,
                w: b.w,
                h: b.h * 0.5,
            },
        ),
        CompositionLayout::Enclosure
        | CompositionLayout::TopSurround
        | CompositionLayout::BottomSurround
        | CompositionLayout::LeftSurround
        | CompositionLayout::Overlay => (
            // first = outer shell (full bbox)
            b,
            // second = inner content (inset)
            inset(b, 0.18),
        ),
        CompositionLayout::TopLeftSurround => (
            // first = outer L-shape (uses top-left half)
            b,
            // second = inner content shifted lower-right
            Bbox {
                x: b.x + b.w * 0.3,
                y: b.y,
                w: b.w * 0.7,
                h: b.h * 0.75,
            },
        ),
        CompositionLayout::TopRightSurround => (
            b,
            Bbox {
                x: b.x,
                y: b.y,
                w: b.w * 0.7,
                h: b.h * 0.75,
            },
        ),
        CompositionLayout::BottomLeftSurround => (
            b,
            Bbox {
                x: b.x + b.w * 0.3,
                y: b.y + b.h * 0.2,
                w: b.w * 0.7,
                h: b.h * 0.8,
            },
        ),
        CompositionLayout::LeftMidRight | CompositionLayout::TopMidBottom => {
            // Binary 分割関数だが、三項レイアウトが渡された場合は半分割で fallback。
            let half = b.w * 0.5;
            (
                Bbox {
                    x: b.x,
                    y: b.y,
                    w: half,
                    h: b.h,
                },
                Bbox {
                    x: b.x + half,
                    y: b.y,
                    w: half,
                    h: b.h,
                },
            )
        }
    }
}

fn split_ternary(b: Bbox, layout: CompositionLayout) -> (Bbox, Bbox, Bbox) {
    match layout {
        CompositionLayout::LeftMidRight => {
            let w = b.w / 3.0;
            (
                Bbox {
                    x: b.x,
                    y: b.y,
                    w,
                    h: b.h,
                },
                Bbox {
                    x: b.x + w,
                    y: b.y,
                    w,
                    h: b.h,
                },
                Bbox {
                    x: b.x + 2.0 * w,
                    y: b.y,
                    w,
                    h: b.h,
                },
            )
        }
        CompositionLayout::TopMidBottom => {
            let h = b.h / 3.0;
            (
                Bbox {
                    x: b.x,
                    y: b.y + 2.0 * h,
                    w: b.w,
                    h,
                },
                Bbox {
                    x: b.x,
                    y: b.y + h,
                    w: b.w,
                    h,
                },
                Bbox {
                    x: b.x,
                    y: b.y,
                    w: b.w,
                    h,
                },
            )
        }
        _ => {
            // 二項レイアウトに三項分割が要求された場合の保守的フォールバック
            let w = b.w / 3.0;
            (
                Bbox {
                    x: b.x,
                    y: b.y,
                    w,
                    h: b.h,
                },
                Bbox {
                    x: b.x + w,
                    y: b.y,
                    w,
                    h: b.h,
                },
                Bbox {
                    x: b.x + 2.0 * w,
                    y: b.y,
                    w,
                    h: b.h,
                },
            )
        }
    }
}

fn inset(b: Bbox, margin: f32) -> Bbox {
    let m_x = b.w * margin;
    let m_y = b.h * margin;
    Bbox {
        x: b.x + m_x,
        y: b.y + m_y,
        w: b.w - 2.0 * m_x,
        h: b.h - 2.0 * m_y,
    }
}

/// 単一の部品文字を、指定 bbox に収まるようにストロークを `skel` に追加する。
/// S5 時点では、最頻出の部首 (一, 二, 木, 日, 月, 口, 田, 力, 女, 子, 玉, 囗, 冂) について
/// 簡易スケルトンを持つ。それ以外の部首は placeholder の正方形枠を描画する。
fn add_component_strokes(ch: char, b: Bbox, skel: &mut GlyphSkeleton, count: &mut usize) {
    let strokes = component_strokes(ch, b);
    for stroke in strokes {
        if *count >= MAX_GLYPH_STROKES {
            return;
        }
        skel.add_stroke(stroke);
        *count += 1;
    }
}

/// 部品 char の Bezier ストローク列を返す。
///
/// `m(u, v)` は正規化座標 `[0, 1]^2` を bbox 内ワールド座標に写像するヘルパ。
/// S6.6 までに 40+ の主要部品を定義し、未定義は矩形 placeholder で代替する。
fn component_strokes(ch: char, b: Bbox) -> Vec<Stroke> {
    let m = |u: f32, v: f32| Point2::new(b.x + u * b.w, b.y + v * b.h);
    match ch {
        // ---- Stroke-count 1-2 atomic shapes ---------------------------------
        '一' => alloc::vec![Stroke::line(m(0.1, 0.5), m(0.9, 0.5))],
        '二' => alloc::vec![
            Stroke::line(m(0.15, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.1, 0.3), m(0.9, 0.3)),
        ],
        '三' => alloc::vec![
            Stroke::line(m(0.15, 0.8), m(0.85, 0.8)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
            Stroke::line(m(0.1, 0.2), m(0.9, 0.2)),
        ],
        '十' => alloc::vec![
            Stroke::line(m(0.5, 0.9), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.5), m(0.9, 0.5)),
        ],
        '人' => alloc::vec![
            // Two diagonal strokes from top center.
            Stroke::new(m(0.5, 0.9), m(0.4, 0.65), m(0.25, 0.4), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.9), m(0.6, 0.65), m(0.75, 0.4), m(0.9, 0.1)),
        ],
        '八' => alloc::vec![
            Stroke::new(m(0.5, 0.85), m(0.4, 0.65), m(0.25, 0.4), m(0.1, 0.15)),
            Stroke::new(m(0.5, 0.85), m(0.6, 0.65), m(0.75, 0.4), m(0.9, 0.15)),
        ],
        '入' => alloc::vec![
            Stroke::new(m(0.5, 0.85), m(0.4, 0.6), m(0.3, 0.35), m(0.15, 0.15)),
            Stroke::new(m(0.35, 0.65), m(0.55, 0.5), m(0.75, 0.35), m(0.9, 0.15)),
        ],
        '刀' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.85, 0.85)),
            Stroke::new(m(0.85, 0.85), m(0.8, 0.55), m(0.7, 0.3), m(0.5, 0.1)),
            Stroke::new(m(0.4, 0.85), m(0.3, 0.55), m(0.2, 0.3), m(0.1, 0.1)),
        ],
        '力' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.85, 0.85), m(0.8, 0.4)),
            Stroke::new(m(0.8, 0.4), m(0.7, 0.2), m(0.5, 0.1), m(0.3, 0.1)),
            Stroke::new(m(0.45, 0.85), m(0.3, 0.55), m(0.2, 0.3), m(0.15, 0.1)),
        ],
        '又' => alloc::vec![
            Stroke::new(m(0.15, 0.85), m(0.35, 0.7), m(0.55, 0.55), m(0.8, 0.45)),
            Stroke::new(m(0.3, 0.6), m(0.45, 0.45), m(0.6, 0.3), m(0.85, 0.1)),
        ],
        '匕' => alloc::vec![
            Stroke::new(m(0.4, 0.85), m(0.3, 0.65), m(0.25, 0.45), m(0.2, 0.3)),
            Stroke::line(m(0.2, 0.3), m(0.85, 0.4)),
            Stroke::line(m(0.85, 0.4), m(0.8, 0.15)),
        ],
        '厶' => alloc::vec![
            Stroke::new(m(0.6, 0.85), m(0.4, 0.6), m(0.25, 0.4), m(0.15, 0.2)),
            Stroke::line(m(0.15, 0.2), m(0.85, 0.2)),
            Stroke::new(m(0.85, 0.2), m(0.75, 0.45), m(0.65, 0.6), m(0.55, 0.7)),
        ],
        '寸' => alloc::vec![
            Stroke::line(m(0.15, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.55, 0.85), m(0.5, 0.1)),
            Stroke::new(m(0.5, 0.3), m(0.4, 0.2), m(0.3, 0.15), m(0.2, 0.15)),
            Stroke::new(m(0.55, 0.55), m(0.65, 0.5), m(0.75, 0.45), m(0.85, 0.45)),
        ],
        '夕' => alloc::vec![
            Stroke::new(m(0.5, 0.9), m(0.35, 0.65), m(0.2, 0.4), m(0.1, 0.2)),
            Stroke::line(m(0.3, 0.6), m(0.85, 0.55)),
            Stroke::new(m(0.85, 0.55), m(0.7, 0.4), m(0.5, 0.25), m(0.3, 0.15)),
            Stroke::line(m(0.45, 0.3), m(0.6, 0.3)),
        ],
        '士' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::line(m(0.05, 0.15), m(0.95, 0.15)),
        ],
        '土' => alloc::vec![
            Stroke::line(m(0.2, 0.78), m(0.8, 0.78)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::line(m(0.05, 0.15), m(0.95, 0.15)),
        ],
        '工' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::line(m(0.1, 0.15), m(0.9, 0.15)),
        ],
        '己' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.85, 0.85), m(0.85, 0.55)),
            Stroke::line(m(0.2, 0.55), m(0.85, 0.55)),
            Stroke::new(m(0.2, 0.85), m(0.18, 0.5), m(0.2, 0.2), m(0.85, 0.15)),
        ],
        // ---- 心 (heart radical) ---------------------------------------------
        '心' => alloc::vec![
            // 3 dots + bottom curve.
            Stroke::new(m(0.35, 0.7), m(0.3, 0.55), m(0.3, 0.45), m(0.35, 0.4)),
            Stroke::new(m(0.55, 0.65), m(0.55, 0.5), m(0.55, 0.4), m(0.55, 0.35)),
            Stroke::new(m(0.75, 0.7), m(0.78, 0.55), m(0.78, 0.45), m(0.75, 0.4)),
            Stroke::new(m(0.15, 0.6), m(0.25, 0.3), m(0.5, 0.15), m(0.85, 0.25)),
        ],
        // ---- 口 (mouth) and box-like ----------------------------------------
        '口' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.2), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.2)),
        ],
        '囗' => alloc::vec![
            Stroke::line(m(0.1, 0.9), m(0.9, 0.9)),
            Stroke::line(m(0.9, 0.9), m(0.9, 0.1)),
            Stroke::line(m(0.1, 0.1), m(0.9, 0.1)),
            Stroke::line(m(0.1, 0.9), m(0.1, 0.1)),
        ],
        '冂' => alloc::vec![
            Stroke::line(m(0.15, 0.9), m(0.85, 0.9)),
            Stroke::line(m(0.85, 0.9), m(0.85, 0.15)),
            Stroke::line(m(0.15, 0.9), m(0.15, 0.15)),
        ],
        // ---- 大 / 子 / 女 / 山 / 中 ----------------------------------------
        '大' => alloc::vec![
            Stroke::line(m(0.1, 0.7), m(0.9, 0.7)),
            Stroke::new(m(0.5, 0.85), m(0.35, 0.55), m(0.2, 0.3), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.7), m(0.6, 0.45), m(0.75, 0.25), m(0.9, 0.1)),
        ],
        '女' => alloc::vec![
            Stroke::new(m(0.55, 0.9), m(0.4, 0.6), m(0.2, 0.35), m(0.1, 0.15)),
            Stroke::new(m(0.55, 0.9), m(0.7, 0.6), m(0.85, 0.35), m(0.9, 0.15)),
            Stroke::line(m(0.15, 0.5), m(0.85, 0.5)),
        ],
        '子' => alloc::vec![
            Stroke::new(m(0.1, 0.85), m(0.3, 0.85), m(0.7, 0.8), m(0.9, 0.7)),
            Stroke::new(m(0.55, 0.85), m(0.5, 0.6), m(0.45, 0.35), m(0.5, 0.1)),
            Stroke::line(m(0.15, 0.45), m(0.85, 0.45)),
        ],
        '玉' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.2, 0.55), m(0.8, 0.55)),
            Stroke::line(m(0.1, 0.2), m(0.9, 0.2)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.2)),
            Stroke::line(m(0.65, 0.35), m(0.72, 0.28)),
        ],
        '山' => alloc::vec![
            Stroke::line(m(0.1, 0.15), m(0.9, 0.15)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::line(m(0.2, 0.6), m(0.2, 0.15)),
            Stroke::line(m(0.8, 0.6), m(0.8, 0.15)),
        ],
        '中' => alloc::vec![
            Stroke::line(m(0.25, 0.85), m(0.75, 0.85)),
            Stroke::line(m(0.75, 0.85), m(0.75, 0.3)),
            Stroke::line(m(0.25, 0.85), m(0.25, 0.3)),
            Stroke::line(m(0.25, 0.3), m(0.75, 0.3)),
            Stroke::line(m(0.5, 0.95), m(0.5, 0.05)),
        ],
        // ---- 日 / 月 / 田 --------------------------------------------------
        '日' => alloc::vec![
            Stroke::line(m(0.2, 0.9), m(0.8, 0.9)),
            Stroke::line(m(0.8, 0.9), m(0.8, 0.1)),
            Stroke::line(m(0.2, 0.1), m(0.8, 0.1)),
            Stroke::line(m(0.2, 0.9), m(0.2, 0.1)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
        ],
        '月' => alloc::vec![
            Stroke::line(m(0.25, 0.9), m(0.75, 0.9)),
            Stroke::line(m(0.75, 0.9), m(0.75, 0.1)),
            Stroke::new(m(0.25, 0.9), m(0.2, 0.6), m(0.22, 0.3), m(0.18, 0.1)),
            Stroke::line(m(0.25, 0.65), m(0.75, 0.65)),
            Stroke::line(m(0.25, 0.4), m(0.75, 0.4)),
        ],
        '田' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.2), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.2)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.2)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
        ],
        // ---- 木 / 本 / 末 / 未 ---------------------------------------------
        '木' => alloc::vec![
            Stroke::line(m(0.5, 0.9), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.65), m(0.9, 0.65)),
            Stroke::new(m(0.5, 0.5), m(0.4, 0.4), m(0.3, 0.25), m(0.15, 0.15)),
            Stroke::new(m(0.5, 0.5), m(0.6, 0.4), m(0.7, 0.25), m(0.85, 0.15)),
        ],
        // ---- 水 / 火 / 氵 --------------------------------------------------
        '水' => alloc::vec![
            Stroke::line(m(0.5, 0.9), m(0.5, 0.1)),
            Stroke::new(m(0.5, 0.55), m(0.35, 0.4), m(0.25, 0.25), m(0.15, 0.1)),
            Stroke::new(m(0.5, 0.55), m(0.65, 0.4), m(0.75, 0.25), m(0.85, 0.1)),
            Stroke::new(m(0.35, 0.75), m(0.25, 0.6), m(0.15, 0.4), m(0.1, 0.3)),
            Stroke::new(m(0.65, 0.75), m(0.75, 0.6), m(0.85, 0.4), m(0.9, 0.3)),
        ],
        // ---- 言 (speech radical) -------------------------------------------
        '言' => alloc::vec![
            // Top dot + 3 horizontals + bottom box (口).
            Stroke::new(m(0.5, 0.95), m(0.5, 0.9), m(0.5, 0.88), m(0.5, 0.85)),
            Stroke::line(m(0.2, 0.78), m(0.8, 0.78)),
            Stroke::line(m(0.2, 0.65), m(0.8, 0.65)),
            Stroke::line(m(0.2, 0.52), m(0.8, 0.52)),
            // 口 below
            Stroke::line(m(0.25, 0.4), m(0.75, 0.4)),
            Stroke::line(m(0.75, 0.4), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.1), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.4), m(0.25, 0.1)),
        ],
        // ---- 手 (hand radical) ---------------------------------------------
        '手' => alloc::vec![
            Stroke::new(m(0.2, 0.85), m(0.4, 0.92), m(0.6, 0.92), m(0.85, 0.85)),
            Stroke::line(m(0.15, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.15, 0.5), m(0.85, 0.5)),
            Stroke::line(m(0.55, 0.85), m(0.5, 0.15)),
            Stroke::new(m(0.5, 0.15), m(0.4, 0.05), m(0.3, 0.08), m(0.25, 0.18)),
        ],
        // ---- 糸 (silk / thread) --------------------------------------------
        '糸' => alloc::vec![
            // Top: 幺 shape — two small loops + tail dots.
            Stroke::new(m(0.5, 0.95), m(0.4, 0.85), m(0.3, 0.75), m(0.25, 0.7)),
            Stroke::new(m(0.25, 0.7), m(0.5, 0.65), m(0.75, 0.65), m(0.85, 0.6)),
            Stroke::new(m(0.5, 0.7), m(0.4, 0.6), m(0.3, 0.55), m(0.25, 0.5)),
            Stroke::new(m(0.25, 0.5), m(0.5, 0.45), m(0.75, 0.45), m(0.85, 0.42)),
            // Bottom: 小 — 3 dots.
            Stroke::line(m(0.5, 0.4), m(0.5, 0.1)),
            Stroke::new(m(0.2, 0.3), m(0.25, 0.2), m(0.3, 0.15), m(0.35, 0.1)),
            Stroke::new(m(0.8, 0.3), m(0.75, 0.2), m(0.7, 0.15), m(0.65, 0.1)),
        ],
        // ---- 辶 (movement radical) ----------------------------------------
        '辶' => alloc::vec![
            Stroke::new(m(0.1, 0.85), m(0.15, 0.8), m(0.2, 0.78), m(0.22, 0.75)),
            Stroke::new(m(0.2, 0.7), m(0.25, 0.55), m(0.3, 0.4), m(0.4, 0.3)),
            Stroke::new(m(0.1, 0.3), m(0.3, 0.25), m(0.6, 0.2), m(0.9, 0.15)),
        ],
        // ---- 攵 (literary radical) -----------------------------------------
        '攵' => alloc::vec![
            Stroke::new(m(0.55, 0.85), m(0.45, 0.7), m(0.3, 0.55), m(0.15, 0.45)),
            Stroke::line(m(0.2, 0.7), m(0.85, 0.65)),
            Stroke::new(m(0.55, 0.65), m(0.4, 0.45), m(0.25, 0.25), m(0.1, 0.1)),
            Stroke::new(m(0.55, 0.65), m(0.7, 0.45), m(0.85, 0.25), m(0.95, 0.1)),
        ],
        // ---- 斤 (axe radical) ----------------------------------------------
        '斤' => alloc::vec![
            Stroke::new(m(0.2, 0.85), m(0.4, 0.8), m(0.6, 0.78), m(0.75, 0.78)),
            Stroke::line(m(0.75, 0.85), m(0.75, 0.15)),
            Stroke::line(m(0.4, 0.85), m(0.4, 0.15)),
            Stroke::line(m(0.4, 0.55), m(0.85, 0.55)),
        ],
        // ---- 貝 (shell) ----------------------------------------------------
        '貝' => alloc::vec![
            Stroke::line(m(0.2, 0.95), m(0.8, 0.95)),
            Stroke::line(m(0.8, 0.95), m(0.8, 0.4)),
            Stroke::line(m(0.2, 0.4), m(0.8, 0.4)),
            Stroke::line(m(0.2, 0.95), m(0.2, 0.4)),
            Stroke::line(m(0.2, 0.78), m(0.8, 0.78)),
            Stroke::line(m(0.2, 0.6), m(0.8, 0.6)),
            // 八 bottom
            Stroke::new(m(0.35, 0.4), m(0.25, 0.25), m(0.15, 0.15), m(0.1, 0.1)),
            Stroke::new(m(0.65, 0.4), m(0.75, 0.25), m(0.85, 0.15), m(0.9, 0.1)),
        ],
        // ---- 見 (see) -- 目 + 儿 -------------------------------------------
        '見' => alloc::vec![
            // 目 part (top)
            Stroke::line(m(0.2, 0.95), m(0.8, 0.95)),
            Stroke::line(m(0.8, 0.95), m(0.8, 0.45)),
            Stroke::line(m(0.2, 0.45), m(0.8, 0.45)),
            Stroke::line(m(0.2, 0.95), m(0.2, 0.45)),
            Stroke::line(m(0.2, 0.8), m(0.8, 0.8)),
            Stroke::line(m(0.2, 0.65), m(0.8, 0.65)),
            // Legs (儿)
            Stroke::new(m(0.3, 0.45), m(0.25, 0.3), m(0.2, 0.18), m(0.15, 0.1)),
            Stroke::new(m(0.6, 0.45), m(0.7, 0.3), m(0.85, 0.18), m(0.95, 0.18)),
        ],
        // ---- 車 (cart) -----------------------------------------------------
        '車' => alloc::vec![
            Stroke::line(m(0.1, 0.85), m(0.9, 0.85)),
            Stroke::line(m(0.2, 0.72), m(0.8, 0.72)),
            Stroke::line(m(0.2, 0.72), m(0.2, 0.35)),
            Stroke::line(m(0.8, 0.72), m(0.8, 0.35)),
            Stroke::line(m(0.2, 0.35), m(0.8, 0.35)),
            Stroke::line(m(0.5, 0.95), m(0.5, 0.05)),
            Stroke::line(m(0.1, 0.2), m(0.9, 0.2)),
        ],
        // ---- 門 (gate) -----------------------------------------------------
        '門' => alloc::vec![
            // Left door
            Stroke::line(m(0.1, 0.95), m(0.4, 0.95)),
            Stroke::line(m(0.1, 0.95), m(0.1, 0.1)),
            Stroke::line(m(0.1, 0.8), m(0.4, 0.8)),
            Stroke::line(m(0.4, 0.95), m(0.4, 0.6)),
            Stroke::line(m(0.1, 0.6), m(0.4, 0.6)),
            // Right door
            Stroke::line(m(0.6, 0.95), m(0.9, 0.95)),
            Stroke::line(m(0.9, 0.95), m(0.9, 0.1)),
            Stroke::line(m(0.6, 0.8), m(0.9, 0.8)),
            Stroke::line(m(0.6, 0.95), m(0.6, 0.6)),
            Stroke::line(m(0.6, 0.6), m(0.9, 0.6)),
        ],
        // ---- 宀 (roof) -----------------------------------------------------
        '宀' => alloc::vec![
            Stroke::line(m(0.5, 0.95), m(0.5, 0.85)),
            Stroke::new(m(0.1, 0.7), m(0.3, 0.78), m(0.7, 0.78), m(0.9, 0.7)),
            Stroke::line(m(0.1, 0.7), m(0.1, 0.55)),
            Stroke::line(m(0.9, 0.7), m(0.9, 0.55)),
        ],
        // ---- 立 / 米 / 弓 / 牛 / 矢 / 戈 / 殳 -----------------------------
        '立' => alloc::vec![
            Stroke::line(m(0.5, 0.95), m(0.5, 0.85)),
            Stroke::line(m(0.2, 0.7), m(0.8, 0.7)),
            Stroke::new(m(0.4, 0.55), m(0.42, 0.45), m(0.42, 0.35), m(0.4, 0.25)),
            Stroke::new(m(0.6, 0.55), m(0.58, 0.45), m(0.58, 0.35), m(0.6, 0.25)),
            Stroke::line(m(0.1, 0.15), m(0.9, 0.15)),
        ],
        '米' => alloc::vec![
            Stroke::new(m(0.3, 0.95), m(0.35, 0.85), m(0.4, 0.78), m(0.45, 0.72)),
            Stroke::new(m(0.7, 0.95), m(0.65, 0.85), m(0.6, 0.78), m(0.55, 0.72)),
            Stroke::line(m(0.1, 0.55), m(0.9, 0.55)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::new(m(0.5, 0.55), m(0.4, 0.4), m(0.3, 0.25), m(0.15, 0.1)),
            Stroke::new(m(0.5, 0.55), m(0.6, 0.4), m(0.7, 0.25), m(0.85, 0.1)),
        ],
        '弓' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.65)),
            Stroke::line(m(0.2, 0.65), m(0.8, 0.65)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.4)),
            Stroke::line(m(0.2, 0.4), m(0.8, 0.4)),
            Stroke::new(m(0.2, 0.4), m(0.3, 0.25), m(0.5, 0.15), m(0.7, 0.1)),
        ],
        '牛' => alloc::vec![
            Stroke::line(m(0.3, 0.92), m(0.55, 0.85)),
            Stroke::line(m(0.2, 0.7), m(0.8, 0.7)),
            Stroke::line(m(0.15, 0.45), m(0.85, 0.45)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.1)),
        ],
        '矢' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.65, 0.78)),
            Stroke::line(m(0.55, 0.92), m(0.5, 0.55)),
            Stroke::line(m(0.1, 0.55), m(0.9, 0.55)),
            Stroke::new(m(0.5, 0.55), m(0.35, 0.4), m(0.2, 0.25), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.55), m(0.6, 0.4), m(0.75, 0.25), m(0.9, 0.1)),
        ],
        '戈' => alloc::vec![
            Stroke::line(m(0.15, 0.78), m(0.85, 0.7)),
            Stroke::new(m(0.85, 0.85), m(0.75, 0.6), m(0.65, 0.35), m(0.5, 0.1)),
            Stroke::new(m(0.5, 0.4), m(0.6, 0.3), m(0.7, 0.25), m(0.8, 0.25)),
            Stroke::line(m(0.7, 0.95), m(0.78, 0.85)),
        ],
        '殳' => alloc::vec![
            // Top: small dash + curve.
            Stroke::line(m(0.25, 0.92), m(0.5, 0.85)),
            Stroke::new(m(0.5, 0.85), m(0.6, 0.7), m(0.65, 0.6), m(0.6, 0.5)),
            // Bottom: 又
            Stroke::new(m(0.15, 0.45), m(0.4, 0.4), m(0.6, 0.35), m(0.85, 0.3)),
            Stroke::new(m(0.3, 0.45), m(0.45, 0.3), m(0.6, 0.2), m(0.85, 0.1)),
        ],
        // ---- 各 (used in 略, 客, 絡, 等) -----------------------------------
        '各' => alloc::vec![
            // Top: 夂
            Stroke::new(m(0.5, 0.95), m(0.35, 0.8), m(0.2, 0.65), m(0.1, 0.55)),
            Stroke::new(m(0.5, 0.85), m(0.6, 0.75), m(0.7, 0.65), m(0.85, 0.55)),
            Stroke::new(m(0.85, 0.55), m(0.75, 0.7), m(0.4, 0.65), m(0.2, 0.55)),
            // Bottom: 口
            Stroke::line(m(0.25, 0.45), m(0.75, 0.45)),
            Stroke::line(m(0.75, 0.45), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.1), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.45), m(0.25, 0.1)),
        ],
        // ---- 重 -----------------------------------------------------------
        '重' => alloc::vec![
            Stroke::line(m(0.35, 0.92), m(0.65, 0.85)),
            Stroke::line(m(0.15, 0.75), m(0.85, 0.75)),
            Stroke::line(m(0.25, 0.62), m(0.75, 0.62)),
            Stroke::line(m(0.25, 0.62), m(0.25, 0.35)),
            Stroke::line(m(0.75, 0.62), m(0.75, 0.35)),
            Stroke::line(m(0.25, 0.48), m(0.75, 0.48)),
            Stroke::line(m(0.25, 0.35), m(0.75, 0.35)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.18), m(0.9, 0.18)),
        ],
        // ---- 隹 (short bird) -----------------------------------------------
        '隹' => alloc::vec![
            Stroke::line(m(0.3, 0.92), m(0.5, 0.85)),
            Stroke::line(m(0.2, 0.78), m(0.85, 0.78)),
            Stroke::line(m(0.2, 0.65), m(0.85, 0.65)),
            Stroke::line(m(0.2, 0.5), m(0.85, 0.5)),
            Stroke::line(m(0.2, 0.35), m(0.85, 0.35)),
            Stroke::line(m(0.2, 0.18), m(0.85, 0.18)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.18)),
            Stroke::line(m(0.2, 0.5), m(0.2, 0.18)),
        ],
        // ---- 頁 (page) -----------------------------------------------------
        '頁' => alloc::vec![
            Stroke::line(m(0.2, 0.95), m(0.8, 0.95)),
            Stroke::line(m(0.15, 0.8), m(0.85, 0.8)),
            Stroke::line(m(0.85, 0.8), m(0.8, 0.35)),
            Stroke::line(m(0.15, 0.8), m(0.2, 0.35)),
            Stroke::line(m(0.2, 0.35), m(0.8, 0.35)),
            Stroke::line(m(0.25, 0.65), m(0.75, 0.65)),
            Stroke::line(m(0.25, 0.5), m(0.75, 0.5)),
            Stroke::new(m(0.35, 0.35), m(0.25, 0.2), m(0.15, 0.1), m(0.1, 0.1)),
            Stroke::new(m(0.65, 0.35), m(0.75, 0.2), m(0.85, 0.1), m(0.9, 0.1)),
        ],
        // ---- 歹 / 戸 / 元 / 西 -----------------------------------------
        '歹' => alloc::vec![
            Stroke::line(m(0.2, 0.92), m(0.8, 0.92)),
            Stroke::line(m(0.5, 0.92), m(0.5, 0.7)),
            Stroke::line(m(0.2, 0.7), m(0.8, 0.7)),
            Stroke::new(m(0.4, 0.7), m(0.3, 0.45), m(0.2, 0.25), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.7), m(0.55, 0.45), m(0.6, 0.25), m(0.7, 0.1)),
        ],
        '戸' => alloc::vec![
            Stroke::line(m(0.3, 0.95), m(0.85, 0.95)),
            Stroke::line(m(0.1, 0.78), m(0.85, 0.78)),
            Stroke::line(m(0.85, 0.78), m(0.85, 0.45)),
            Stroke::line(m(0.1, 0.45), m(0.85, 0.45)),
            Stroke::new(m(0.5, 0.78), m(0.4, 0.55), m(0.25, 0.3), m(0.1, 0.1)),
        ],
        '元' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.2, 0.6), m(0.8, 0.6)),
            Stroke::new(m(0.4, 0.6), m(0.32, 0.4), m(0.25, 0.2), m(0.15, 0.1)),
            Stroke::new(m(0.6, 0.6), m(0.7, 0.35), m(0.85, 0.2), m(0.9, 0.18)),
        ],
        '西' => alloc::vec![
            Stroke::line(m(0.15, 0.9), m(0.85, 0.9)),
            Stroke::line(m(0.15, 0.9), m(0.15, 0.15)),
            Stroke::line(m(0.85, 0.9), m(0.85, 0.15)),
            Stroke::line(m(0.15, 0.15), m(0.85, 0.15)),
            Stroke::line(m(0.15, 0.55), m(0.85, 0.55)),
            Stroke::line(m(0.35, 0.55), m(0.35, 0.15)),
            Stroke::line(m(0.65, 0.55), m(0.65, 0.15)),
        ],
        // ---- 也 (used in 他, 地, 池, 等) -----------------------------------
        '也' => alloc::vec![
            Stroke::line(m(0.15, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.3, 0.85), m(0.3, 0.2)),
            Stroke::line(m(0.3, 0.2), m(0.85, 0.15)),
            Stroke::line(m(0.85, 0.75), m(0.85, 0.15)),
            Stroke::line(m(0.6, 0.8), m(0.55, 0.5)),
        ],
        // ---- 本 / 末 ------------------------------------------------------
        '本' => alloc::vec![
            Stroke::line(m(0.5, 0.92), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.62), m(0.9, 0.62)),
            Stroke::new(m(0.5, 0.5), m(0.35, 0.35), m(0.2, 0.2), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.5), m(0.65, 0.35), m(0.8, 0.2), m(0.9, 0.1)),
            Stroke::line(m(0.3, 0.18), m(0.7, 0.18)),
        ],
        '末' => alloc::vec![
            Stroke::line(m(0.1, 0.85), m(0.9, 0.85)),
            Stroke::line(m(0.15, 0.62), m(0.85, 0.62)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.1)),
            Stroke::new(m(0.5, 0.5), m(0.35, 0.35), m(0.2, 0.2), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.5), m(0.65, 0.35), m(0.8, 0.2), m(0.9, 0.1)),
        ],
        // ---- Frequently-used Leaf kanji (Joyo) -----------------------------
        '上' => alloc::vec![
            Stroke::line(m(0.2, 0.62), m(0.8, 0.62)),
            Stroke::line(m(0.5, 0.62), m(0.5, 0.18)),
            Stroke::line(m(0.1, 0.18), m(0.9, 0.18)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.62)),
        ],
        '世' => alloc::vec![
            Stroke::line(m(0.1, 0.82), m(0.9, 0.82)),
            Stroke::line(m(0.25, 0.82), m(0.25, 0.18)),
            Stroke::line(m(0.5, 0.82), m(0.5, 0.18)),
            Stroke::line(m(0.75, 0.82), m(0.75, 0.18)),
            Stroke::line(m(0.1, 0.18), m(0.9, 0.18)),
        ],
        '了' => alloc::vec![
            Stroke::new(m(0.2, 0.85), m(0.45, 0.9), m(0.7, 0.85), m(0.85, 0.75)),
            Stroke::new(m(0.55, 0.85), m(0.5, 0.55), m(0.45, 0.25), m(0.3, 0.1)),
        ],
        '予' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::new(m(0.6, 0.85), m(0.5, 0.65), m(0.4, 0.45), m(0.25, 0.3)),
            Stroke::line(m(0.25, 0.55), m(0.75, 0.55)),
            Stroke::new(m(0.55, 0.55), m(0.5, 0.35), m(0.45, 0.2), m(0.35, 0.1)),
        ],
        '事' => alloc::vec![
            Stroke::line(m(0.2, 0.92), m(0.8, 0.92)),
            Stroke::line(m(0.1, 0.78), m(0.9, 0.78)),
            Stroke::line(m(0.2, 0.65), m(0.8, 0.65)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
            Stroke::line(m(0.1, 0.35), m(0.9, 0.35)),
            Stroke::line(m(0.5, 0.92), m(0.5, 0.1)),
            Stroke::new(m(0.5, 0.1), m(0.4, 0.05), m(0.3, 0.1), m(0.25, 0.2)),
        ],
        '今' => alloc::vec![
            Stroke::new(m(0.5, 0.95), m(0.3, 0.75), m(0.15, 0.55), m(0.1, 0.5)),
            Stroke::new(m(0.5, 0.95), m(0.7, 0.75), m(0.85, 0.55), m(0.9, 0.5)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
            Stroke::line(m(0.45, 0.4), m(0.45, 0.15)),
            Stroke::line(m(0.3, 0.15), m(0.65, 0.15)),
        ],
        '以' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.18, 0.1)),
            Stroke::line(m(0.25, 0.5), m(0.4, 0.5)),
            Stroke::new(m(0.55, 0.85), m(0.6, 0.55), m(0.65, 0.25), m(0.7, 0.1)),
            Stroke::new(m(0.7, 0.1), m(0.78, 0.3), m(0.85, 0.55), m(0.9, 0.85)),
        ],
        '共' => alloc::vec![
            Stroke::line(m(0.25, 0.85), m(0.25, 0.55)),
            Stroke::line(m(0.75, 0.85), m(0.75, 0.55)),
            Stroke::line(m(0.1, 0.55), m(0.9, 0.55)),
            Stroke::line(m(0.5, 0.55), m(0.5, 0.25)),
            Stroke::line(m(0.2, 0.25), m(0.8, 0.25)),
            Stroke::new(m(0.3, 0.25), m(0.25, 0.18), m(0.2, 0.12), m(0.1, 0.1)),
            Stroke::new(m(0.7, 0.25), m(0.75, 0.18), m(0.8, 0.12), m(0.9, 0.1)),
        ],
        '出' => alloc::vec![
            Stroke::line(m(0.5, 0.92), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.5), m(0.9, 0.5)),
            Stroke::line(m(0.1, 0.5), m(0.1, 0.1)),
            Stroke::line(m(0.9, 0.5), m(0.9, 0.1)),
            Stroke::line(m(0.35, 0.75), m(0.35, 0.5)),
            Stroke::line(m(0.65, 0.75), m(0.65, 0.5)),
            Stroke::line(m(0.1, 0.1), m(0.9, 0.1)),
        ],
        '先' => alloc::vec![
            Stroke::line(m(0.35, 0.9), m(0.5, 0.85)),
            Stroke::line(m(0.1, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.5, 0.85), m(0.45, 0.55)),
            Stroke::line(m(0.15, 0.55), m(0.85, 0.5)),
            Stroke::new(m(0.4, 0.5), m(0.32, 0.35), m(0.22, 0.2), m(0.12, 0.1)),
            Stroke::new(m(0.6, 0.5), m(0.7, 0.35), m(0.85, 0.22), m(0.92, 0.2)),
        ],
        '失' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.6, 0.78)),
            Stroke::line(m(0.55, 0.92), m(0.5, 0.6)),
            Stroke::line(m(0.1, 0.55), m(0.9, 0.55)),
            Stroke::new(m(0.5, 0.55), m(0.35, 0.4), m(0.2, 0.25), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.55), m(0.6, 0.4), m(0.75, 0.25), m(0.9, 0.1)),
        ],
        '文' => alloc::vec![
            Stroke::new(m(0.45, 0.95), m(0.5, 0.9), m(0.55, 0.88), m(0.55, 0.85)),
            Stroke::line(m(0.15, 0.75), m(0.85, 0.7)),
            Stroke::new(m(0.5, 0.7), m(0.4, 0.45), m(0.25, 0.25), m(0.1, 0.1)),
            Stroke::new(m(0.4, 0.55), m(0.55, 0.4), m(0.75, 0.25), m(0.9, 0.1)),
        ],
        '比' => alloc::vec![
            // Two 匕 side by side.
            Stroke::new(m(0.2, 0.85), m(0.15, 0.6), m(0.1, 0.4), m(0.08, 0.25)),
            Stroke::line(m(0.08, 0.25), m(0.4, 0.35)),
            Stroke::line(m(0.4, 0.35), m(0.38, 0.1)),
            Stroke::new(m(0.65, 0.85), m(0.6, 0.6), m(0.55, 0.4), m(0.55, 0.25)),
            Stroke::line(m(0.55, 0.25), m(0.9, 0.35)),
            Stroke::line(m(0.9, 0.35), m(0.85, 0.1)),
        ],
        '正' => alloc::vec![
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.4, 0.85), m(0.4, 0.55)),
            Stroke::line(m(0.4, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.4, 0.55), m(0.85, 0.55)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.15)),
            Stroke::line(m(0.1, 0.15), m(0.9, 0.15)),
        ],
        '生' => alloc::vec![
            Stroke::line(m(0.3, 0.95), m(0.55, 0.85)),
            Stroke::line(m(0.15, 0.7), m(0.85, 0.7)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.1), m(0.9, 0.1)),
        ],
        '由' => alloc::vec![
            Stroke::line(m(0.5, 0.95), m(0.5, 0.78)),
            Stroke::line(m(0.2, 0.78), m(0.8, 0.78)),
            Stroke::line(m(0.8, 0.78), m(0.8, 0.18)),
            Stroke::line(m(0.2, 0.18), m(0.8, 0.18)),
            Stroke::line(m(0.2, 0.78), m(0.2, 0.18)),
            Stroke::line(m(0.5, 0.78), m(0.5, 0.18)),
            Stroke::line(m(0.2, 0.48), m(0.8, 0.48)),
        ],
        '用' => alloc::vec![
            Stroke::line(m(0.2, 0.9), m(0.8, 0.9)),
            Stroke::line(m(0.8, 0.9), m(0.78, 0.1)),
            Stroke::line(m(0.2, 0.9), m(0.22, 0.1)),
            Stroke::line(m(0.2, 0.65), m(0.8, 0.65)),
            Stroke::line(m(0.2, 0.4), m(0.8, 0.4)),
            Stroke::line(m(0.5, 0.9), m(0.5, 0.1)),
        ],
        '目' => alloc::vec![
            Stroke::line(m(0.25, 0.92), m(0.75, 0.92)),
            Stroke::line(m(0.75, 0.92), m(0.75, 0.08)),
            Stroke::line(m(0.25, 0.08), m(0.75, 0.08)),
            Stroke::line(m(0.25, 0.92), m(0.25, 0.08)),
            Stroke::line(m(0.25, 0.65), m(0.75, 0.65)),
            Stroke::line(m(0.25, 0.4), m(0.75, 0.4)),
        ],
        '自' => alloc::vec![
            Stroke::new(m(0.45, 0.97), m(0.45, 0.92), m(0.45, 0.9), m(0.4, 0.88)),
            Stroke::line(m(0.25, 0.85), m(0.75, 0.85)),
            Stroke::line(m(0.75, 0.85), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.1), m(0.75, 0.1)),
            Stroke::line(m(0.25, 0.85), m(0.25, 0.1)),
            Stroke::line(m(0.25, 0.6), m(0.75, 0.6)),
            Stroke::line(m(0.25, 0.4), m(0.75, 0.4)),
        ],
        '行' => alloc::vec![
            // 彳 (left) + 亍 (right)
            Stroke::new(m(0.18, 0.92), m(0.15, 0.85), m(0.13, 0.78), m(0.13, 0.72)),
            Stroke::new(m(0.3, 0.92), m(0.25, 0.78), m(0.18, 0.6), m(0.12, 0.5)),
            Stroke::line(m(0.18, 0.7), m(0.18, 0.1)),
            // 亍 (right side)
            Stroke::line(m(0.5, 0.85), m(0.85, 0.78)),
            Stroke::line(m(0.45, 0.55), m(0.9, 0.5)),
            Stroke::line(m(0.65, 0.78), m(0.65, 0.1)),
        ],
        '金' => alloc::vec![
            // 人 top + 王 below with dots
            Stroke::new(m(0.5, 0.95), m(0.35, 0.78), m(0.2, 0.62), m(0.12, 0.55)),
            Stroke::new(m(0.5, 0.95), m(0.65, 0.78), m(0.8, 0.62), m(0.88, 0.55)),
            Stroke::line(m(0.2, 0.55), m(0.8, 0.55)),
            Stroke::line(m(0.25, 0.4), m(0.75, 0.4)),
            Stroke::line(m(0.5, 0.55), m(0.5, 0.1)),
            Stroke::line(m(0.15, 0.1), m(0.85, 0.1)),
            Stroke::new(m(0.3, 0.5), m(0.28, 0.42), m(0.28, 0.36), m(0.3, 0.3)),
            Stroke::new(m(0.7, 0.5), m(0.72, 0.42), m(0.72, 0.36), m(0.7, 0.3)),
        ],
        '長' => alloc::vec![
            Stroke::line(m(0.3, 0.92), m(0.85, 0.92)),
            Stroke::line(m(0.85, 0.92), m(0.85, 0.5)),
            Stroke::line(m(0.3, 0.5), m(0.85, 0.5)),
            Stroke::line(m(0.3, 0.92), m(0.3, 0.5)),
            Stroke::line(m(0.3, 0.78), m(0.85, 0.78)),
            Stroke::line(m(0.3, 0.65), m(0.85, 0.65)),
            Stroke::line(m(0.5, 0.5), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.4), m(0.9, 0.4)),
            Stroke::new(m(0.5, 0.4), m(0.4, 0.25), m(0.25, 0.18), m(0.1, 0.1)),
            Stroke::new(m(0.5, 0.4), m(0.6, 0.25), m(0.75, 0.18), m(0.9, 0.1)),
        ],
        '高' => alloc::vec![
            Stroke::line(m(0.45, 0.97), m(0.55, 0.92)),
            Stroke::line(m(0.15, 0.8), m(0.85, 0.8)),
            Stroke::line(m(0.3, 0.7), m(0.7, 0.7)),
            Stroke::line(m(0.7, 0.7), m(0.7, 0.5)),
            Stroke::line(m(0.3, 0.5), m(0.7, 0.5)),
            Stroke::line(m(0.3, 0.7), m(0.3, 0.5)),
            // 口 below
            Stroke::line(m(0.2, 0.4), m(0.8, 0.4)),
            Stroke::line(m(0.8, 0.4), m(0.8, 0.05)),
            Stroke::line(m(0.2, 0.05), m(0.8, 0.05)),
            Stroke::line(m(0.2, 0.4), m(0.2, 0.05)),
            Stroke::line(m(0.2, 0.25), m(0.8, 0.25)),
            Stroke::line(m(0.2, 0.13), m(0.8, 0.13)),
        ],
        '面' => alloc::vec![
            Stroke::line(m(0.5, 0.95), m(0.5, 0.85)),
            Stroke::line(m(0.15, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.85, 0.85), m(0.85, 0.1)),
            Stroke::line(m(0.15, 0.85), m(0.15, 0.1)),
            Stroke::line(m(0.15, 0.1), m(0.85, 0.1)),
            Stroke::line(m(0.15, 0.6), m(0.85, 0.6)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.1)),
            Stroke::line(m(0.15, 0.35), m(0.85, 0.35)),
        ],
        '気' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.85, 0.85), m(0.85, 0.65)),
            Stroke::line(m(0.15, 0.7), m(0.85, 0.65)),
            Stroke::new(m(0.2, 0.85), m(0.15, 0.55), m(0.15, 0.3), m(0.2, 0.1)),
            Stroke::line(m(0.35, 0.55), m(0.7, 0.5)),
            Stroke::line(m(0.4, 0.4), m(0.7, 0.35)),
            Stroke::new(m(0.4, 0.25), m(0.5, 0.35), m(0.6, 0.4), m(0.85, 0.4)),
        ],
        '真' => alloc::vec![
            Stroke::line(m(0.5, 0.95), m(0.5, 0.85)),
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.2, 0.7), m(0.8, 0.7)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.4)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.4)),
            Stroke::line(m(0.2, 0.55), m(0.8, 0.55)),
            Stroke::line(m(0.2, 0.4), m(0.8, 0.4)),
            Stroke::line(m(0.1, 0.25), m(0.9, 0.25)),
            Stroke::new(m(0.3, 0.25), m(0.25, 0.15), m(0.2, 0.1), m(0.15, 0.1)),
            Stroke::new(m(0.7, 0.25), m(0.75, 0.15), m(0.8, 0.1), m(0.85, 0.1)),
        ],
        '数' => alloc::vec![
            // 米 left side approx + 攵 right side.
            Stroke::line(m(0.1, 0.85), m(0.4, 0.85)),
            Stroke::line(m(0.1, 0.65), m(0.4, 0.65)),
            Stroke::line(m(0.25, 0.92), m(0.25, 0.45)),
            Stroke::new(m(0.25, 0.55), m(0.18, 0.4), m(0.12, 0.3), m(0.08, 0.2)),
            Stroke::new(m(0.25, 0.55), m(0.32, 0.4), m(0.38, 0.3), m(0.42, 0.2)),
            // 攵 right
            Stroke::new(m(0.75, 0.92), m(0.65, 0.75), m(0.55, 0.6), m(0.5, 0.55)),
            Stroke::line(m(0.5, 0.7), m(0.9, 0.65)),
            Stroke::new(m(0.7, 0.65), m(0.6, 0.45), m(0.5, 0.25), m(0.45, 0.1)),
            Stroke::new(m(0.7, 0.65), m(0.78, 0.45), m(0.85, 0.25), m(0.92, 0.1)),
        ],
        '無' => alloc::vec![
            Stroke::line(m(0.1, 0.92), m(0.9, 0.92)),
            Stroke::line(m(0.25, 0.92), m(0.25, 0.4)),
            Stroke::line(m(0.45, 0.92), m(0.45, 0.4)),
            Stroke::line(m(0.65, 0.92), m(0.65, 0.4)),
            Stroke::line(m(0.85, 0.92), m(0.85, 0.4)),
            Stroke::line(m(0.1, 0.75), m(0.9, 0.75)),
            Stroke::line(m(0.1, 0.58), m(0.9, 0.58)),
            Stroke::line(m(0.1, 0.4), m(0.9, 0.4)),
            // 灬 bottom dots
            Stroke::new(m(0.2, 0.3), m(0.22, 0.2), m(0.22, 0.15), m(0.2, 0.1)),
            Stroke::new(m(0.4, 0.3), m(0.42, 0.2), m(0.42, 0.15), m(0.4, 0.1)),
            Stroke::new(m(0.6, 0.3), m(0.58, 0.2), m(0.58, 0.15), m(0.6, 0.1)),
            Stroke::new(m(0.8, 0.3), m(0.78, 0.2), m(0.78, 0.15), m(0.8, 0.1)),
        ],
        // ---- placeholder fallback ------------------------------------------
        _ => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.15)),
            Stroke::line(m(0.2, 0.15), m(0.8, 0.15)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.15)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_returns_empty_with_advance() {
        let sdf = generate('龥', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn registered_kanji_have_inside_pixels() {
        let params = MetaFontParams::sans_regular();
        // extoria-website で実使用される登録済み漢字から代表的なものを選択。
        for ch in ['明', '品', '加', '仕', '体', '部', '思', '記', '時'] {
            let sdf = generate(ch, &params);
            assert!(sdf.advance > 0.0, "{ch}: advance positive");
            let inside = sdf.data.iter().any(|d| *d < 0.0);
            assert!(inside, "{ch}: should have inside pixels after composition");
            for d in sdf.data {
                assert!(d.is_finite(), "{ch}: SDF must be finite");
            }
        }
    }

    #[test]
    fn mei_kanji_renders_two_components() {
        // 明 = ⿰日月 — 5 (日) + 5 (月) = 10 strokes, all should fit in 16-slot skeleton.
        let sdf = generate('明', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
    }

    #[test]
    fn nested_kanji_renders() {
        // 品 = ⿱口⿰口口 — depth 2 nesting (3 occurrences of 口).
        let sdf = generate('品', &MetaFontParams::sans_regular());
        assert!(sdf.advance > 0.0);
        let inside = sdf.data.iter().any(|d| *d < 0.0);
        assert!(inside);
    }

    #[test]
    fn bbox_split_left_right_halves() {
        let (l, r) = split_binary(Bbox::unit(), CompositionLayout::LeftRight);
        assert!((l.w - 0.5).abs() < 1e-5);
        assert!((r.x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn bbox_split_top_bottom_first_is_top() {
        let (t, b) = split_binary(Bbox::unit(), CompositionLayout::TopBottom);
        assert!((t.y - 0.5).abs() < 1e-5);
        assert!(b.y.abs() < 1e-5);
    }

    #[test]
    fn bbox_split_ternary_left_mid_right() {
        let (a, b, c) = split_ternary(Bbox::unit(), CompositionLayout::LeftMidRight);
        assert!((a.w - 1.0 / 3.0).abs() < 1e-4);
        assert!((b.x - 1.0 / 3.0).abs() < 1e-4);
        assert!((c.x - 2.0 / 3.0).abs() < 1e-4);
    }
}
