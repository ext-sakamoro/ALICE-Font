//! DSL 経由で定義した MVP 55+ radical + 19 composed kanji library。
//!
//! v0.7.4-alpha 拡張: 35 radical 追加 (合計 50 primitive)、11 composed
//! 追加 (合計 15)。全 char が real Bezier stroke を出力。
//!
//! # Coverage
//!
//! Primitive radicals (50):
//! - 数字 (13): 一 二 三 四 五 六 七 八 九 十 百 千 万
//! - 方向 (7): 上 下 中 前 後 左 右
//! - 自然 (20): 木 日 月 田 口 目 山 水 火 土 川 石 天 空 花 草 雨 雪 白 青
//! - 人体・動作 (11): 心 大 人 力 女 子 手 足 耳 見 走
//! - 構造 (4): 立 米 糸 車
//!
//! Composed via IDS (15):
//! - 明 = ⿰ 日 月、林 = ⿰ 木 木、森 = ⿱ 木 ⿰ 木 木、相 = ⿰ 木 目
//! - 男 = ⿱ 田 力、好 = ⿰ 女 子、岩 = ⿱ 山 石、早 = ⿱ 日 十
//! - 音 = ⿱ 立 日、意 = ⿱ 音 心、想 = ⿱ 相 心、志 = ⿱ 士 心 (士 は 十 approx)
//! - 半 = ⿱ 八 干 (approx)、雲 = ⿱ 雨 云 (approx)、電 = ⿱ 雨 电 (approx)
//!
//! # DSL vs `kanji::build_skeleton` の比較
//!
//! v0.7.0-alpha の `kanji::build_skeleton` は `radicals.rs` の CjkRadical
//! (partial Bezier) + `ids_db.rs` (798 Leaf placeholder) 依存。
//! 本 module は **手書き DSL 定義** で 20 char が確実に real render 可能。
//! Consumer は `dsl_lookup(ch).or_else(|| kanji::build_skeleton(ch))` で
//! DSL 優先 fallback pattern が使える。
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

use super::compose::{left_right, top_bottom};
use super::stroke::{dot, hook, horizontal, left_falling, right_falling, rising, vertical};
use super::{glyph, DEFAULT_PEN_WIDTH};
use crate::skeleton_shader::SkeletonGlyph;

// ============================================================================
// 数字系
// ============================================================================

/// **一** — 1 stroke horizontal line at middle。
///
/// v0.7.2-alpha: taper (起筆 thin → 中央 → 収筆 heavy) 楷書 typical。
#[must_use]
pub fn ichi() -> SkeletonGlyph {
    glyph('一')
        .add_variable(horizontal(0.5, 0.1, 0.9), 0.04, 0.07)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **二** — 2 horizontal lines (top thin、bottom heavy)。
#[must_use]
pub fn ni() -> SkeletonGlyph {
    glyph('二')
        .add_variable(horizontal(0.7, 0.15, 0.85), 0.035, 0.055) // 上 (short + thin)
        .add_variable(horizontal(0.3, 0.1, 0.9), 0.04, 0.07) // 下 (long + heavy)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **三** — 3 horizontal lines (top narrow、bottom wide、progressive weight)。
#[must_use]
pub fn san() -> SkeletonGlyph {
    glyph('三')
        .add_variable(horizontal(0.8, 0.2, 0.8), 0.03, 0.05)
        .add_variable(horizontal(0.5, 0.25, 0.75), 0.035, 0.055)
        .add_variable(horizontal(0.2, 0.1, 0.9), 0.04, 0.075)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **十** — cross。horizontal + vertical。縦棒 が主 (太)、横棒 は副 (細)。
#[must_use]
pub fn juu() -> SkeletonGlyph {
    glyph('十')
        .add_variable(horizontal(0.5, 0.1, 0.9), 0.035, 0.055) // 横棒 taper
        .add_variable(vertical(0.5, 0.1, 0.9), 0.06, 0.05) // 縦棒 (太 → 底 hook 予兆)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **四** — four。□ + 内 short 儿 (2 short vertical)。
#[must_use]
pub fn yon() -> SkeletonGlyph {
    glyph('四')
        .add_variable(horizontal(0.85, 0.2, 0.8), 0.04, 0.06) // 上辺
        .add_variable(vertical(0.2, 0.15, 0.85), 0.06, 0.05) // 左辺
        .add_variable(vertical(0.8, 0.15, 0.85), 0.06, 0.05) // 右辺
        .add_variable(horizontal(0.15, 0.2, 0.8), 0.045, 0.065) // 下辺
        .add_variable(vertical(0.4, 0.3, 0.75), 0.045, 0.035) // 内 left 短縦
        .add_variable(left_falling((0.6, 0.75), (0.65, 0.3)), 0.04, 0.055) // 内 right 短撇
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **五** — five。top horizontal + 折 turn + bottom horizontal。
#[must_use]
pub fn go() -> SkeletonGlyph {
    glyph('五')
        .add_variable(horizontal(0.85, 0.15, 0.85), 0.04, 0.06) // 上横棒
        .add_variable(vertical(0.25, 0.35, 0.85), 0.055, 0.05) // 左縦 (upper)
        .add_variable(horizontal(0.55, 0.25, 0.85), 0.04, 0.055) // 中央横
        .add_variable(vertical(0.75, 0.15, 0.55), 0.055, 0.05) // 右縦 (下 half)
        .add_variable(horizontal(0.15, 0.1, 0.9), 0.05, 0.07) // 下横棒
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **六** — six。亠 (top hat) + 八 (bottom legs)。
#[must_use]
pub fn roku() -> SkeletonGlyph {
    glyph('六')
        .add_variable(dot((0.5, 0.85), 0.1), 0.06, 0.045) // 上点
        .add_variable(horizontal(0.65, 0.1, 0.9), 0.045, 0.065) // 一 (下 of 亠)
        .add_variable(left_falling((0.35, 0.5), (0.15, 0.1)), 0.055, 0.02) // 左撇
        .add_variable(right_falling((0.65, 0.5), (0.85, 0.1)), 0.025, 0.075) // 右点
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **七** — seven。1 horizontal + hook (乙 系)。
#[must_use]
pub fn shichi() -> SkeletonGlyph {
    glyph('七')
        .add_variable(horizontal(0.65, 0.1, 0.9), 0.045, 0.065) // 上横棒
        .add_variable(vertical(0.4, 0.15, 0.65), 0.055, 0.05) // 中央縦 (short)
        .add_variable(hook((0.4, 0.15), (0.85, 0.15), (0.05, 0.1)), 0.05, 0.045) // 底 hook 右上
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **八** — eight。left 撇 + right 捺 (人 に類似だが wider)。
#[must_use]
pub fn hachi() -> SkeletonGlyph {
    glyph('八')
        .add_variable(left_falling((0.4, 0.8), (0.1, 0.15)), 0.06, 0.025) // 左撇
        .add_variable(right_falling((0.6, 0.8), (0.9, 0.15)), 0.025, 0.08) // 右点
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **九** — nine。撇 + 折 hook。
#[must_use]
pub fn kyuu() -> SkeletonGlyph {
    glyph('九')
        .add_variable(left_falling((0.3, 0.85), (0.15, 0.15)), 0.06, 0.03) // 左撇
        .add_variable(horizontal(0.7, 0.15, 0.85), 0.045, 0.055) // 折 top 部
        .add_variable(hook((0.85, 0.7), (0.85, 0.2), (-0.1, 0.05)), 0.055, 0.06) // 右縦 hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **百** — hundred。一 (top) + 白 pattern (rectangle + inner horizontal)。
#[must_use]
pub fn hyaku() -> SkeletonGlyph {
    glyph('百')
        .add_variable(horizontal(0.9, 0.15, 0.85), 0.045, 0.07) // 上横棒
        .add_variable(horizontal(0.7, 0.2, 0.8), 0.04, 0.06) // 白 上辺
        .add_variable(vertical(0.2, 0.1, 0.7), 0.055, 0.05) // 左辺
        .add_variable(vertical(0.8, 0.1, 0.7), 0.055, 0.05) // 右辺
        .add_variable(horizontal(0.1, 0.2, 0.8), 0.045, 0.065) // 下辺
        .add_variable(horizontal(0.4, 0.2, 0.8), 0.04, 0.055) // 中央横
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **千** — thousand。撇 + 一 + 十 の base。
#[must_use]
pub fn sen() -> SkeletonGlyph {
    glyph('千')
        .add_variable(left_falling((0.55, 0.9), (0.35, 0.75)), 0.06, 0.03) // 上撇
        .add_variable(horizontal(0.7, 0.1, 0.9), 0.045, 0.065) // 一 (横棒)
        .add_variable(vertical(0.5, 0.15, 0.7), 0.06, 0.05) // 縦棒
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **万** — ten thousand。一 + 折 + 撇。
#[must_use]
pub fn man() -> SkeletonGlyph {
    glyph('万')
        .add_variable(horizontal(0.75, 0.1, 0.9), 0.045, 0.065) // 上横棒
        .add_variable(vertical(0.25, 0.4, 0.75), 0.055, 0.05) // 左縦 (short)
        .add_variable(hook((0.25, 0.4), (0.85, 0.4), (0.0, -0.15)), 0.055, 0.05) // 折 (right + down)
        .add_variable(left_falling((0.55, 0.4), (0.15, 0.1)), 0.05, 0.025) // 左下撇
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// 方向・位置
// ============================================================================

/// **上** — up。short horizontal (top) + vertical + long horizontal (bottom)。
#[must_use]
pub fn ue() -> SkeletonGlyph {
    glyph('上')
        .add_variable(horizontal(0.6, 0.4, 0.75), 0.045, 0.06) // 上 short 横
        .add_variable(vertical(0.4, 0.2, 0.85), 0.065, 0.05) // 中央縦 (太)
        .add_variable(horizontal(0.2, 0.1, 0.9), 0.05, 0.07) // 下横棒 (heavy)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **下** — down。long horizontal (top) + vertical + dot。
#[must_use]
pub fn shita() -> SkeletonGlyph {
    glyph('下')
        .add_variable(horizontal(0.8, 0.1, 0.9), 0.05, 0.07) // 上横棒 (heavy)
        .add_variable(vertical(0.5, 0.15, 0.8), 0.065, 0.05) // 中央縦
        .add_variable(dot((0.7, 0.45), 0.1), 0.06, 0.045) // 右点
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **中** — middle。口 (rectangle) + vertical through middle。
#[must_use]
pub fn naka() -> SkeletonGlyph {
    glyph('中')
        .add_variable(horizontal(0.75, 0.3, 0.7), 0.04, 0.06) // 上辺
        .add_variable(vertical(0.3, 0.4, 0.75), 0.055, 0.05) // 左辺
        .add_variable(vertical(0.7, 0.4, 0.75), 0.055, 0.05) // 右辺
        .add_variable(horizontal(0.4, 0.3, 0.7), 0.04, 0.06) // 下辺
        .add_variable(vertical(0.5, 0.1, 0.9), 0.07, 0.055) // 中央貫通縦棒 (太)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **前** — front。simplified: 八 (top) + 一 + 月 (bottom)。
#[must_use]
pub fn mae() -> SkeletonGlyph {
    glyph('前')
        .add_variable(left_falling((0.4, 0.9), (0.3, 0.75)), 0.05, 0.025) // 上 左点
        .add_variable(right_falling((0.6, 0.9), (0.7, 0.75)), 0.025, 0.055) // 上 右点
        .add_variable(horizontal(0.7, 0.1, 0.9), 0.045, 0.065) // 上横棒
        .add_variable(horizontal(0.55, 0.3, 0.75), 0.04, 0.055) // 上 中央 short 横
        .add_variable(vertical(0.3, 0.1, 0.55), 0.055, 0.05) // 左縦 (月 部)
        .add_variable(vertical(0.75, 0.1, 0.55), 0.055, 0.05) // 右縦
        .add_variable(horizontal(0.35, 0.3, 0.75), 0.04, 0.055) // 内 horizontal
        .add_variable(horizontal(0.15, 0.3, 0.75), 0.045, 0.06) // 下 horizontal
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **後** — back (simplified 彳 + tuple)。
#[must_use]
pub fn ushiro() -> SkeletonGlyph {
    glyph('後')
        .add_variable(left_falling((0.2, 0.9), (0.1, 0.7)), 0.05, 0.03) // 彳 上撇
        .add_variable(left_falling((0.2, 0.7), (0.1, 0.5)), 0.045, 0.03) // 彳 中撇
        .add_variable(vertical(0.15, 0.1, 0.55), 0.055, 0.05) // 彳 縦
        .add_variable(left_falling((0.55, 0.85), (0.4, 0.7)), 0.05, 0.03) // 幺 上
        .add_variable(hook((0.4, 0.7), (0.8, 0.7), (0.0, -0.1)), 0.05, 0.045) // 幺 折
        .add_variable(vertical(0.7, 0.4, 0.7), 0.05, 0.045) // 幺 縦
        .add_variable(right_falling((0.5, 0.35), (0.85, 0.1)), 0.03, 0.075) // 夂 右下捺
        .add_variable(left_falling((0.55, 0.35), (0.35, 0.1)), 0.055, 0.025) // 夂 左下撇
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **左** — left。一 + 撇 + 工。
#[must_use]
pub fn hidari() -> SkeletonGlyph {
    glyph('左')
        .add_variable(horizontal(0.85, 0.15, 0.75), 0.045, 0.065) // 上横棒
        .add_variable(left_falling((0.55, 0.85), (0.1, 0.4)), 0.06, 0.025) // 左撇
        .add_variable(horizontal(0.35, 0.15, 0.75), 0.04, 0.055) // 工 上横
        .add_variable(vertical(0.45, 0.1, 0.35), 0.055, 0.05) // 工 縦
        .add_variable(horizontal(0.1, 0.15, 0.85), 0.05, 0.07) // 工 下横 (heavy)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **右** — right。一 + 撇 + 口。
#[must_use]
pub fn migi() -> SkeletonGlyph {
    glyph('右')
        .add_variable(horizontal(0.85, 0.15, 0.75), 0.045, 0.065) // 上横棒
        .add_variable(left_falling((0.5, 0.85), (0.15, 0.4)), 0.06, 0.03) // 左撇
        .add_variable(horizontal(0.4, 0.35, 0.85), 0.04, 0.055) // 口 上辺
        .add_variable(vertical(0.35, 0.1, 0.4), 0.055, 0.05) // 口 左辺
        .add_variable(vertical(0.85, 0.1, 0.4), 0.055, 0.05) // 口 右辺
        .add_variable(horizontal(0.1, 0.35, 0.85), 0.045, 0.06) // 口 下辺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// 自然 radicals
// ============================================================================

/// **木** — tree radical。主幹 + 上横 + 左撇 + 右捺。
///
/// v0.7.2-alpha: 主幹 heavy、上横 balanced、撇 taper thin、捺 heavy end。
#[must_use]
pub fn ki() -> SkeletonGlyph {
    glyph('木')
        .add_variable(horizontal(0.75, 0.1, 0.9), 0.04, 0.06) // 上横棒 taper
        .add_variable(vertical(0.5, 0.15, 0.9), 0.07, 0.055) // 主幹 (太、bottom taper)
        .add_variable(left_falling((0.5, 0.55), (0.15, 0.15)), 0.06, 0.02) // 撇 tapered
        .add_variable(right_falling((0.5, 0.55), (0.85, 0.15)), 0.03, 0.08) // 捺 heavy end
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **日** — sun / day radical。□ + 中央 horizontal。
#[must_use]
pub fn hi() -> SkeletonGlyph {
    glyph('日')
        .add(horizontal(0.9, 0.2, 0.8)) // 上辺
        .add(vertical(0.2, 0.2, 0.9)) // 左辺
        .add(vertical(0.8, 0.2, 0.9)) // 右辺
        .add(horizontal(0.2, 0.2, 0.8)) // 下辺
        .add(horizontal(0.55, 0.2, 0.8)) // 中央
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **月** — moon / month radical。日 よりわずかに narrow、内 2 横線。
#[must_use]
pub fn tsuki() -> SkeletonGlyph {
    glyph('月')
        .add(horizontal(0.9, 0.25, 0.75)) // 上辺
        .add(vertical(0.25, 0.1, 0.9)) // 左辺 (下は 少し延長)
        .add(vertical(0.75, 0.2, 0.9)) // 右辺
        .add(horizontal(0.65, 0.25, 0.75)) // 内 1
        .add(horizontal(0.4, 0.25, 0.75)) // 内 2
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **田** — rice field radical。□ + cross。
#[must_use]
pub fn ta() -> SkeletonGlyph {
    glyph('田')
        .add(horizontal(0.9, 0.15, 0.85)) // 上辺
        .add(vertical(0.15, 0.15, 0.9)) // 左辺
        .add(vertical(0.85, 0.15, 0.9)) // 右辺
        .add(horizontal(0.15, 0.15, 0.85)) // 下辺
        .add(horizontal(0.525, 0.15, 0.85)) // 中央 horizontal
        .add(vertical(0.5, 0.15, 0.9)) // 中央 vertical
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **口** — mouth radical。single □。
#[must_use]
pub fn kuchi() -> SkeletonGlyph {
    glyph('口')
        .add(horizontal(0.75, 0.25, 0.75)) // 上辺
        .add(vertical(0.25, 0.25, 0.75)) // 左辺
        .add(vertical(0.75, 0.25, 0.75)) // 右辺
        .add(horizontal(0.25, 0.25, 0.75)) // 下辺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **目** — eye radical。□ + 3 horizontal lines (日 より tall + 2 内線)。
#[must_use]
pub fn me() -> SkeletonGlyph {
    glyph('目')
        .add(horizontal(0.9, 0.25, 0.75)) // 上辺
        .add(vertical(0.25, 0.1, 0.9)) // 左辺
        .add(vertical(0.75, 0.1, 0.9)) // 右辺
        .add(horizontal(0.1, 0.25, 0.75)) // 下辺
        .add(horizontal(0.67, 0.25, 0.75)) // 内 1
        .add(horizontal(0.43, 0.25, 0.75)) // 内 2
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **山** — mountain radical。base + 3 peaks。中央 peak 最太。
#[must_use]
pub fn yama() -> SkeletonGlyph {
    glyph('山')
        .add_variable(horizontal(0.2, 0.1, 0.9), 0.045, 0.065) // 底辺 (heavy base)
        .add_variable(vertical(0.5, 0.2, 0.9), 0.075, 0.05) // 中央 peak (tallest + thick)
        .add_variable(vertical(0.15, 0.2, 0.55), 0.055, 0.04) // 左 peak
        .add_variable(vertical(0.85, 0.2, 0.55), 0.055, 0.04) // 右 peak
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **水** — water radical。中央 hook + 4 短 strokes。
#[must_use]
pub fn mizu() -> SkeletonGlyph {
    glyph('水')
        .add(hook((0.5, 0.9), (0.5, 0.2), (-0.08, 0.05))) // 中央 hook
        .add(left_falling((0.55, 0.7), (0.2, 0.3))) // 左上 撇
        .add(right_falling((0.55, 0.5), (0.75, 0.15))) // 右下 捺
        .add(rising((0.15, 0.3), (0.35, 0.55))) // 左下 提
        .add(left_falling((0.7, 0.55), (0.85, 0.4))) // 右上 短撇
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **火** — fire radical。2 dots + 撇捺 (人 の下に 2 点)。
#[must_use]
pub fn hi_fire() -> SkeletonGlyph {
    glyph('火')
        .add(dot((0.25, 0.75), 0.12)) // 左 dot
        .add(dot((0.75, 0.75), 0.12)) // 右 dot
        .add(left_falling((0.5, 0.75), (0.2, 0.1))) // 中央左 撇
        .add(right_falling((0.5, 0.55), (0.85, 0.1))) // 中央右 捺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **土** — earth / soil radical。2 horizontal + 1 vertical (十 系だが下辺 wider)。
#[must_use]
pub fn tsuchi() -> SkeletonGlyph {
    glyph('土')
        .add_variable(horizontal(0.6, 0.25, 0.75), 0.045, 0.055) // 中央 horizontal (shorter)
        .add_variable(vertical(0.5, 0.2, 0.9), 0.065, 0.055) // 中央 vertical
        .add_variable(horizontal(0.2, 0.1, 0.9), 0.045, 0.065) // 底辺 (widest)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **川** — river。3 verticals (left short + middle straight + right hook)。
#[must_use]
pub fn kawa() -> SkeletonGlyph {
    glyph('川')
        .add_variable(left_falling((0.2, 0.85), (0.15, 0.15)), 0.06, 0.045) // 左撇 (short)
        .add_variable(vertical(0.5, 0.15, 0.85), 0.065, 0.055) // 中央縦
        .add_variable(hook((0.85, 0.85), (0.85, 0.15), (-0.08, 0.06)), 0.06, 0.05) // 右 hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **石** — stone。一 + 撇 + 口。
#[must_use]
pub fn ishi() -> SkeletonGlyph {
    glyph('石')
        .add_variable(horizontal(0.85, 0.2, 0.85), 0.045, 0.065) // 上横棒
        .add_variable(left_falling((0.6, 0.85), (0.15, 0.4)), 0.06, 0.03) // 左撇
        .add_variable(horizontal(0.45, 0.4, 0.85), 0.04, 0.055) // 口 上辺
        .add_variable(vertical(0.4, 0.1, 0.45), 0.055, 0.05) // 口 左辺
        .add_variable(vertical(0.85, 0.1, 0.45), 0.055, 0.05) // 口 右辺
        .add_variable(horizontal(0.1, 0.4, 0.85), 0.045, 0.06) // 口 下辺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **天** — heaven。一 + 大 (下部)。
#[must_use]
pub fn ten() -> SkeletonGlyph {
    glyph('天')
        .add_variable(horizontal(0.85, 0.15, 0.85), 0.045, 0.065) // 上横棒
        .add_variable(horizontal(0.65, 0.2, 0.8), 0.04, 0.06) // 大 上横 (少し short)
        .add_variable(left_falling((0.5, 0.6), (0.15, 0.1)), 0.06, 0.025) // 左撇
        .add_variable(right_falling((0.5, 0.6), (0.85, 0.1)), 0.025, 0.075) // 右捺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **空** — sky / empty。穴 (roof) + 工 (bottom)。
#[must_use]
pub fn sora() -> SkeletonGlyph {
    glyph('空')
        .add_variable(dot((0.5, 0.9), 0.08), 0.055, 0.045) // 上点 (roof)
        .add_variable(horizontal(0.75, 0.1, 0.9), 0.045, 0.065) // 冖 上横
        .add_variable(vertical(0.1, 0.55, 0.75), 0.055, 0.05) // 冖 左短縦
        .add_variable(vertical(0.9, 0.55, 0.75), 0.055, 0.05) // 冖 右短縦
        .add_variable(left_falling((0.35, 0.6), (0.25, 0.4)), 0.05, 0.03) // 八 左点
        .add_variable(right_falling((0.65, 0.6), (0.75, 0.4)), 0.03, 0.055) // 八 右点
        .add_variable(horizontal(0.35, 0.25, 0.75), 0.04, 0.05) // 工 上横
        .add_variable(vertical(0.5, 0.1, 0.35), 0.06, 0.05) // 工 中央縦
        .add_variable(horizontal(0.1, 0.15, 0.85), 0.05, 0.07) // 工 下横 (heavy)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **花** — flower。艹 (grass crown) + 化 (下部)。
#[must_use]
pub fn hana() -> SkeletonGlyph {
    glyph('花')
        .add_variable(horizontal(0.85, 0.1, 0.9), 0.045, 0.06) // 艹 横棒
        .add_variable(vertical(0.3, 0.7, 0.95), 0.05, 0.045) // 艹 左縦
        .add_variable(vertical(0.7, 0.7, 0.95), 0.05, 0.045) // 艹 右縦
        .add_variable(left_falling((0.35, 0.7), (0.15, 0.15)), 0.06, 0.025) // 亻 左撇
        .add_variable(vertical(0.4, 0.15, 0.7), 0.06, 0.05) // 亻 縦
        .add_variable(left_falling((0.75, 0.7), (0.55, 0.35)), 0.055, 0.03) // ヒ 左撇
        .add_variable(hook((0.55, 0.35), (0.85, 0.35), (0.0, 0.1)), 0.05, 0.045) // ヒ 折 hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **草** — grass。艹 + 早 (下部、simplified)。
#[must_use]
pub fn kusa() -> SkeletonGlyph {
    glyph('草')
        .add_variable(horizontal(0.9, 0.1, 0.9), 0.045, 0.06) // 艹 横棒
        .add_variable(vertical(0.3, 0.75, 1.0), 0.05, 0.045) // 艹 左縦
        .add_variable(vertical(0.7, 0.75, 1.0), 0.05, 0.045) // 艹 右縦
        .add_variable(horizontal(0.75, 0.2, 0.8), 0.04, 0.06) // 日 上辺
        .add_variable(vertical(0.2, 0.4, 0.75), 0.055, 0.05) // 日 左辺
        .add_variable(vertical(0.8, 0.4, 0.75), 0.055, 0.05) // 日 右辺
        .add_variable(horizontal(0.4, 0.2, 0.8), 0.04, 0.06) // 日 下辺
        .add_variable(horizontal(0.575, 0.2, 0.8), 0.035, 0.05) // 日 中央
        .add_variable(horizontal(0.25, 0.1, 0.9), 0.05, 0.07) // 十 一
        .add_variable(vertical(0.5, 0.05, 0.4), 0.065, 0.055) // 十 縦
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **雨** — rain。一 + 冂 + | + 4 dots。
#[must_use]
pub fn ame() -> SkeletonGlyph {
    glyph('雨')
        .add_variable(horizontal(0.95, 0.15, 0.85), 0.045, 0.06) // 上横棒
        .add_variable(horizontal(0.75, 0.1, 0.9), 0.05, 0.07) // 冂 上辺
        .add_variable(vertical(0.15, 0.15, 0.75), 0.055, 0.05) // 冂 左辺
        .add_variable(vertical(0.85, 0.15, 0.75), 0.055, 0.05) // 冂 右辺
        .add_variable(vertical(0.5, 0.15, 0.75), 0.055, 0.05) // 中央縦
        .add_variable(dot((0.3, 0.5), 0.07), 0.055, 0.045) // 内 左上 dot
        .add_variable(dot((0.7, 0.5), 0.07), 0.055, 0.045) // 内 右上 dot
        .add_variable(dot((0.3, 0.3), 0.07), 0.055, 0.045) // 内 左下 dot
        .add_variable(dot((0.7, 0.3), 0.07), 0.055, 0.045) // 内 右下 dot
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **雪** — snow (approximation)。雨-top + 彐-bottom (broken lines)。
#[must_use]
pub fn yuki() -> SkeletonGlyph {
    glyph('雪')
        .add_variable(horizontal(0.95, 0.2, 0.8), 0.045, 0.06) // 上横棒
        .add_variable(horizontal(0.8, 0.1, 0.9), 0.05, 0.07) // 冂 上
        .add_variable(vertical(0.15, 0.45, 0.8), 0.055, 0.05) // 冂 左
        .add_variable(vertical(0.85, 0.45, 0.8), 0.055, 0.05) // 冂 右
        .add_variable(dot((0.4, 0.6), 0.06), 0.055, 0.045) // 内 dot 1
        .add_variable(dot((0.6, 0.6), 0.06), 0.055, 0.045) // 内 dot 2
        .add_variable(horizontal(0.4, 0.15, 0.85), 0.045, 0.065) // 彐 上
        .add_variable(vertical(0.15, 0.1, 0.4), 0.055, 0.05) // 彐 左
        .add_variable(horizontal(0.25, 0.15, 0.85), 0.04, 0.06) // 彐 中
        .add_variable(horizontal(0.1, 0.15, 0.85), 0.05, 0.07) // 彐 下
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **白** — white。撇 + 日 pattern。
#[must_use]
pub fn shiro() -> SkeletonGlyph {
    glyph('白')
        .add_variable(left_falling((0.5, 0.95), (0.35, 0.85)), 0.055, 0.03) // 撇 (top hat)
        .add_variable(horizontal(0.85, 0.25, 0.75), 0.045, 0.06) // 上辺
        .add_variable(vertical(0.25, 0.15, 0.85), 0.055, 0.05) // 左辺
        .add_variable(vertical(0.75, 0.15, 0.85), 0.055, 0.05) // 右辺
        .add_variable(horizontal(0.15, 0.25, 0.75), 0.045, 0.06) // 下辺
        .add_variable(horizontal(0.5, 0.25, 0.75), 0.04, 0.055) // 中央
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **青** — blue。龶 (top: 亠 + 士) + 月 (bottom)。
#[must_use]
pub fn ao() -> SkeletonGlyph {
    glyph('青')
        .add_variable(horizontal(0.9, 0.2, 0.8), 0.045, 0.06) // 上横棒
        .add_variable(horizontal(0.75, 0.15, 0.85), 0.045, 0.065) // 中央 上横
        .add_variable(vertical(0.5, 0.55, 0.9), 0.065, 0.055) // 中央縦
        .add_variable(horizontal(0.55, 0.2, 0.8), 0.045, 0.065) // 龶 下横
        .add_variable(horizontal(0.5, 0.2, 0.8), 0.045, 0.06) // 月 上辺
        .add_variable(vertical(0.2, 0.1, 0.5), 0.055, 0.05) // 月 左辺
        .add_variable(vertical(0.8, 0.1, 0.5), 0.055, 0.05) // 月 右辺
        .add_variable(horizontal(0.35, 0.2, 0.8), 0.04, 0.055) // 月 内 1
        .add_variable(horizontal(0.15, 0.2, 0.8), 0.045, 0.06) // 月 下辺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// 人体 radicals
// ============================================================================

/// **心** — heart radical。3 dots + 1 hook。dots は round-heavy。
#[must_use]
pub fn kokoro() -> SkeletonGlyph {
    glyph('心')
        .add_variable(dot((0.25, 0.7), 0.1), 0.06, 0.05) // 左 dot
        .add_variable(hook((0.5, 0.7), (0.5, 0.25), (0.15, 0.1)), 0.07, 0.05) // 中央 hook 太
        .add_variable(dot((0.7, 0.55), 0.08), 0.05, 0.045) // 右上 dot
        .add_variable(dot((0.85, 0.7), 0.1), 0.06, 0.05) // 右 dot
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **大** — big radical。一 + 撇捺 (人 の上に 一)。
#[must_use]
pub fn dai() -> SkeletonGlyph {
    glyph('大')
        .add_variable(horizontal(0.7, 0.15, 0.85), 0.045, 0.065) // 上 horizontal taper
        .add_variable(left_falling((0.5, 0.7), (0.15, 0.1)), 0.06, 0.025) // 左撇 tapered
        .add_variable(right_falling((0.5, 0.7), (0.85, 0.1)), 0.03, 0.08) // 右捺 heavy end
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **人** — person radical。単純 撇捺 (書道 pen taper 強調)。
#[must_use]
pub fn hito() -> SkeletonGlyph {
    glyph('人')
        .add_variable(left_falling((0.5, 0.85), (0.15, 0.1)), 0.07, 0.02) // 左撇 taper
        .add_variable(right_falling((0.5, 0.85), (0.85, 0.1)), 0.03, 0.09) // 右捺 heavy
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **力** — power radical。折 + 撇 (2 stroke)。
#[must_use]
pub fn chikara() -> SkeletonGlyph {
    glyph('力')
        // 折 (top-left → top-right → bottom-right hook via single curved stroke)
        .add(hook((0.2, 0.85), (0.75, 0.85), (-0.05, -0.15))) // 折の実質: 上辺 + 右下 hook
        .add(vertical(0.75, 0.15, 0.85)) // 右辺 (縦)
        .add(left_falling((0.3, 0.55), (0.15, 0.1))) // 左下 撇
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **女** — woman radical。撇 + 折 + 一 (3 stroke)。
#[must_use]
pub fn onna() -> SkeletonGlyph {
    glyph('女')
        .add(left_falling((0.55, 0.85), (0.15, 0.15))) // 左撇 (大 kanji tilt)
        .add(right_falling((0.4, 0.55), (0.85, 0.15))) // 右下 curved to 右下
        .add(horizontal(0.4, 0.15, 0.85)) // 中央 horizontal
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **子** — child radical。3 stroke (7 系 + 一)。
#[must_use]
pub fn ko() -> SkeletonGlyph {
    glyph('子')
        // 7-shape 上部 (簡略: 上 horizontal + 撇 curved)
        .add(horizontal(0.85, 0.2, 0.8)) // 上辺
        .add(left_falling((0.7, 0.85), (0.3, 0.5))) // 左上 curved
        .add(vertical(0.5, 0.1, 0.7)) // 中央 vertical
        .add(horizontal(0.55, 0.2, 0.8)) // 中央 horizontal
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **手** — hand radical。3 horizontals + hook。
#[must_use]
pub fn te() -> SkeletonGlyph {
    glyph('手')
        .add_variable(left_falling((0.55, 0.9), (0.35, 0.8)), 0.05, 0.03) // 上撇
        .add_variable(horizontal(0.75, 0.1, 0.9), 0.045, 0.065) // 上横
        .add_variable(horizontal(0.55, 0.15, 0.85), 0.04, 0.06) // 中央横
        .add_variable(horizontal(0.35, 0.1, 0.9), 0.05, 0.07) // 下横 (heavy)
        .add_variable(hook((0.5, 0.9), (0.5, 0.1), (-0.08, 0.06)), 0.07, 0.055) // 中央 vertical hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **足** — foot。口 (top) + 止 (bottom)。
#[must_use]
pub fn ashi() -> SkeletonGlyph {
    glyph('足')
        .add_variable(horizontal(0.95, 0.3, 0.7), 0.04, 0.055) // 口 上辺
        .add_variable(vertical(0.3, 0.6, 0.95), 0.055, 0.05) // 口 左
        .add_variable(vertical(0.7, 0.6, 0.95), 0.055, 0.05) // 口 右
        .add_variable(horizontal(0.6, 0.3, 0.7), 0.04, 0.055) // 口 下
        .add_variable(horizontal(0.4, 0.15, 0.85), 0.045, 0.065) // 止 上横
        .add_variable(vertical(0.4, 0.1, 0.6), 0.055, 0.05) // 止 左短縦
        .add_variable(vertical(0.6, 0.1, 0.55), 0.055, 0.05) // 止 中央縦
        .add_variable(horizontal(0.1, 0.1, 0.9), 0.05, 0.07) // 止 下横
        .add_variable(right_falling((0.5, 0.4), (0.85, 0.1)), 0.03, 0.075) // 止 右捺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **耳** — ear。rectangle + 3 horizontal lines。
#[must_use]
pub fn mimi() -> SkeletonGlyph {
    glyph('耳')
        .add_variable(horizontal(0.9, 0.25, 0.75), 0.045, 0.06) // 上辺
        .add_variable(vertical(0.25, 0.05, 0.9), 0.06, 0.05) // 左辺
        .add_variable(vertical(0.75, 0.1, 0.9), 0.055, 0.05) // 右辺
        .add_variable(horizontal(0.7, 0.25, 0.75), 0.04, 0.055) // 内 1
        .add_variable(horizontal(0.5, 0.25, 0.75), 0.04, 0.055) // 内 2
        .add_variable(horizontal(0.3, 0.25, 0.75), 0.04, 0.055) // 内 3
        .add_variable(horizontal(0.1, 0.15, 0.85), 0.05, 0.07) // 下横 (heavy, extends beyond)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **見** — see。目 + 儿 (legs) at bottom。
#[must_use]
pub fn miru() -> SkeletonGlyph {
    glyph('見')
        .add_variable(horizontal(0.95, 0.3, 0.7), 0.04, 0.06) // 目 上辺
        .add_variable(vertical(0.3, 0.4, 0.95), 0.055, 0.05) // 目 左辺
        .add_variable(vertical(0.7, 0.4, 0.95), 0.055, 0.05) // 目 右辺
        .add_variable(horizontal(0.4, 0.3, 0.7), 0.04, 0.055) // 目 下辺
        .add_variable(horizontal(0.78, 0.3, 0.7), 0.035, 0.05) // 目 内 1
        .add_variable(horizontal(0.6, 0.3, 0.7), 0.035, 0.05) // 目 内 2
        .add_variable(left_falling((0.5, 0.4), (0.2, 0.1)), 0.06, 0.03) // 儿 左撇
        .add_variable(hook((0.55, 0.4), (0.85, 0.15), (-0.06, 0.06)), 0.03, 0.055) // 儿 右 hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **走** — run。土 (top) + 龰 (bottom stride)。
#[must_use]
pub fn hashiru() -> SkeletonGlyph {
    glyph('走')
        .add_variable(horizontal(0.9, 0.2, 0.8), 0.045, 0.06) // 土 上横
        .add_variable(vertical(0.5, 0.55, 0.9), 0.065, 0.055) // 土 縦
        .add_variable(horizontal(0.55, 0.15, 0.85), 0.05, 0.07) // 土 下横 (heavy)
        .add_variable(left_falling((0.5, 0.55), (0.15, 0.2)), 0.06, 0.03) // 龰 左撇
        .add_variable(vertical(0.5, 0.15, 0.55), 0.055, 0.05) // 龰 中央縦 (short)
        .add_variable(horizontal(0.25, 0.15, 0.85), 0.045, 0.06) // 龰 中横
        .add_variable(right_falling((0.5, 0.2), (0.85, 0.05)), 0.03, 0.08) // 龰 右捺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// 構造 radicals
// ============================================================================

/// **立** — stand。亠 + 儿-like base。
#[must_use]
pub fn tatsu() -> SkeletonGlyph {
    glyph('立')
        .add_variable(dot((0.5, 0.95), 0.08), 0.055, 0.045) // 上点
        .add_variable(horizontal(0.8, 0.2, 0.8), 0.045, 0.065) // 上横
        .add_variable(left_falling((0.35, 0.6), (0.25, 0.4)), 0.05, 0.03) // 左点
        .add_variable(right_falling((0.65, 0.6), (0.75, 0.4)), 0.03, 0.055) // 右点
        .add_variable(vertical(0.5, 0.15, 0.6), 0.06, 0.05) // 中央縦
        .add_variable(horizontal(0.1, 0.1, 0.9), 0.05, 0.075) // 下横 (widest, heavy)
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **米** — rice。十 + 4 corner dots。
#[must_use]
pub fn kome() -> SkeletonGlyph {
    glyph('米')
        .add_variable(horizontal(0.5, 0.1, 0.9), 0.05, 0.07) // 十 一
        .add_variable(vertical(0.5, 0.1, 0.9), 0.065, 0.055) // 十 縦
        .add_variable(dot((0.25, 0.75), 0.08), 0.055, 0.045) // 左上点
        .add_variable(dot((0.75, 0.75), 0.08), 0.055, 0.045) // 右上点
        .add_variable(left_falling((0.35, 0.35), (0.15, 0.15)), 0.055, 0.03) // 左下 撇
        .add_variable(right_falling((0.65, 0.35), (0.85, 0.15)), 0.03, 0.06) // 右下 捺
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **糸** — thread。幺 (top) + 小 (bottom)。
#[must_use]
pub fn ito() -> SkeletonGlyph {
    glyph('糸')
        .add_variable(left_falling((0.55, 0.95), (0.35, 0.8)), 0.055, 0.03) // 幺 上撇
        .add_variable(hook((0.35, 0.8), (0.7, 0.8), (0.0, -0.1)), 0.05, 0.045) // 幺 折
        .add_variable(vertical(0.65, 0.5, 0.8), 0.055, 0.05) // 幺 縦
        .add_variable(left_falling((0.5, 0.5), (0.3, 0.35)), 0.055, 0.03) // 幺 下撇
        .add_variable(hook((0.3, 0.35), (0.7, 0.35), (0.0, -0.08)), 0.05, 0.045) // 幺 下折
        .add_variable(vertical(0.5, 0.05, 0.35), 0.065, 0.055) // 小 中央縦
        .add_variable(left_falling((0.4, 0.3), (0.15, 0.05)), 0.05, 0.025) // 小 左点
        .add_variable(right_falling((0.6, 0.3), (0.85, 0.05)), 0.025, 0.055) // 小 右点
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **車** — car / wheel。田 + top/bottom horizontals。
#[must_use]
pub fn kuruma() -> SkeletonGlyph {
    glyph('車')
        .add_variable(horizontal(0.9, 0.1, 0.9), 0.05, 0.07) // 上横 (extending)
        .add_variable(horizontal(0.75, 0.2, 0.8), 0.04, 0.06) // 田 上辺
        .add_variable(vertical(0.2, 0.2, 0.75), 0.055, 0.05) // 田 左辺
        .add_variable(vertical(0.8, 0.2, 0.75), 0.055, 0.05) // 田 右辺
        .add_variable(horizontal(0.475, 0.2, 0.8), 0.04, 0.055) // 田 中横
        .add_variable(horizontal(0.2, 0.2, 0.8), 0.04, 0.06) // 田 下辺
        .add_variable(horizontal(0.05, 0.1, 0.9), 0.05, 0.075) // 下横 (widest)
        .add_variable(vertical(0.5, 0.05, 0.9), 0.065, 0.055) // 中央貫通縦
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// Composed kanji (IDS operator 経由)
// ============================================================================

/// **明** — ⿰ 日 月。bright / clear。
///
/// Compose = left half 日 + right half 月。
#[must_use]
pub fn mei() -> SkeletonGlyph {
    left_right('明', 0.5, hi(), tsuki())
}

/// **林** — ⿰ 木 木。grove / woods。
#[must_use]
pub fn hayashi() -> SkeletonGlyph {
    left_right('林', 0.5, ki(), ki())
}

/// **森** — ⿱ 木 ⿰ 木 木。forest。3 木 (top + bottom-left + bottom-right)。
#[must_use]
pub fn mori() -> SkeletonGlyph {
    top_bottom('森', 0.5, ki(), left_right('_', 0.5, ki(), ki()))
}

/// **相** — ⿰ 木 目。mutual / phase。
#[must_use]
pub fn sou() -> SkeletonGlyph {
    left_right('相', 0.5, ki(), me())
}

/// **男** — ⿱ 田 力。man。
#[must_use]
pub fn otoko() -> SkeletonGlyph {
    top_bottom('男', 0.5, ta(), chikara())
}

/// **好** — ⿰ 女 子。like / good。
#[must_use]
pub fn suki() -> SkeletonGlyph {
    left_right('好', 0.5, onna(), ko())
}

/// **岩** — ⿱ 山 石。rock / cliff。
#[must_use]
pub fn iwa() -> SkeletonGlyph {
    top_bottom('岩', 0.5, yama(), ishi())
}

/// **早** — ⿱ 日 十。early / morning。
#[must_use]
pub fn hayai() -> SkeletonGlyph {
    top_bottom('早', 0.5, hi(), juu())
}

/// **音** — sound。v0.7.5: nested top_bottom を廃止して direct 11-stroke 定義。
///
/// 上部 立 (y > 0.5) + 下部 日 (y < 0.5)、compression 回避で書体密度改善。
#[must_use]
pub fn oto() -> SkeletonGlyph {
    glyph('音')
        // 立 top (y 0.55-1.0)
        .add_variable(dot((0.5, 0.95), 0.06), 0.055, 0.045) // 上点
        .add_variable(horizontal(0.9, 0.2, 0.8), 0.045, 0.06) // 立 上横
        .add_variable(left_falling((0.35, 0.78), (0.28, 0.66)), 0.05, 0.03) // 立 左点
        .add_variable(right_falling((0.65, 0.78), (0.72, 0.66)), 0.03, 0.055) // 立 右点
        .add_variable(vertical(0.5, 0.62, 0.78), 0.055, 0.05) // 立 中央縦
        .add_variable(horizontal(0.6, 0.15, 0.85), 0.05, 0.07) // 立 下横 (heavy)
        // 日 bottom (y 0-0.5)
        .add_variable(horizontal(0.48, 0.25, 0.75), 0.045, 0.06) // 日 上辺
        .add_variable(vertical(0.25, 0.05, 0.48), 0.055, 0.05) // 日 左辺
        .add_variable(vertical(0.75, 0.05, 0.48), 0.055, 0.05) // 日 右辺
        .add_variable(horizontal(0.05, 0.25, 0.75), 0.045, 0.06) // 日 下辺
        .add_variable(horizontal(0.26, 0.25, 0.75), 0.04, 0.055) // 日 中央
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **意** — meaning。v0.7.5: direct 16-stroke 定義 (nested triple composition 廃止)。
///
/// 上 立 (y 0.7-1.0) + 中 日 (y 0.4-0.7) + 下 心 (y 0-0.4) の 3 段。
#[must_use]
pub fn kokoro_meaning() -> SkeletonGlyph {
    glyph('意')
        // 立 top (y 0.72-1.0)
        .add_variable(dot((0.5, 0.97), 0.05), 0.05, 0.04) // 上点
        .add_variable(horizontal(0.93, 0.2, 0.8), 0.045, 0.055) // 立 上横
        .add_variable(vertical(0.5, 0.75, 0.85), 0.05, 0.045) // 立 中央 short 縦
        .add_variable(horizontal(0.75, 0.15, 0.85), 0.048, 0.062) // 立 下横
        // 日 middle (y 0.42-0.7)
        .add_variable(horizontal(0.68, 0.28, 0.72), 0.04, 0.055) // 日 上辺
        .add_variable(vertical(0.28, 0.42, 0.68), 0.05, 0.045) // 日 左辺
        .add_variable(vertical(0.72, 0.42, 0.68), 0.05, 0.045) // 日 右辺
        .add_variable(horizontal(0.42, 0.28, 0.72), 0.04, 0.055) // 日 下辺
        .add_variable(horizontal(0.55, 0.28, 0.72), 0.035, 0.05) // 日 中央
        // 心 bottom (y 0-0.4)
        .add_variable(dot((0.2, 0.28), 0.08), 0.055, 0.045) // 心 左 dot
        .add_variable(hook((0.5, 0.3), (0.5, 0.05), (0.15, 0.05)), 0.06, 0.05) // 心 中央 hook
        .add_variable(dot((0.7, 0.22), 0.07), 0.05, 0.045) // 心 右上 dot
        .add_variable(dot((0.85, 0.28), 0.08), 0.055, 0.045) // 心 右 dot
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **想** — thought。v0.7.5: direct 15-stroke 定義 (nested triple composition 廃止)。
///
/// 上 相 = 左 木 + 右 目 (y 0.4-1.0) + 下 心 (y 0-0.4)。
#[must_use]
pub fn omou() -> SkeletonGlyph {
    glyph('想')
        // 木 top-left (y 0.42-1.0、x 0-0.5)
        .add_variable(horizontal(0.85, 0.05, 0.45), 0.045, 0.06) // 上横
        .add_variable(vertical(0.25, 0.45, 0.98), 0.06, 0.05) // 主幹
        .add_variable(left_falling((0.25, 0.65), (0.05, 0.5)), 0.055, 0.03) // 撇
        .add_variable(right_falling((0.25, 0.65), (0.45, 0.5)), 0.03, 0.065) // 捺
        // 目 top-right (y 0.42-1.0、x 0.5-1.0)
        .add_variable(horizontal(0.95, 0.55, 0.9), 0.04, 0.055) // 目 上
        .add_variable(vertical(0.55, 0.45, 0.95), 0.05, 0.045) // 目 左
        .add_variable(vertical(0.9, 0.45, 0.95), 0.05, 0.045) // 目 右
        .add_variable(horizontal(0.45, 0.55, 0.9), 0.04, 0.055) // 目 下
        .add_variable(horizontal(0.78, 0.55, 0.9), 0.035, 0.05) // 目 内 1
        .add_variable(horizontal(0.62, 0.55, 0.9), 0.035, 0.05) // 目 内 2
        // 心 bottom (y 0-0.42)
        .add_variable(dot((0.2, 0.28), 0.08), 0.055, 0.045) // 左 dot
        .add_variable(hook((0.5, 0.3), (0.5, 0.05), (0.15, 0.05)), 0.06, 0.05) // 中央 hook
        .add_variable(dot((0.7, 0.22), 0.07), 0.05, 0.045) // 右上 dot
        .add_variable(dot((0.85, 0.28), 0.08), 0.055, 0.045) // 右 dot
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **志** — ⿱ 士 心 (士 approximated by 十)。will / ambition。
#[must_use]
pub fn kokorozashi() -> SkeletonGlyph {
    top_bottom('志', 0.5, juu(), kokoro())
}

/// **半** — ⿱ 八 十 (approximation, 干 系)。half。
#[must_use]
pub fn han() -> SkeletonGlyph {
    top_bottom('半', 0.55, hachi(), juu())
}

/// **雲** — cloud。v0.7.5: direct 12-stroke 定義 (nested top_bottom 廃止)。
///
/// 上 雨 (y 0.45-1.0、simplified 8 stroke) + 下 云 (y 0-0.45、4 stroke)。
#[must_use]
pub fn kumo() -> SkeletonGlyph {
    glyph('雲')
        // 雨 top (y 0.47-1.0、density 減らして 5 dot → 8 stroke)
        .add_variable(horizontal(0.97, 0.18, 0.82), 0.045, 0.058) // 上横
        .add_variable(horizontal(0.82, 0.12, 0.88), 0.05, 0.068) // 冂 上
        .add_variable(vertical(0.15, 0.47, 0.82), 0.052, 0.048) // 冂 左
        .add_variable(vertical(0.85, 0.47, 0.82), 0.052, 0.048) // 冂 右
        .add_variable(vertical(0.5, 0.47, 0.82), 0.052, 0.048) // 中央縦
        .add_variable(dot((0.32, 0.65), 0.06), 0.05, 0.04) // 左 dot
        .add_variable(dot((0.68, 0.65), 0.06), 0.05, 0.04) // 右 dot
        // 云 bottom (y 0-0.42、simplified as 二 style)
        .add_variable(horizontal(0.35, 0.15, 0.7), 0.045, 0.06) // 上横
        .add_variable(horizontal(0.2, 0.1, 0.55), 0.04, 0.055) // 下 slash
        .add_variable(left_falling((0.6, 0.35), (0.35, 0.05)), 0.05, 0.03) // 左撇
        .add_variable(right_falling((0.6, 0.15), (0.8, 0.02)), 0.03, 0.06) // 右点
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

/// **電** — electricity。v0.7.5: direct 15-stroke 定義 (nested top_bottom 廃止)。
///
/// 上 雨 (y 0.5-1.0、simplified) + 下 电/田 with descender hook (y 0-0.5)。
#[must_use]
pub fn den() -> SkeletonGlyph {
    glyph('電')
        // 雨 top (y 0.52-1.0)
        .add_variable(horizontal(0.97, 0.18, 0.82), 0.045, 0.058) // 上横
        .add_variable(horizontal(0.82, 0.12, 0.88), 0.05, 0.068) // 冂 上
        .add_variable(vertical(0.15, 0.55, 0.82), 0.052, 0.048) // 冂 左
        .add_variable(vertical(0.85, 0.55, 0.82), 0.052, 0.048) // 冂 右
        .add_variable(vertical(0.5, 0.55, 0.82), 0.052, 0.048) // 中央縦
        .add_variable(dot((0.32, 0.65), 0.055), 0.05, 0.04) // 左 dot
        .add_variable(dot((0.68, 0.65), 0.055), 0.05, 0.04) // 右 dot
        // 田 bottom (y 0.05-0.5)
        .add_variable(horizontal(0.48, 0.22, 0.78), 0.045, 0.06) // 田 上辺
        .add_variable(vertical(0.22, 0.1, 0.48), 0.05, 0.045) // 田 左辺
        .add_variable(vertical(0.78, 0.1, 0.48), 0.05, 0.045) // 田 右辺
        .add_variable(horizontal(0.29, 0.22, 0.78), 0.04, 0.055) // 田 中央 horizontal
        .add_variable(vertical(0.5, 0.1, 0.48), 0.05, 0.045) // 田 中央 vertical
        .add_variable(horizontal(0.1, 0.22, 0.78), 0.045, 0.06) // 田 下辺
        // 电 の descender hook (y < 0.05)
        .add_variable(hook((0.5, 0.1), (0.5, -0.05), (-0.1, 0.05)), 0.055, 0.04) // 下 hook
        .weight(DEFAULT_PEN_WIDTH)
        .build()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ichi_has_1_stroke_horizontal_at_middle() {
        let g = ichi();
        assert_eq!(g.codepoint, '一');
        assert_eq!(g.stroke_count, 1);
        // Stroke should be near y=0.5
        let s = &g.strokes[0];
        assert!((s.p0.y - 0.5).abs() < 0.01);
    }

    #[test]
    fn san_has_3_strokes() {
        let g = san();
        assert_eq!(g.stroke_count, 3);
    }

    #[test]
    fn ki_has_4_strokes_vertical_middle() {
        let g = ki();
        assert_eq!(g.stroke_count, 4);
        // 主幹 vertical should be at x=0.5
        let vertical_stroke = &g.strokes[1]; // 2nd stroke = vertical
        assert!((vertical_stroke.p0.x - 0.5).abs() < 0.01);
        assert!((vertical_stroke.p3.x - 0.5).abs() < 0.01);
    }

    #[test]
    fn mei_composes_hi_plus_tsuki() {
        let g = mei();
        assert_eq!(g.codepoint, '明');
        assert_eq!(g.stroke_count, hi().stroke_count + tsuki().stroke_count);
    }

    #[test]
    fn hayashi_composes_ki_plus_ki() {
        let g = hayashi();
        assert_eq!(g.stroke_count, ki().stroke_count * 2);
    }

    #[test]
    fn mori_composes_three_ki_within_max_strokes() {
        let g = mori();
        assert_eq!(g.codepoint, '森');
        let expected = ki().stroke_count * 3;
        assert_eq!(
            g.stroke_count,
            expected.min(crate::glyph::MAX_GLYPH_STROKES)
        );
    }

    #[test]
    fn all_mvp_radicals_have_multi_strokes() {
        // v0.7.4-alpha: 全 55 radical が 1 stroke 以上、MAX_GLYPH_STROKES 以下
        let radicals = [
            ichi(),
            ni(),
            san(),
            yon(),
            go(),
            roku(),
            shichi(),
            hachi(),
            kyuu(),
            juu(),
            hyaku(),
            sen(),
            man(),
            ue(),
            shita(),
            naka(),
            mae(),
            ushiro(),
            hidari(),
            migi(),
            ki(),
            hi(),
            tsuki(),
            ta(),
            kuchi(),
            me(),
            yama(),
            mizu(),
            hi_fire(),
            tsuchi(),
            kawa(),
            ishi(),
            ten(),
            sora(),
            hana(),
            kusa(),
            ame(),
            yuki(),
            shiro(),
            ao(),
            kokoro(),
            dai(),
            hito(),
            chikara(),
            onna(),
            ko(),
            te(),
            ashi(),
            mimi(),
            miru(),
            hashiru(),
            tatsu(),
            kome(),
            ito(),
            kuruma(),
        ];
        assert_eq!(radicals.len(), 55, "expected 55 radicals in v0.7.4");
        for r in radicals.iter() {
            assert!(r.stroke_count >= 1, "{:?} has 0 strokes", r.codepoint);
            assert!(
                r.stroke_count <= crate::glyph::MAX_GLYPH_STROKES,
                "{:?} exceeds MAX_GLYPH_STROKES",
                r.codepoint
            );
        }
    }

    #[test]
    fn all_composed_kanji_have_valid_strokes() {
        // v0.7.4-alpha: 15 composed kanji all render with valid stroke counts
        let composed = [
            mei(),
            hayashi(),
            mori(),
            sou(),
            otoko(),
            suki(),
            iwa(),
            hayai(),
            oto(),
            kokoro_meaning(),
            omou(),
            kokorozashi(),
            han(),
            kumo(),
            den(),
        ];
        assert_eq!(composed.len(), 15, "expected 15 composed in v0.7.4");
        for c in composed.iter() {
            assert!(
                c.stroke_count >= 2,
                "{:?} composed should have ≥2 strokes",
                c.codepoint
            );
        }
    }
}
