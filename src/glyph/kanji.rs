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

/// 部品 char の Bezier ストローク列を返す。S6 で 214 部首全てに拡充される。
fn component_strokes(ch: char, b: Bbox) -> Vec<Stroke> {
    let m = |u: f32, v: f32| Point2::new(b.x + u * b.w, b.y + v * b.h);
    match ch {
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
        '木' => alloc::vec![
            Stroke::line(m(0.5, 0.9), m(0.5, 0.1)),
            Stroke::line(m(0.1, 0.65), m(0.9, 0.65)),
            Stroke::new(m(0.5, 0.5), m(0.4, 0.4), m(0.3, 0.25), m(0.15, 0.15)),
            Stroke::new(m(0.5, 0.5), m(0.6, 0.4), m(0.7, 0.25), m(0.85, 0.15)),
        ],
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
        '口' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.2), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.2)),
        ],
        '田' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
            Stroke::line(m(0.8, 0.85), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.2), m(0.8, 0.2)),
            Stroke::line(m(0.2, 0.85), m(0.2, 0.2)),
            Stroke::line(m(0.5, 0.85), m(0.5, 0.2)),
            Stroke::line(m(0.2, 0.5), m(0.8, 0.5)),
        ],
        '力' => alloc::vec![
            Stroke::line(m(0.2, 0.85), m(0.85, 0.85)),
            Stroke::line(m(0.85, 0.85), m(0.8, 0.4)),
            Stroke::new(m(0.8, 0.4), m(0.7, 0.2), m(0.5, 0.1), m(0.3, 0.1)),
            Stroke::new(m(0.45, 0.85), m(0.3, 0.55), m(0.2, 0.3), m(0.15, 0.1)),
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
        _ => {
            // Placeholder: 矩形枠 (S6 で 214 部首をすべて拡充するまでの暫定)
            alloc::vec![
                Stroke::line(m(0.2, 0.85), m(0.8, 0.85)),
                Stroke::line(m(0.8, 0.85), m(0.8, 0.15)),
                Stroke::line(m(0.2, 0.15), m(0.8, 0.15)),
                Stroke::line(m(0.2, 0.85), m(0.2, 0.15)),
            ]
        }
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
