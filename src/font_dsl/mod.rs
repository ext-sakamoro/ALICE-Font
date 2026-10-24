//! Font DSL — Metafont-style declarative Bezier stroke builder for CJK glyphs。
//!
//! v0.7.1-alpha (2026-07-31、Approach 4):
//! Rust builder API で永字八法 8 primitive + IDS 12 operator を提供、
//! 同 DSL source から `SkeletonGlyph` を出力 (v0.7.0 skeleton shader 直結)。
//!
//! # Design goal
//!
//! - **ALICE 原理準拠**: 「法則を送れ」— DSL source が Rust const、compile-time 展開
//! - **Content reusable**: `木` 定義 1 度で `森 / 林 / 相 / 校 / 松` で再利用
//! - **Zero-asset compliant**: 出力は既存 v0.7.0 skeleton path infrastructure に流れる
//!
//! # Example
//!
//! ```
//! use alice_font::font_dsl::stroke::*;
//! use alice_font::font_dsl::glyph;
//!
//! let ichi = glyph('一')
//!     .add(horizontal(0.5, 0.1, 0.9))
//!     .weight(0.05)
//!     .build();
//!
//! assert_eq!(ichi.stroke_count, 1);
//! ```
//!
//! SPDX-License-Identifier: LicenseRef-Proprietary
//! Author: Moroya Sakamoto

extern crate alloc;
use alloc::vec::Vec;

use crate::glyph::MAX_GLYPH_STROKES;
use crate::skeleton_shader::SkeletonGlyph;
use crate::stroke::{Point2, Stroke};

pub mod compose;
pub mod radicals;
pub mod stroke;

/// Default pen radius (em unit)。BIZ UDPGothic Regular に近い weight。
pub const DEFAULT_PEN_WIDTH: f32 = 0.045;

/// Glyph builder — declarative stroke accumulator。
///
/// v0.7.2-alpha: `widths` per-stroke variable pen support added。
#[derive(Debug, Clone)]
pub struct GlyphBuilder {
    codepoint: char,
    strokes: Vec<Stroke>,
    /// v0.7.2-alpha: per-stroke (width_start, width_end)、`.add(...)` は default 均一。
    widths: Vec<(f32, f32)>,
    pen_width: f32,
    advance: f32,
}

impl GlyphBuilder {
    /// New builder for `codepoint`、default pen/advance。
    #[must_use]
    pub fn new(codepoint: char) -> Self {
        Self {
            codepoint,
            strokes: Vec::new(),
            widths: Vec::new(),
            pen_width: DEFAULT_PEN_WIDTH,
            advance: 1.0,
        }
    }

    /// Add 1 stroke with uniform default pen width。
    ///
    /// max = `MAX_GLYPH_STROKES` (追加分は build 時に truncate)。
    #[must_use]
    #[allow(clippy::should_implement_trait)] // Builder method、std::ops::Add trait とは無関係
    pub fn add(mut self, stroke: Stroke) -> Self {
        self.strokes.push(stroke);
        self.widths.push((self.pen_width, self.pen_width));
        self
    }

    /// v0.7.2-alpha: Add 1 stroke with variable pen widths (taper along Bezier t)。
    ///
    /// - `w_start`: pen radius at Bezier t=0 (stroke start)
    /// - `w_end`: pen radius at Bezier t=1 (stroke end)
    /// - Shader が `mix(w_start, w_end, t)` で interpolate、tapered / heavy-end strokes を表現。
    ///
    /// Ex: 撇 taper = `.add_variable(left_falling(...), 0.06, 0.02)`
    ///     捺 heavy end = `.add_variable(right_falling(...), 0.02, 0.08)`
    #[must_use]
    pub fn add_variable(mut self, stroke: Stroke, w_start: f32, w_end: f32) -> Self {
        self.strokes.push(stroke);
        self.widths.push((w_start, w_end));
        self
    }

    /// Pen radius (em unit) 設定。
    #[must_use]
    pub fn weight(mut self, w: f32) -> Self {
        self.pen_width = w;
        self
    }

    /// Advance (em unit) 設定。default = 1.0。
    #[must_use]
    pub fn advance(mut self, a: f32) -> Self {
        self.advance = a;
        self
    }

    /// `SkeletonGlyph` 出力。stroke overflow は truncate、stroke count 記録。
    #[must_use]
    pub fn build(self) -> SkeletonGlyph {
        let mut strokes = [Stroke::line(Point2::ZERO, Point2::ZERO); MAX_GLYPH_STROKES];
        let mut stroke_widths = [(self.pen_width, self.pen_width); MAX_GLYPH_STROKES];
        let stroke_count = self.strokes.len().min(MAX_GLYPH_STROKES);
        strokes[..stroke_count].copy_from_slice(&self.strokes[..stroke_count]);
        stroke_widths[..stroke_count].copy_from_slice(&self.widths[..stroke_count]);
        SkeletonGlyph {
            codepoint: self.codepoint,
            strokes,
            stroke_widths,
            stroke_count,
            advance: self.advance,
            pen_width: self.pen_width,
        }
    }
}

/// Convenience constructor — `glyph('一').add(...).build()` idiom。
#[must_use]
pub fn glyph(codepoint: char) -> GlyphBuilder {
    GlyphBuilder::new(codepoint)
}

/// DSL 経由の char → `SkeletonGlyph` lookup (v0.7.1-alpha MVP 15+ radicals)。
///
/// # Returns
/// - `Some(glyph)` — DSL 定義済 (v0.7.1-alpha: `radicals` module 参照)
/// - `None` — 未定義 (fallback は consumer 側で `kanji::build_skeleton` 経由に切替可)
#[must_use]
pub fn dsl_lookup(ch: char) -> Option<SkeletonGlyph> {
    match ch {
        // ---- 数字系 (v0.7.1 + v0.7.4 拡張) -----
        '一' => Some(radicals::ichi()),
        '二' => Some(radicals::ni()),
        '三' => Some(radicals::san()),
        '四' => Some(radicals::yon()),
        '五' => Some(radicals::go()),
        '六' => Some(radicals::roku()),
        '七' => Some(radicals::shichi()),
        '八' => Some(radicals::hachi()),
        '九' => Some(radicals::kyuu()),
        '十' => Some(radicals::juu()),
        '百' => Some(radicals::hyaku()),
        '千' => Some(radicals::sen()),
        '万' => Some(radicals::man()),
        // ---- 方向・位置 (v0.7.4) -----
        '上' => Some(radicals::ue()),
        '下' => Some(radicals::shita()),
        '中' => Some(radicals::naka()),
        '前' => Some(radicals::mae()),
        '後' => Some(radicals::ushiro()),
        '左' => Some(radicals::hidari()),
        '右' => Some(radicals::migi()),
        // ---- 基本 radical (v0.7.1 + v0.7.4) -----
        '木' => Some(radicals::ki()),
        '日' => Some(radicals::hi()),
        '月' => Some(radicals::tsuki()),
        '田' => Some(radicals::ta()),
        '口' => Some(radicals::kuchi()),
        '目' => Some(radicals::me()),
        '山' => Some(radicals::yama()),
        '心' => Some(radicals::kokoro()),
        '大' => Some(radicals::dai()),
        '人' => Some(radicals::hito()),
        '力' => Some(radicals::chikara()),
        '女' => Some(radicals::onna()),
        '子' => Some(radicals::ko()),
        '土' => Some(radicals::tsuchi()),
        '水' => Some(radicals::mizu()),
        '火' => Some(radicals::hi_fire()),
        '川' => Some(radicals::kawa()),
        '石' => Some(radicals::ishi()),
        '天' => Some(radicals::ten()),
        '空' => Some(radicals::sora()),
        '花' => Some(radicals::hana()),
        '草' => Some(radicals::kusa()),
        '雨' => Some(radicals::ame()),
        '雪' => Some(radicals::yuki()),
        '白' => Some(radicals::shiro()),
        '青' => Some(radicals::ao()),
        // ---- 人体・動作 (v0.7.4) -----
        '手' => Some(radicals::te()),
        '足' => Some(radicals::ashi()),
        '耳' => Some(radicals::mimi()),
        '見' => Some(radicals::miru()),
        '走' => Some(radicals::hashiru()),
        // ---- 構造 (v0.7.4) -----
        '立' => Some(radicals::tatsu()),
        '米' => Some(radicals::kome()),
        '糸' => Some(radicals::ito()),
        '車' => Some(radicals::kuruma()),
        // ---- 合成 kanji (IDS composition、v0.7.1 + v0.7.4) -----
        '明' => Some(radicals::mei()),
        '林' => Some(radicals::hayashi()),
        '森' => Some(radicals::mori()),
        '相' => Some(radicals::sou()),
        '男' => Some(radicals::otoko()),
        '好' => Some(radicals::suki()),
        '岩' => Some(radicals::iwa()),
        '早' => Some(radicals::hayai()),
        '音' => Some(radicals::oto()),
        '意' => Some(radicals::kokoro_meaning()),
        '想' => Some(radicals::omou()),
        '志' => Some(radicals::kokorozashi()),
        '半' => Some(radicals::han()),
        '雲' => Some(radicals::kumo()),
        '電' => Some(radicals::den()),
        _ => None,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_builder_ichi_has_one_stroke() {
        let g = glyph('一')
            .add(stroke::horizontal(0.5, 0.1, 0.9))
            .weight(0.05)
            .build();
        assert_eq!(g.codepoint, '一');
        assert_eq!(g.stroke_count, 1);
        assert!((g.pen_width - 0.05).abs() < 1e-6);
        assert!((g.advance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn glyph_builder_overflow_truncates_at_max() {
        let mut b = glyph('X');
        // Add MAX_GLYPH_STROKES + 5 strokes
        for _ in 0..(MAX_GLYPH_STROKES + 5) {
            b = b.add(stroke::horizontal(0.5, 0.0, 1.0));
        }
        let g = b.build();
        assert_eq!(g.stroke_count, MAX_GLYPH_STROKES);
    }

    #[test]
    fn dsl_lookup_registered_kanji_returns_some() {
        assert!(dsl_lookup('一').is_some());
        assert!(dsl_lookup('木').is_some());
        assert!(dsl_lookup('明').is_some());
    }

    #[test]
    fn dsl_lookup_unregistered_returns_none() {
        assert!(dsl_lookup('龍').is_none());
    }

    #[test]
    fn dsl_lookup_ichi_matches_direct_builder() {
        let via_lookup = dsl_lookup('一').unwrap();
        let direct = radicals::ichi();
        assert_eq!(via_lookup.codepoint, direct.codepoint);
        assert_eq!(via_lookup.stroke_count, direct.stroke_count);
    }
}
